#!/bin/bash
# ============================================================================
# Torrust Backup Script
# ============================================================================
# Production backup script for Torrust Tracker deployments.
# Configuration is sourced from /etc/backup/backup.conf (bash format).
#
# Configuration File Format (/etc/backup/backup.conf):
#   BACKUP_RETENTION_DAYS=7         # Days to keep old backups
#   BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
#   DB_TYPE=mysql                   # mysql, sqlite, or none
#   DB_HOST=mysql                   # MySQL only
#   DB_PORT=3306                    # MySQL only
#   DB_USER=tracker_user            # MySQL only
#   DB_PASSWORD=secret              # MySQL only
#   DB_NAME=torrust_tracker         # MySQL only
#   DB_PATH=/data/storage/...       # SQLite only
#
# Mount Points:
#   /backups  - Output directory for all backups (read-write)
#   /data     - Source data directory (read-only)
#
# Output Structure:
#   /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz
#   /backups/sqlite/sqlite_YYYYMMDD_HHMMSS.db.gz
#   /backups/config/config_YYYYMMDD_HHMMSS.tar.gz
# ============================================================================

set -e
set -o pipefail

# =============================================================================
# Constants
# =============================================================================
# Allow override for testing, but set defaults for production
BACKUP_DIR_MYSQL="${BACKUP_DIR_MYSQL:-/backups/mysql}"
BACKUP_DIR_SQLITE="${BACKUP_DIR_SQLITE:-/backups/sqlite}"
BACKUP_DIR_CONFIG="${BACKUP_DIR_CONFIG:-/backups/config}"
CONFIG_FILE="${CONFIG_FILE:-/etc/backup/backup.conf}"

# =============================================================================
# Main Entry Point
# =============================================================================

# Main entry point for the backup container.
# Loads configuration, validates it, and executes a backup cycle.
# The container exits after completing the backup.
#
# Arguments:
#   None
# Returns:
#   None
# Exit codes:
#   0 - Backup completed successfully
#   1 - Configuration error or backup failure
main() {
    log "Torrust Backup Container starting"
    
    load_configuration
    validate_configuration
    print_configuration

    run_backup_cycle
    log "Backup complete - container will exit"
}

# =============================================================================
# Backup Cycle
# =============================================================================

# Executes a complete backup cycle.
# Performs database backup (MySQL/SQLite/none), config files backup,
# and cleanup of old backups based on retention policy.
#
# Arguments:
#   None
# Returns:
#   None
# Exit codes:
#   0 - Backup cycle completed successfully
#   1 - Backup operation failed
run_backup_cycle() {
    log "=========================================="
    log "Starting backup cycle"
    log "=========================================="

    # Backup database
    case "$DB_TYPE" in
        mysql)
            backup_mysql
            ;;
        sqlite)
            backup_sqlite
            ;;
        none)
            log "Database backup disabled (DB_TYPE=none)"
            ;;
        *)
            log_error "Unknown database type: $DB_TYPE"
            exit 1
            ;;
    esac

    # Backup config files
    backup_config_files

    # Clean up old backups
    cleanup_old_backups

    log "=========================================="
    log "Backup cycle completed successfully"
    log "=========================================="
}

# =============================================================================
# Configuration
# =============================================================================

# Loads backup configuration from the config file.
# Sources the bash-format configuration file and sets default values
# for optional variables.
#
# Arguments:
#   None (uses global CONFIG_FILE constant)
# Returns:
#   None
# Exit codes:
#   0 - Configuration loaded successfully
#   1 - Configuration file not found
# Sets global variables:
#   BACKUP_RETENTION_DAYS - Days to keep old backups (default: 7)
#   DB_TYPE - Database type: mysql, sqlite, or none (default: none)
#   Plus any variables defined in the config file
load_configuration() {
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi

    log "Loading configuration from: $CONFIG_FILE"
    # shellcheck source=/dev/null
    source "$CONFIG_FILE"

    # Set defaults for optional variables
    BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"
    DB_TYPE="${DB_TYPE:-none}"
}

# Validates the loaded configuration.
# Checks database-specific configuration based on DB_TYPE and
# validates backup paths file if specified.
#
# Arguments:
#   None (uses global configuration variables)
# Returns:
#   None
# Exit codes:
#   0 - Configuration is valid
#   1 - Configuration validation failed
validate_configuration() {
    if [ "$DB_TYPE" = "mysql" ]; then
        validate_mysql_config
    elif [ "$DB_TYPE" = "sqlite" ]; then
        validate_sqlite_config
    fi

    if [ -n "$BACKUP_PATHS_FILE" ]; then
        if [ ! -f "$BACKUP_PATHS_FILE" ]; then
            log_error "Backup paths file not found: $BACKUP_PATHS_FILE"
            exit 1
        fi
    fi
}

# Validates MySQL database configuration.
# Checks that all required MySQL variables are set.
#
# Arguments:
#   None (uses global DB_* variables)
# Returns:
#   None
# Exit codes:
#   0 - All required MySQL variables are set
#   1 - One or more required variables are missing
# Required variables:
#   DB_HOST, DB_PORT, DB_USER, DB_PASSWORD, DB_NAME
validate_mysql_config() {
    local missing=()
    [ -z "${DB_HOST:-}" ] && missing+=("DB_HOST")
    [ -z "${DB_PORT:-}" ] && missing+=("DB_PORT")
    [ -z "${DB_USER:-}" ] && missing+=("DB_USER")
    [ -z "${DB_PASSWORD:-}" ] && missing+=("DB_PASSWORD")
    [ -z "${DB_NAME:-}" ] && missing+=("DB_NAME")

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "MySQL backup enabled but missing required variables: ${missing[*]}"
        exit 1
    fi
}

# Validates SQLite database configuration.
# Checks that DB_PATH is set and points to an existing file.
#
# Arguments:
#   None (uses global DB_PATH variable)
# Returns:
#   None
# Exit codes:
#   0 - DB_PATH is set and file exists
#   1 - DB_PATH not set or file not found
validate_sqlite_config() {
    if [ -z "${DB_PATH:-}" ]; then
        log_error "SQLite backup enabled but DB_PATH not set"
        exit 1
    fi

    if [ ! -f "$DB_PATH" ]; then
        log_error "SQLite database file not found: $DB_PATH"
        exit 1
    fi
}

# Prints the loaded configuration to stderr.
# Logs retention policy, database type, and optional paths file.
#
# Arguments:
#   None (uses global configuration variables)
# Returns:
#   None
print_configuration() {
    log "Configuration:"
    log "  Retention: $BACKUP_RETENTION_DAYS days"
    log "  Database: $DB_TYPE"
    if [ -n "$BACKUP_PATHS_FILE" ]; then
        log "  Config paths file: $BACKUP_PATHS_FILE"
    fi
}

# =============================================================================
# Backup Operations
# =============================================================================

# Generates a timestamp for backup filenames.
# Format: YYYYMMDD_HHMMSS
#
# Arguments:
#   None
# Outputs:
#   Timestamp string to stdout
generate_timestamp() {
    date +%Y%m%d_%H%M%S
}

# Logs backup file size after successful completion.
#
# Arguments:
#   $1 - Path to backup file
# Returns:
#   None
log_backup_completion() {
    local backup_file="$1"
    log "  Size: $(du -h "$backup_file" | cut -f1)"
}

# Ensures a backup directory exists.
# Creates the directory if it doesn't exist, including parent directories.
#
# Arguments:
#   $1 - Path to backup directory
# Returns:
#   None
ensure_backup_directory() {
    local backup_dir="$1"
    mkdir -p "$backup_dir"
}

# Reads and validates backup paths from a file.
# Skips comments (lines starting with #) and empty lines.
# Logs warnings for paths that don't exist.
#
# Arguments:
#   $1 - Path to file containing backup paths (one per line)
# Outputs:
#   Valid paths to stdout (one per line)
# Returns:
#   None (outputs empty string if no valid paths found)
read_backup_paths() {
    local paths_file="$1"
    local paths=()
    
    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "${line// /}" ]] && continue
        
        # Check if path exists
        if [ -e "$line" ]; then
            paths+=("$line")
        else
            log "Warning: Path not found, skipping: $line"
        fi
    done < "$paths_file"
    
    # Return paths as array elements, one per line
    # Use conditional to handle empty arrays correctly
    if [ ${#paths[@]} -gt 0 ]; then
        printf '%s\n' "${paths[@]}"
    fi
}

# Performs MySQL database backup.
# Uses mysqldump with --single-transaction for consistent snapshot.
# Compresses output with gzip.
#
# Arguments:
#   None (uses global DB_* and BACKUP_DIR_MYSQL variables)
# Returns:
#   None
# Exit codes:
#   0 - Backup completed successfully
#   1 - Backup failed (partial file is removed)
# Output file:
#   /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz
backup_mysql() {
    local timestamp
    timestamp=$(generate_timestamp)
    local backup_file="$BACKUP_DIR_MYSQL/mysql_${timestamp}.sql.gz"

    log "Starting MySQL backup: $DB_NAME@$DB_HOST:$DB_PORT"

    ensure_backup_directory "$BACKUP_DIR_MYSQL"

    # Perform backup with compression
    # Use MYSQL_PWD env var to avoid password on command line
    export MYSQL_PWD="$DB_PASSWORD"
    
    if mysqldump \
        --defaults-file=/etc/mysql/mysql-client.cnf \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --user="$DB_USER" \
        --single-transaction \
        --quick \
        --lock-tables=false \
        "$DB_NAME" | gzip > "$backup_file"; then
        log "MySQL backup completed: $backup_file"
        log_backup_completion "$backup_file"
    else
        log_error "MySQL backup failed"
        rm -f "$backup_file"
        exit 1
    fi
}

# Performs SQLite database backup.
# Uses SQLite's .backup command for safe online backup.
# Compresses output with gzip.
#
# Arguments:
#   None (uses global DB_PATH and BACKUP_DIR_SQLITE variables)
# Returns:
#   None
# Exit codes:
#   0 - Backup completed successfully
#   1 - Backup or compression failed (partial files are removed)
# Output file:
#   /backups/sqlite/sqlite_YYYYMMDD_HHMMSS.db.gz
backup_sqlite() {
    local timestamp
    timestamp=$(generate_timestamp)
    local backup_file="$BACKUP_DIR_SQLITE/sqlite_${timestamp}.db.gz"

    log "Starting SQLite backup: $DB_PATH"

    ensure_backup_directory "$BACKUP_DIR_SQLITE"

    # Create temporary uncompressed backup
    local temp_backup="${backup_file%.gz}"
    
    # Use SQLite .backup command for safe online backup
    if sqlite3 "$DB_PATH" ".backup '$temp_backup'"; then
        # Compress the backup
        if gzip "$temp_backup"; then
            log "SQLite backup completed: $backup_file"
            log_backup_completion "$backup_file"
        else
            log_error "SQLite compression failed"
            rm -f "$temp_backup" "$backup_file"
            exit 1
        fi
    else
        log_error "SQLite backup failed"
        rm -f "$temp_backup"
        exit 1
    fi
}

# Performs configuration files backup.
# Reads paths from BACKUP_PATHS_FILE and creates a compressed tar archive.
# Preserves absolute paths in the archive.
#
# Arguments:
#   None (uses global BACKUP_PATHS_FILE and BACKUP_DIR_CONFIG variables)
# Returns:
#   0 - Backup completed successfully or skipped (no paths file)
#   1 - Backup failed (partial file is removed)
# Output file:
#   /backups/config/config_YYYYMMDD_HHMMSS.tar.gz
backup_config_files() {
    if [ -z "$BACKUP_PATHS_FILE" ]; then
        log "No backup paths file specified, skipping config backup"
        return 0
    fi

    local timestamp
    timestamp=$(generate_timestamp)
    local backup_file="$BACKUP_DIR_CONFIG/config_${timestamp}.tar.gz"

    log "Starting config files backup"

    ensure_backup_directory "$BACKUP_DIR_CONFIG"

    # Read and validate paths from file
    local paths=()
    mapfile -t paths < <(read_backup_paths "$BACKUP_PATHS_FILE")

    if [ ${#paths[@]} -eq 0 ]; then
        log "No valid paths to backup"
        return 0
    fi

    # Create tar archive with compression
    # Use -C / to preserve absolute paths in archive
    if tar -czf "$backup_file" -C / "${paths[@]}" 2>/dev/null; then
        log "Config backup completed: $backup_file"
        log "  Files backed up: ${#paths[@]}"
        log_backup_completion "$backup_file"
    else
        log_error "Config backup failed"
        rm -f "$backup_file"
        exit 1
    fi
}

# =============================================================================
# Cleanup Operations
# =============================================================================

# Cleans up old backups across all backup directories.
# Removes backups older than BACKUP_RETENTION_DAYS.
# Logs the total count of deleted files.
#
# Arguments:
#   None (uses global BACKUP_RETENTION_DAYS variable)
# Returns:
#   None
cleanup_old_backups() {
    log "Cleaning up backups older than $BACKUP_RETENTION_DAYS days"

    local deleted_count=0

    deleted_count=$((deleted_count + $(cleanup_mysql_backups)))
    deleted_count=$((deleted_count + $(cleanup_sqlite_backups)))
    deleted_count=$((deleted_count + $(cleanup_config_backups)))

    if [ $deleted_count -eq 0 ]; then
        log "  No old backups to delete"
    else
        log "  Deleted $deleted_count old backup(s)"
    fi
}

# Cleans up old MySQL backups.
# Removes files matching mysql_*.sql.gz older than retention period.
#
# Arguments:
#   None (uses global BACKUP_DIR_MYSQL and BACKUP_RETENTION_DAYS)
# Outputs:
#   Count of deleted files to stdout
cleanup_mysql_backups() {
    cleanup_backup_directory "$BACKUP_DIR_MYSQL" "mysql_*.sql.gz" "MySQL"
}

# Cleans up old SQLite backups.
# Removes files matching sqlite_*.db.gz older than retention period.
#
# Arguments:
#   None (uses global BACKUP_DIR_SQLITE and BACKUP_RETENTION_DAYS)
# Outputs:
#   Count of deleted files to stdout
cleanup_sqlite_backups() {
    cleanup_backup_directory "$BACKUP_DIR_SQLITE" "sqlite_*.db.gz" "SQLite"
}

# Cleans up old config backups.
# Removes files matching config_*.tar.gz older than retention period.
#
# Arguments:
#   None (uses global BACKUP_DIR_CONFIG and BACKUP_RETENTION_DAYS)
# Outputs:
#   Count of deleted files to stdout
cleanup_config_backups() {
    cleanup_backup_directory "$BACKUP_DIR_CONFIG" "config_*.tar.gz" "config"
}

# Generic backup directory cleanup.
# Finds and removes backup files older than retention period.
#
# Arguments:
#   $1 - Backup directory path
#   $2 - File pattern (e.g., "mysql_*.sql.gz")
#   $3 - Backup type name (for logging)
# Outputs:
#   Count of deleted files to stdout
cleanup_backup_directory() {
    local backup_dir="$1"
    local file_pattern="$2"
    local backup_type="$3"
    local count=0
    
    if [ -d "$backup_dir" ]; then
        while IFS= read -r file; do
            rm -f "$file"
            log "  Deleted old $backup_type backup: $(basename "$file")"
            ((count++))
        done < <(find "$backup_dir" -name "$file_pattern" -type f -mtime +"$BACKUP_RETENTION_DAYS")
    fi
    
    echo "$count"
}

# =============================================================================
# Logging Utilities
# =============================================================================

# Logs a message to stderr with timestamp.
# Format: [YYYY-MM-DD HH:MM:SS] message
#
# Arguments:
#   $* - Message to log
# Outputs:
#   Timestamped message to stderr
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" >&2
}

# Logs an error message to stderr with timestamp and ERROR prefix.
# Format: [YYYY-MM-DD HH:MM:SS] ERROR: message
#
# Arguments:
#   $* - Error message to log
# Outputs:
#   Timestamped error message to stderr
log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2
}

# Run main if script is executed (not sourced)
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
