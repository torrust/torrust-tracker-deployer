#!/bin/bash
# ============================================================================
# Unified Backup Script
# ============================================================================
# Configuration-driven backup script. All behavior is controlled via
# environment variables and a paths file - no rebuild needed.
#
# Environment Variables:
#   BACKUP_INTERVAL        - Seconds between backups (default: 86400 = 24h)
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
#   - Packages config files older than 1 hour (gzip)
#   - Deletes backups older than BACKUP_RETENTION_DAYS
# ============================================================================

set -e

# =============================================================================
# Constants
# =============================================================================
# These can be overridden for testing by setting them before sourcing the script.

BACKUP_DIR_MYSQL="${BACKUP_DIR_MYSQL:-/backups/mysql}"
BACKUP_DIR_CONFIG="${BACKUP_DIR_CONFIG:-/backups/config}"
PACKAGE_AGE_MINUTES="${PACKAGE_AGE_MINUTES:-60}"

# =============================================================================
# Main Entry Point
# =============================================================================

# Orchestrates the backup container lifecycle.
#
# Supports two modes:
#   - CONTINUOUS (default): Runs backups at intervals forever (sidecar pattern)
#   - SINGLE: Runs one backup and exits (maintenance-window pattern)
#
# Mode is controlled via BACKUP_MODE environment variable:
#   BACKUP_MODE=single     - Run once and exit
#   BACKUP_MODE=continuous - Run forever (default)
#
# Side Effects:
#   - Exits with code 1 if MySQL is enabled but misconfigured
#   - In continuous mode: runs indefinitely until the container is stopped
#   - In single mode: exits after one backup cycle
main() {
    validate_configuration
    print_configuration
    
    if is_single_mode; then
        log "Running in SINGLE mode (one backup, then exit)"
        run_single_backup
    else
        log "Running in CONTINUOUS mode (scheduled backups)"
        run_initial_backup
        run_scheduled_backups
    fi
}

# Checks if backup should run once and exit.
#
# Single mode is used for maintenance-window backups where the host
# orchestrates the container lifecycle.
#
# Returns:
#   0 (true) if BACKUP_MODE is "single", 1 (false) otherwise
is_single_mode() {
    [ "${BACKUP_MODE:-continuous}" = "single" ]
}

# Runs a single backup cycle and exits.
#
# Used by maintenance-window orchestration where the backup container
# is run once by the host script, not as a continuous sidecar.
run_single_backup() {
    run_backup_cycle
    log "Single backup complete - container will exit"
}

# Validates all enabled backup sources have required configuration.
#
# Side Effects:
#   - Delegates to source-specific validators
#   - May exit the script if validation fails
validate_configuration() {
    if is_mysql_enabled; then
        validate_mysql_configuration
    fi
}

# Ensures all required MySQL environment variables are set.
#
# Required Variables:
#   MYSQL_HOST, MYSQL_DATABASE, MYSQL_USER, MYSQL_PASSWORD
#
# Side Effects:
#   - Exits with code 1 if any required variable is missing
#   - Logs descriptive error message listing missing variables
validate_mysql_configuration() {
    local missing=()

    [ -z "${MYSQL_HOST:-}" ] && missing+=("MYSQL_HOST")
    [ -z "${MYSQL_DATABASE:-}" ] && missing+=("MYSQL_DATABASE")
    [ -z "${MYSQL_USER:-}" ] && missing+=("MYSQL_USER")
    [ -z "${MYSQL_PASSWORD:-}" ] && missing+=("MYSQL_PASSWORD")

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "MySQL backup enabled but missing required variables: ${missing[*]}"
        exit 1
    fi
}

# Logs the current configuration on startup.
#
# Displays: mode, interval, retention days, MySQL enabled state, paths file location.
print_configuration() {
    log "Backup container starting..."
    log "Configuration:"
    log_item "Mode: ${BACKUP_MODE:-continuous}"
    log_item "Interval: $(get_interval)s"
    log_item "Retention: $(get_retention_days) days"
    log_item "MySQL backup: ${BACKUP_MYSQL_ENABLED:-false}"
    log_item "Paths file: ${BACKUP_PATHS_FILE:-<not set>}"
}

# Executes an immediate backup cycle on startup.
#
# Ensures data is backed up as soon as the container starts,
# without waiting for the first scheduled interval.
run_initial_backup() {
    run_backup_cycle
}

# Runs backup cycles in an infinite loop at the configured interval.
#
# This is the main scheduling loop - runs forever until container stops.
run_scheduled_backups() {
    local interval
    interval=$(get_interval)

    while true; do
        sleep "${interval}"
        run_backup_cycle
    done
}

# Executes a complete backup cycle: MySQL, config files, then maintenance.
#
# Sequence:
#   1. Backup MySQL database (if enabled)
#   2. Backup config files (if paths file configured)
#   3. Run maintenance (package old files, delete expired backups)
run_backup_cycle() {
    log_header "Backup cycle starting"

    backup_mysql
    backup_config_files
    run_maintenance

    log_header "Backup cycle complete"
}

# =============================================================================
# MySQL Backup
# =============================================================================

# Creates a compressed backup of the MySQL database.
#
# The backup is streamed directly through gzip to minimize disk I/O
# and storage requirements. Uses single-transaction mode for consistency.
#
# Side Effects:
#   - Creates /backups/mysql directory if it doesn't exist
#   - Writes compressed SQL dump to /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz
#   - Logs progress and final file size
backup_mysql() {
    if ! is_mysql_enabled; then
        return 0
    fi

    log "Starting MySQL backup..."

    ensure_directory_exists "${BACKUP_DIR_MYSQL}"

    local output_file
    output_file=$(generate_mysql_backup_path)

    log_item "Database: ${MYSQL_DATABASE}@${MYSQL_HOST}:${MYSQL_PORT:-3306}"

    dump_mysql_database "${output_file}"

    log_item "Output: ${output_file} ($(get_file_size "${output_file}"))"
    log "MySQL backup complete"
}

# Generates a timestamped file path for a MySQL backup.
#
# Returns:
#   Path string: /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz
generate_mysql_backup_path() {
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    echo "${BACKUP_DIR_MYSQL}/mysql_${timestamp}.sql.gz"
}

# Executes mariadb-dump and pipes output through gzip to the target file.
#
# Arguments:
#   $1 - output_file: Path where the compressed dump will be written
#
# Options used:
#   --single-transaction: Consistent snapshot without locking tables
#   --routines: Include stored procedures and functions
#   --triggers: Include triggers
#   --no-tablespaces: Skip tablespace statements (avoids permission issues)
dump_mysql_database() {
    local output_file="$1"

    mariadb-dump \
        --host="${MYSQL_HOST}" \
        --port="${MYSQL_PORT:-3306}" \
        --user="${MYSQL_USER}" \
        --password="${MYSQL_PASSWORD}" \
        --ssl=0 \
        --single-transaction \
        --routines \
        --triggers \
        --no-tablespaces \
        "${MYSQL_DATABASE}" | gzip > "${output_file}"
}

# =============================================================================
# Config Files Backup
# =============================================================================

# Backs up configuration files listed in the paths file.
#
# Reads paths from BACKUP_PATHS_FILE, copies each file/directory to the
# backup location preserving the relative path structure under /data/.
#
# Side Effects:
#   - Creates /backups/config directory structure as needed
#   - Copies files preserving directory hierarchy
#   - Logs each copied path and any missing paths as warnings
backup_config_files() {
    local paths_file
    paths_file=$(get_paths_file)

    if ! has_valid_paths_file "${paths_file}"; then
        log "Config backup skipped: no paths file configured"
        return 0
    fi

    log "Starting config backup from: ${paths_file}"

    ensure_directory_exists "${BACKUP_DIR_CONFIG}"

    # Process paths and count results (avoid subshell to preserve logging)
    local copied=0
    local errors=0

    while IFS= read -r line || [[ -n "$line" ]]; do
        if is_comment_or_empty "$line"; then
            continue
        fi

        local path
        path=$(trim_whitespace "$line")

        if [ ! -e "$path" ]; then
            log_item "Warning: not found: ${path}"
            errors=$((errors + 1))
        else
            copy_to_backup_directory "$path"
            log_item "Copied: ${path}"
            copied=$((copied + 1))
        fi
    done < "${paths_file}"

    log "Config backup complete: ${copied} items copied, ${errors} not found"
}

# Checks if the paths file is configured and exists.
#
# Arguments:
#   $1 - paths_file: Path to the backup paths configuration file
#
# Returns:
#   0 (true) if file is set and exists, 1 (false) otherwise
has_valid_paths_file() {
    local paths_file="$1"
    [ -n "${paths_file}" ] && [ -f "${paths_file}" ]
}

# Backs up a single path, logging success or failure.
#
# Arguments:
#   $1 - source_path: Path to the file or directory to backup
#
# Returns:
#   0 on success, 1 if path doesn't exist
backup_single_path() {
    local source_path="$1"

    if [ ! -e "$source_path" ]; then
        log_item "Warning: not found: ${source_path}"
        return 1
    fi

    copy_to_backup_directory "$source_path"
    log_item "Copied: ${source_path}"
    return 0
}

# Copies a path to the backup directory, preserving relative structure.
#
# The source path is assumed to be under /data/. The relative portion
# after /data/ is preserved in the backup directory structure.
#
# Arguments:
#   $1 - source_path: Absolute path to copy (must be under /data/)
#
# Example:
#   /data/storage/tracker/etc/config.toml
#   -> /backups/config/storage/tracker/etc/config.toml
copy_to_backup_directory() {
    local source_path="$1"
    local relative_path="${source_path#/data/}"
    local target_dir
    target_dir="${BACKUP_DIR_CONFIG}/$(dirname "$relative_path")"

    mkdir -p "$target_dir"
    cp -r "$source_path" "$target_dir/"
}

# =============================================================================
# Maintenance (Packaging & Retention)
# =============================================================================

# Runs post-backup maintenance tasks.
#
# Maintenance is performed after each backup cycle to:
#   1. Compress old config files to save space
#   2. Delete backups older than the retention period
run_maintenance() {
    log "Running maintenance..."
    package_old_config_files
    delete_expired_backups
}

# Compresses uncompressed config files older than PACKAGE_AGE_MINUTES.
#
# This two-phase approach (copy raw, then compress later) allows for:
#   - Quick initial backups (no compression overhead)
#   - Deduplication-friendly storage for recent files
#   - Space savings for older files via gzip
#
# Side Effects:
#   - Compresses files in-place with gzip (adds .gz extension)
#   - Logs the count of packaged files
package_old_config_files() {
    if [ ! -d "${BACKUP_DIR_CONFIG}" ]; then
        return 0
    fi

    local count
    count=$(find "${BACKUP_DIR_CONFIG}" -type f ! -name "*.gz" -mmin +"${PACKAGE_AGE_MINUTES}" 2>/dev/null | wc -l)

    if [ "$count" -gt 0 ]; then
        log_item "Packaging ${count} config file(s) older than ${PACKAGE_AGE_MINUTES} minutes..."
        find "${BACKUP_DIR_CONFIG}" -type f ! -name "*.gz" -mmin +"${PACKAGE_AGE_MINUTES}" -exec gzip {} \;
    fi
}

# Finds uncompressed files older than PACKAGE_AGE_MINUTES.
#
# Arguments:
#   $1 - directory: Directory to search
#
# Returns:
#   Newline-separated list of matching file paths (stdout)
find_uncompressed_old_files() {
    local directory="$1"
    find "$directory" -type f ! -name "*.gz" -mmin +"${PACKAGE_AGE_MINUTES}" 2>/dev/null || true
}

# Deletes backups older than the configured retention period.
#
# Applies retention policy to both MySQL dumps and config files.
# Also cleans up any empty directories left after deletion.
#
# Side Effects:
#   - Deletes old backup files from both backup directories
#   - Removes empty directories in the config backup tree
#   - Logs total count of deleted files
delete_expired_backups() {
    local retention_days
    retention_days=$(get_retention_days)

    local mysql_deleted config_deleted
    mysql_deleted=$(delete_old_files_from "${BACKUP_DIR_MYSQL}" "*.sql.gz" "${retention_days}")
    config_deleted=$(delete_old_files_from "${BACKUP_DIR_CONFIG}" "*" "${retention_days}")

    cleanup_empty_directories "${BACKUP_DIR_CONFIG}"

    if [ "$mysql_deleted" -gt 0 ] || [ "$config_deleted" -gt 0 ]; then
        log_item "Retention: removed ${mysql_deleted} MySQL, ${config_deleted} config files"
    fi
}

# Deletes files matching a pattern older than specified days.
#
# Arguments:
#   $1 - directory: Directory to search
#   $2 - pattern: Glob pattern to match (e.g., "*.sql.gz", "*")
#   $3 - days: Delete files older than this many days
#
# Returns:
#   Number of deleted files (stdout)
delete_old_files_from() {
    local directory="$1"
    local pattern="$2"
    local days="$3"

    if [ ! -d "$directory" ]; then
        echo 0
        return
    fi

    local count
    count=$(find "$directory" -name "$pattern" -type f -mtime +"${days}" 2>/dev/null | wc -l)

    if [ "$count" -gt 0 ]; then
        find "$directory" -name "$pattern" -type f -mtime +"${days}" -delete 2>/dev/null || true
    fi

    echo "$count"
}

# Removes empty directories from a directory tree.
#
# Arguments:
#   $1 - directory: Root directory to clean
#
# Side Effects:
#   - Deletes all empty directories recursively
cleanup_empty_directories() {
    local directory="$1"
    if [ -d "$directory" ]; then
        find "$directory" -type d -empty -delete 2>/dev/null || true
    fi
}

# =============================================================================
# Text Processing Helpers
# =============================================================================

# Checks if a line is a comment or empty (for paths file parsing).
#
# Arguments:
#   $1 - line: The line to check
#
# Returns:
#   0 (true) if line is empty, whitespace-only, or starts with #
#   1 (false) otherwise
is_comment_or_empty() {
    local line="$1"
    [[ -z "$line" || "$line" =~ ^[[:space:]]*$ || "$line" =~ ^[[:space:]]*# ]]
}

# Removes leading and trailing whitespace from a string.
#
# Arguments:
#   $1 - string: The string to trim
#
# Returns:
#   Trimmed string (stdout)
trim_whitespace() {
    echo "$1" | xargs
}

# =============================================================================
# File System Helpers
# =============================================================================

# Creates a directory if it doesn't exist (like mkdir -p).
#
# Arguments:
#   $1 - directory: Path to create
ensure_directory_exists() {
    local directory="$1"
    mkdir -p "$directory"
}

# Gets the human-readable size of a file.
#
# Arguments:
#   $1 - file: Path to the file
#
# Returns:
#   Size string (e.g., "4.2K", "1.5M") via stdout
get_file_size() {
    local file="$1"
    du -h "$file" | cut -f1
}

# =============================================================================
# Configuration Getters
# =============================================================================

# Returns the backup interval in seconds.
#
# Returns:
#   BACKUP_INTERVAL or 86400 (default, 24 hours)
get_interval() {
    echo "${BACKUP_INTERVAL:-86400}"
}

# Returns the backup retention period in days.
#
# Returns:
#   BACKUP_RETENTION_DAYS or 7 (default)
get_retention_days() {
    echo "${BACKUP_RETENTION_DAYS:-7}"
}

# Returns the path to the backup paths configuration file.
#
# Returns:
#   BACKUP_PATHS_FILE or empty string
get_paths_file() {
    echo "${BACKUP_PATHS_FILE:-}"
}

# Checks if MySQL backup is enabled.
#
# Returns:
#   0 (true) if BACKUP_MYSQL_ENABLED is "true", 1 (false) otherwise
is_mysql_enabled() {
    [ "${BACKUP_MYSQL_ENABLED:-false}" = "true" ]
}

# =============================================================================
# Logging
# =============================================================================

# Logs a message with timestamp.
#
# Arguments:
#   $1 - message: The message to log
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Logs a section header (for major operations).
#
# Arguments:
#   $1 - title: The header title
log_header() {
    log "=== $1 ==="
}

# Logs an indented item (for sub-operations or details).
#
# Arguments:
#   $1 - message: The message to log
log_item() {
    log "  $1"
}

# Logs an error message to stderr.
#
# Arguments:
#   $1 - message: The error message
log_error() {
    log "ERROR: $1" >&2
}

# =============================================================================
# Script Execution
# =============================================================================

# Allow sourcing for testing, or run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
