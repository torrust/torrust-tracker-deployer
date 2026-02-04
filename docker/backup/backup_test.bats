#!/usr/bin/env bats
# ============================================================================
# Torrust Backup Script Tests
# ============================================================================
# Comprehensive unit tests for backup.sh using bats-core.
# Tests cover configuration loading, validation, backup operations, and cleanup.
#
# Test execution: bats backup_test.bats
# ============================================================================

# Test setup - runs before each test
setup() {
    # Source the backup script without executing main
    source backup.sh

    # Create temporary test directories
    export TEST_DIR="${BATS_TEST_TMPDIR}/backup_test_$$"
    export BACKUP_DIR_MYSQL="$TEST_DIR/backups/mysql"
    export BACKUP_DIR_SQLITE="$TEST_DIR/backups/sqlite"
    export BACKUP_DIR_CONFIG="$TEST_DIR/backups/config"
    export TEST_DATA_DIR="$TEST_DIR/data"
    export TEST_CONFIG_DIR="$TEST_DIR/config"
    
    mkdir -p "$BACKUP_DIR_MYSQL" "$BACKUP_DIR_SQLITE" "$BACKUP_DIR_CONFIG"
    mkdir -p "$TEST_DATA_DIR" "$TEST_CONFIG_DIR"
    
    # Override config file location for tests
    export CONFIG_FILE="$TEST_CONFIG_DIR/backup.conf"
}

# Test teardown - runs after each test
teardown() {
    rm -rf "$TEST_DIR"
}

# =============================================================================
# Configuration Loading Tests
# =============================================================================

@test "it_should_return_error_when_configuration_file_does_not_exist" {
    run load_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Configuration file not found" ]]
}

@test "it_should_load_all_variables_when_given_valid_configuration_file" {
    cat > "$CONFIG_FILE" <<EOF
BACKUP_RETENTION_DAYS=7
DB_TYPE=none
EOF
    
    load_configuration
    [ "$BACKUP_RETENTION_DAYS" = "7" ]
    [ "$DB_TYPE" = "none" ]
}

@test "it_should_set_default_values_when_optional_variables_are_missing" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
EOF
    
    load_configuration
    [ "$BACKUP_RETENTION_DAYS" = "7" ]
}

@test "it_should_load_mysql_variables_when_database_type_is_mysql" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=mysql
DB_HOST=mysql-host
DB_PORT=3306
DB_USER=test_user
DB_PASSWORD=test_pass
DB_NAME=test_db
EOF
    
    load_configuration
    [ "$DB_TYPE" = "mysql" ]
    [ "$DB_HOST" = "mysql-host" ]
    [ "$DB_PORT" = "3306" ]
    [ "$DB_USER" = "test_user" ]
    [ "$DB_PASSWORD" = "test_pass" ]
    [ "$DB_NAME" = "test_db" ]
}

@test "it_should_load_sqlite_variables_when_database_type_is_sqlite" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=sqlite
DB_PATH=/data/test.db
EOF
    
    load_configuration
    [ "$DB_TYPE" = "sqlite" ]
    [ "$DB_PATH" = "/data/test.db" ]
}

@test "it_should_load_backup_paths_file_setting_when_specified_in_configuration" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
EOF
    
    load_configuration
    [ "$BACKUP_PATHS_FILE" = "/etc/backup/backup-paths.txt" ]
}

# =============================================================================
# MySQL Validation Tests
# =============================================================================

@test "it_should_pass_validation_when_all_mysql_required_variables_are_set" {
    DB_HOST="mysql"
    DB_PORT="3306"
    DB_USER="user"
    DB_PASSWORD="pass"
    DB_NAME="db"
    
    run validate_mysql_config
    [ "$status" -eq 0 ]
}

@test "it_should_return_error_when_mysql_host_is_missing" {
    DB_PORT="3306"
    DB_USER="user"
    DB_PASSWORD="pass"
    DB_NAME="db"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_HOST" ]]
}

@test "it_should_return_error_when_mysql_port_is_missing" {
    DB_HOST="mysql"
    DB_USER="user"
    DB_PASSWORD="pass"
    DB_NAME="db"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_PORT" ]]
}

@test "it_should_return_error_when_mysql_user_is_missing" {
    DB_HOST="mysql"
    DB_PORT="3306"
    DB_PASSWORD="pass"
    DB_NAME="db"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_USER" ]]
}

@test "it_should_return_error_when_mysql_password_is_missing" {
    DB_HOST="mysql"
    DB_PORT="3306"
    DB_USER="user"
    DB_NAME="db"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_PASSWORD" ]]
}

@test "it_should_return_error_when_mysql_database_name_is_missing" {
    DB_HOST="mysql"
    DB_PORT="3306"
    DB_USER="user"
    DB_PASSWORD="pass"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_NAME" ]]
}

@test "it_should_list_all_missing_variables_when_multiple_mysql_fields_are_absent" {
    DB_HOST="mysql"
    
    run validate_mysql_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_PORT" ]]
    [[ "$output" =~ "DB_USER" ]]
    [[ "$output" =~ "DB_PASSWORD" ]]
    [[ "$output" =~ "DB_NAME" ]]
}

# =============================================================================
# SQLite Validation Tests
# =============================================================================

@test "it_should_pass_validation_when_sqlite_database_path_is_valid" {
    DB_PATH="$TEST_DATA_DIR/test.db"
    touch "$DB_PATH"
    
    run validate_sqlite_config
    [ "$status" -eq 0 ]
}

@test "it_should_return_error_when_sqlite_database_path_is_not_set" {
    unset DB_PATH
    
    run validate_sqlite_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_PATH not set" ]]
}

@test "it_should_return_error_when_sqlite_database_file_does_not_exist" {
    DB_PATH="$TEST_DATA_DIR/nonexistent.db"
    
    run validate_sqlite_config
    [ "$status" -eq 1 ]
    [[ "$output" =~ "not found" ]]
}

# =============================================================================
# General Validation Tests
# =============================================================================

@test "it_should_pass_validation_when_database_type_is_none" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
EOF
    
    load_configuration
    run validate_configuration
    [ "$status" -eq 0 ]
}

@test "it_should_validate_mysql_configuration_when_database_type_is_mysql" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=mysql
DB_HOST=mysql
EOF
    
    load_configuration
    run validate_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "missing required variables" ]]
}

@test "it_should_validate_sqlite_configuration_when_database_type_is_sqlite" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=sqlite
EOF
    
    load_configuration
    run validate_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "DB_PATH not set" ]]
}

@test "it_should_return_error_when_backup_paths_file_does_not_exist" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
BACKUP_PATHS_FILE=/nonexistent/backup-paths.txt
EOF
    
    load_configuration
    run validate_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Backup paths file not found" ]]
}

@test "it_should_pass_validation_when_backup_paths_file_exists" {
    local paths_file="$TEST_CONFIG_DIR/backup-paths.txt"
    touch "$paths_file"
    
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
BACKUP_PATHS_FILE=$paths_file
EOF
    
    load_configuration
    run validate_configuration
    [ "$status" -eq 0 ]
}

# =============================================================================
# SQLite Backup Tests
# =============================================================================

@test "it_should_create_compressed_backup_file_when_backing_up_sqlite_database" {
    # Create a test SQLite database
    DB_PATH="$TEST_DATA_DIR/test.db"
    sqlite3 "$DB_PATH" "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT);"
    sqlite3 "$DB_PATH" "INSERT INTO test (name) VALUES ('test_data');"
    
    run backup_sqlite
    [ "$status" -eq 0 ]
    
    # Check backup file exists and is compressed
    local backup_files=("$BACKUP_DIR_SQLITE"/sqlite_*.db.gz)
    [ -f "${backup_files[0]}" ]
    
    # Verify it's a gzip file (check magic bytes)
    local first_bytes
    first_bytes=$(od -An -t x1 -N 2 "${backup_files[0]}" | tr -d ' ')
    [ "$first_bytes" = "1f8b" ]  # gzip magic bytes
}

@test "it_should_preserve_database_integrity_when_backing_up_sqlite_database" {
    # Create test database with data
    DB_PATH="$TEST_DATA_DIR/test.db"
    sqlite3 "$DB_PATH" "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT);"
    sqlite3 "$DB_PATH" "INSERT INTO users (username) VALUES ('alice'), ('bob');"
    
    backup_sqlite
    
    # Extract and verify backup
    local backup_files=("$BACKUP_DIR_SQLITE"/sqlite_*.db.gz)
    gunzip -c "${backup_files[0]}" > "$TEST_DIR/restored.db"
    
    # Verify data integrity
    local count
    count=$(sqlite3 "$TEST_DIR/restored.db" "SELECT COUNT(*) FROM users;")
    [ "$count" = "2" ]
}

@test "it_should_log_backup_completion_when_backing_up_sqlite_database" {
    DB_PATH="$TEST_DATA_DIR/test.db"
    sqlite3 "$DB_PATH" "CREATE TABLE test (id INTEGER);"
    
    run backup_sqlite
    [[ "$output" =~ "Starting SQLite backup" ]]
    [[ "$output" =~ "SQLite backup completed" ]]
}

# =============================================================================
# Config Files Backup Tests
# =============================================================================

@test "it_should_skip_backup_when_paths_file_is_not_specified" {
    unset BACKUP_PATHS_FILE
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "No backup paths file specified" ]]
}

@test "it_should_create_compressed_tar_archive_when_backing_up_config_files" {
    # Create test files
    mkdir -p "$TEST_DATA_DIR/config"
    echo "test_content" > "$TEST_DATA_DIR/config/test.conf"
    
    # Create paths file
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    echo "$TEST_DATA_DIR/config/test.conf" > "$BACKUP_PATHS_FILE"
    
    run backup_config_files
    [ "$status" -eq 0 ]
    
    # Verify archive exists
    local backup_files=("$BACKUP_DIR_CONFIG"/config_*.tar.gz)
    [ -f "${backup_files[0]}" ]
    
    # Verify it's a gzip compressed tar (check magic bytes)
    local first_bytes
    first_bytes=$(od -An -t x1 -N 2 "${backup_files[0]}" | tr -d ' ')
    [ "$first_bytes" = "1f8b" ]  # gzip magic bytes
}

@test "it_should_skip_comments_when_reading_paths_file" {
    # Create test file
    echo "content" > "$TEST_DATA_DIR/file1.txt"
    
    # Create paths file with comments
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    cat > "$BACKUP_PATHS_FILE" <<EOF
# This is a comment
$TEST_DATA_DIR/file1.txt
# Another comment
EOF
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Files backed up: 1" ]]
}

@test "it_should_skip_empty_lines_when_reading_paths_file" {
    echo "content" > "$TEST_DATA_DIR/file1.txt"
    
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    cat > "$BACKUP_PATHS_FILE" <<EOF
$TEST_DATA_DIR/file1.txt

EOF
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Files backed up: 1" ]]
}

@test "it_should_warn_about_missing_paths_when_backing_up_config_files" {
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    cat > "$BACKUP_PATHS_FILE" <<EOF
$TEST_DATA_DIR/existing.txt
$TEST_DATA_DIR/missing.txt
EOF
    
    touch "$TEST_DATA_DIR/existing.txt"
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Path not found" ]]
    [[ "$output" =~ "missing.txt" ]]
}

@test "it_should_handle_no_valid_paths_when_backing_up_config_files" {
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    echo "$TEST_DATA_DIR/nonexistent.txt" > "$BACKUP_PATHS_FILE"
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "No valid paths to backup" ]]
}

@test "it_should_backup_multiple_files_when_paths_file_contains_multiple_entries" {
    # Create multiple test files
    mkdir -p "$TEST_DATA_DIR/config"
    echo "file1" > "$TEST_DATA_DIR/config/file1.txt"
    echo "file2" > "$TEST_DATA_DIR/config/file2.txt"
    echo "file3" > "$TEST_DATA_DIR/config/file3.txt"
    
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    cat > "$BACKUP_PATHS_FILE" <<EOF
$TEST_DATA_DIR/config/file1.txt
$TEST_DATA_DIR/config/file2.txt
$TEST_DATA_DIR/config/file3.txt
EOF
    
    run backup_config_files
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Files backed up: 3" ]]
}

@test "it_should_backup_directories_when_directory_path_is_specified" {
    mkdir -p "$TEST_DATA_DIR/config/subdir"
    echo "file" > "$TEST_DATA_DIR/config/subdir/file.txt"
    
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/paths.txt"
    echo "$TEST_DATA_DIR/config" > "$BACKUP_PATHS_FILE"
    
    run backup_config_files
    [ "$status" -eq 0 ]
    
    # Verify directory was archived
    local backup_files=("$BACKUP_DIR_CONFIG"/config_*.tar.gz)
    tar -tzf "${backup_files[0]}" | grep -q "config/subdir/file.txt"
}

# =============================================================================
# Cleanup Tests
# =============================================================================

@test "it_should_remove_mysql_backups_when_older_than_retention_period" {
    # Create old and new backup files
    touch "$BACKUP_DIR_MYSQL/mysql_20200101_120000.sql.gz"
    touch "$BACKUP_DIR_MYSQL/mysql_$(date +%Y%m%d_%H%M%S).sql.gz"
    
    # Make the old file appear old (8 days)
    touch -t 202001010000 "$BACKUP_DIR_MYSQL/mysql_20200101_120000.sql.gz"
    
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    
    # Old file should be deleted
    [ ! -f "$BACKUP_DIR_MYSQL/mysql_20200101_120000.sql.gz" ]
    
    # New file should remain
    local new_files=("$BACKUP_DIR_MYSQL"/mysql_*.sql.gz)
    [ -f "${new_files[0]}" ]
}

@test "it_should_remove_sqlite_backups_when_older_than_retention_period" {
    touch "$BACKUP_DIR_SQLITE/sqlite_20200101_120000.db.gz"
    touch "$BACKUP_DIR_SQLITE/sqlite_$(date +%Y%m%d_%H%M%S).db.gz"
    
    touch -t 202001010000 "$BACKUP_DIR_SQLITE/sqlite_20200101_120000.db.gz"
    
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    
    [ ! -f "$BACKUP_DIR_SQLITE/sqlite_20200101_120000.db.gz" ]
}

@test "it_should_remove_config_backups_when_older_than_retention_period" {
    touch "$BACKUP_DIR_CONFIG/config_20200101_120000.tar.gz"
    touch "$BACKUP_DIR_CONFIG/config_$(date +%Y%m%d_%H%M%S).tar.gz"
    
    touch -t 202001010000 "$BACKUP_DIR_CONFIG/config_20200101_120000.tar.gz"
    
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    
    [ ! -f "$BACKUP_DIR_CONFIG/config_20200101_120000.tar.gz" ]
}

@test "it_should_keep_recent_backups_when_cleaning_old_backups" {
    touch "$BACKUP_DIR_MYSQL/mysql_$(date +%Y%m%d_%H%M%S).sql.gz"
    
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    
    local files=("$BACKUP_DIR_MYSQL"/mysql_*.sql.gz)
    [ -f "${files[0]}" ]
}

@test "it_should_handle_empty_backup_directories_when_cleaning_old_backups" {
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    [[ "$output" =~ "No old backups to delete" ]]
}

@test "it_should_log_deleted_files_count_when_cleaning_old_backups" {
    touch "$BACKUP_DIR_MYSQL/mysql_20200101_120000.sql.gz"
    touch "$BACKUP_DIR_SQLITE/sqlite_20200101_120000.db.gz"
    
    touch -t 202001010000 "$BACKUP_DIR_MYSQL/mysql_20200101_120000.sql.gz"
    touch -t 202001010000 "$BACKUP_DIR_SQLITE/sqlite_20200101_120000.db.gz"
    
    BACKUP_RETENTION_DAYS=7
    run cleanup_old_backups
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Deleted 2 old backup(s)" ]]
}

# =============================================================================
# Backup Cycle Tests
# =============================================================================

@test "it_should_handle_backup_cycle_when_database_type_is_none" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=none
EOF
    
    load_configuration
    run run_backup_cycle
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Database backup disabled" ]]
}

@test "it_should_fail_backup_cycle_when_database_type_is_unknown" {
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=unknown
EOF
    
    load_configuration
    run run_backup_cycle
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Unknown database type" ]]
}

@test "it_should_perform_sqlite_backup_when_running_backup_cycle" {
    DB_PATH="$TEST_DATA_DIR/test.db"
    sqlite3 "$DB_PATH" "CREATE TABLE test (id INTEGER);"
    
    cat > "$CONFIG_FILE" <<EOF
DB_TYPE=sqlite
DB_PATH=$DB_PATH
BACKUP_RETENTION_DAYS=7
EOF
    
    load_configuration
    run run_backup_cycle
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Starting SQLite backup" ]]
    [[ "$output" =~ "Backup cycle completed" ]]
}

# =============================================================================
# Logging Tests
# =============================================================================

@test "it_should_output_to_stderr_with_timestamp_when_logging" {
    run log "test message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "[20" ]]  # Timestamp starts with year
    [[ "$output" =~ "test message" ]]
}

@test "it_should_output_to_stderr_with_error_prefix_when_logging_errors" {
    run log_error "error message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "ERROR" ]]
    [[ "$output" =~ "error message" ]]
}

# =============================================================================
# Integration Test
# =============================================================================

@test "it_should_complete_full_backup_cycle_with_all_components" {
    # Setup: Create SQLite database
    DB_PATH="$TEST_DATA_DIR/tracker.db"
    sqlite3 "$DB_PATH" "CREATE TABLE torrents (id INTEGER PRIMARY KEY, info_hash TEXT);"
    sqlite3 "$DB_PATH" "INSERT INTO torrents (info_hash) VALUES ('abcd1234');"
    
    # Setup: Create config files
    mkdir -p "$TEST_DATA_DIR/config"
    echo "test_config" > "$TEST_DATA_DIR/config/app.conf"
    
    # Setup: Create paths file
    BACKUP_PATHS_FILE="$TEST_CONFIG_DIR/backup-paths.txt"
    echo "$TEST_DATA_DIR/config/app.conf" > "$BACKUP_PATHS_FILE"
    
    # Setup: Create config file
    cat > "$CONFIG_FILE" <<EOF
BACKUP_RETENTION_DAYS=7
DB_TYPE=sqlite
DB_PATH=$DB_PATH
BACKUP_PATHS_FILE=$BACKUP_PATHS_FILE
EOF
    
    load_configuration
    validate_configuration
    run run_backup_cycle
    
    [ "$status" -eq 0 ]
    
    # Verify SQLite backup exists
    local sqlite_backups=("$BACKUP_DIR_SQLITE"/sqlite_*.db.gz)
    [ -f "${sqlite_backups[0]}" ]
    
    # Verify config backup exists
    local config_backups=("$BACKUP_DIR_CONFIG"/config_*.tar.gz)
    [ -f "${config_backups[0]}" ]
    
    # Verify log messages
    [[ "$output" =~ "Starting backup cycle" ]]
    [[ "$output" =~ "SQLite backup completed" ]]
    [[ "$output" =~ "Config backup completed" ]]
    [[ "$output" =~ "Backup cycle completed successfully" ]]
}
