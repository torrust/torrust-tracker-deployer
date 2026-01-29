#!/bin/bash
# ============================================================================
# Unified Backup Script
# ============================================================================
# Configuration-driven backup script. All behavior is controlled via
# environment variables and a paths file - no rebuild needed.
#
# Environment Variables:
#   BACKUP_INTERVAL        - Seconds between backups (default: 120)
#   BACKUP_RETENTION_DAYS  - Days to keep backups before deletion (default: 7)
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
#   /config   - Backup configuration files (backup-paths.txt)
#   /data     - Source data directory (mount app storage here, read-only)
#
# Output Structure:
#   /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz  - MySQL dumps (compressed)
#   /backups/config/                             - Config file copies
#
# Maintenance:
#   After each backup cycle, the script:
#   - Compresses config files older than 1 hour
#   - Deletes backups older than BACKUP_RETENTION_DAYS
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
    run_maintenance

    log_header "Backup cycle complete"
}

print_config() {
    local interval
    interval=$(get_interval)
    local retention_days
    retention_days=$(get_retention_days)

    log "Backup sidecar starting..."
    log "Configuration:"
    log_item "Interval: ${interval}s"
    log_item "Retention: ${retention_days} days"
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
# Maintenance (Compression & Retention)
# =============================================================================

run_maintenance() {
    log "Running maintenance..."
    compress_old_config_backups
    apply_retention_policy
}

compress_old_config_backups() {
    # Compress config files older than 1 hour that aren't already compressed
    # MySQL dumps are already gzipped during creation
    local config_dir="/backups/config"

    if [ ! -d "$config_dir" ]; then
        return 0
    fi

    local count
    count=$(find "$config_dir" -type f ! -name "*.gz" -mmin +60 2>/dev/null | wc -l)

    if [ "$count" -gt 0 ]; then
        log "Compressing $count config file(s) older than 1 hour..."
        find "$config_dir" -type f ! -name "*.gz" -mmin +60 -exec gzip {} \;
        log "Compression complete"
    fi
}

apply_retention_policy() {
    local retention_days
    retention_days=$(get_retention_days)

    log "Applying retention policy: delete backups older than ${retention_days} days"

    local mysql_deleted=0
    local config_deleted=0

    # Delete old MySQL backups
    if [ -d "/backups/mysql" ]; then
        mysql_deleted=$(find /backups/mysql -name "*.sql.gz" -mtime +"${retention_days}" 2>/dev/null | wc -l)
        find /backups/mysql -name "*.sql.gz" -mtime +"${retention_days}" -delete 2>/dev/null || true
    fi

    # Delete old config backups (both compressed and uncompressed)
    if [ -d "/backups/config" ]; then
        config_deleted=$(find /backups/config -type f -mtime +"${retention_days}" 2>/dev/null | wc -l)
        find /backups/config -type f -mtime +"${retention_days}" -delete 2>/dev/null || true
        # Clean up empty directories
        find /backups/config -type d -empty -delete 2>/dev/null || true
    fi

    if [ "$mysql_deleted" -gt 0 ] || [ "$config_deleted" -gt 0 ]; then
        log "Retention cleanup: removed $mysql_deleted MySQL backup(s), $config_deleted config file(s)"
    fi
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

get_retention_days() {
    echo "${BACKUP_RETENTION_DAYS:-7}"
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
