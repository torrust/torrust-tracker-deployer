#!/bin/bash
# ============================================================================
# Unified Backup Script
# ============================================================================
# Configuration-driven backup script. All behavior is controlled via
# environment variables and a paths file - no rebuild needed.
#
# Environment Variables:
#   BACKUP_INTERVAL        - Seconds between backups (default: 120)
#   BACKUP_MYSQL_ENABLED   - Enable MySQL backup: true/false (default: false)
#   BACKUP_PATHS_FILE      - Path to file listing paths to backup (optional)
#   MYSQL_HOST, MYSQL_PORT, MYSQL_DATABASE, MYSQL_USER, MYSQL_PASSWORD
#
# Paths File Format (one path per line):
#   # Comment lines start with #
#   /data/.env
#   /data/storage/tracker/etc/tracker.toml
#   /data/storage/prometheus/etc
#   /data/storage/grafana/provisioning
#
# Mount Points:
#   /backups  - Output directory for all backups
#   /data     - Source data directory (mount app storage here, read-only)
#
# Output Structure:
#   /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz  - MySQL dumps
#   /backups/config/                             - Config file copies
# ============================================================================

set -e

# =============================================================================
# Main Entry Point
# =============================================================================

main() {
    print_config
    run_backup
    run_scheduled_backups
}

run_scheduled_backups() {
    local interval
    interval=$(get_interval)

    while true; do
        sleep "${interval}"
        run_backup
    done
}

run_backup() {
    log_header "Backup cycle starting"

    backup_mysql
    backup_config

    log_header "Backup cycle complete"
}

print_config() {
    local interval
    interval=$(get_interval)

    log "Backup sidecar starting..."
    log "Configuration:"
    log_item "Interval: ${interval}s"
    log_item "MySQL backup: ${BACKUP_MYSQL_ENABLED:-false}"
    log_item "Paths file: ${BACKUP_PATHS_FILE:-<not set>}"
}

# =============================================================================
# MySQL Backup
# =============================================================================

backup_mysql() {
    if ! is_mysql_enabled; then
        return 0
    fi

    log "Starting MySQL backup..."

    local output_dir="/backups/mysql"
    mkdir -p "${output_dir}"

    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local output_file="${output_dir}/mysql_${timestamp}.sql.gz"

    log "Database: ${MYSQL_DATABASE}@${MYSQL_HOST}:${MYSQL_PORT}"

    mariadb-dump \
        --host="${MYSQL_HOST}" \
        --port="${MYSQL_PORT}" \
        --user="${MYSQL_USER}" \
        --password="${MYSQL_PASSWORD}" \
        --ssl=0 \
        --single-transaction \
        --routines \
        --triggers \
        --no-tablespaces \
        "${MYSQL_DATABASE}" | gzip > "${output_file}"

    local size
    size=$(du -h "${output_file}" | cut -f1)
    log "MySQL backup complete: ${output_file} (${size})"
}

# =============================================================================
# Config Files Backup
# =============================================================================

backup_config() {
    local paths_file
    paths_file=$(get_paths_file)

    if [ -z "${paths_file}" ] || [ ! -f "${paths_file}" ]; then
        log "No paths file configured (BACKUP_PATHS_FILE not set or file missing)"
        return 0
    fi

    log "Starting config backup from: ${paths_file}"

    local config_dir="/backups/config"
    mkdir -p "${config_dir}"

    local count=0
    local errors=0

    while IFS= read -r line || [[ -n "$line" ]]; do
        if is_comment_or_empty "$line"; then
            continue
        fi

        local path
        path=$(trim_whitespace "$line")

        if [ -e "$path" ]; then
            copy_path_to_backup "$path" "$config_dir"
            log_item "Copied: ${path}"
            count=$((count + 1))
        else
            log_item "Warning: not found: ${path}"
            errors=$((errors + 1))
        fi
    done < "${paths_file}"

    log "Config backup complete: ${count} items copied, ${errors} not found"
}

copy_path_to_backup() {
    local source_path="$1"
    local config_dir="$2"

    local rel_path="${source_path#/data/}"
    local target_dir
    target_dir="${config_dir}/$(dirname "$rel_path")"

    mkdir -p "$target_dir"
    cp -r "$source_path" "$target_dir/"
}

# =============================================================================
# Path Processing Helpers
# =============================================================================

is_comment_or_empty() {
    local line="$1"
    [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]]
}

trim_whitespace() {
    echo "$1" | xargs
}

# =============================================================================
# Configuration Helpers
# =============================================================================

get_interval() {
    echo "${BACKUP_INTERVAL:-120}"
}

is_mysql_enabled() {
    [ "${BACKUP_MYSQL_ENABLED:-false}" = "true" ]
}

get_paths_file() {
    echo "${BACKUP_PATHS_FILE:-}"
}

# =============================================================================
# Logging Helpers
# =============================================================================

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

log_header() {
    log "=== $1 ==="
}

log_item() {
    log "  $1"
}

# =============================================================================
# Script Execution
# =============================================================================

# Allow sourcing for testing, or run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
