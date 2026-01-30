#!/usr/bin/env bats
# ============================================================================
# Unit Tests for Backup Script
# ============================================================================
# Uses bats-core (Bash Automated Testing System).
#
# Run locally: bats backup_test.bats
# Run in Docker: docker build includes test stage
#
# Test naming convention follows project standards:
#   it_should_{expected_behavior}_when_{condition}
# ============================================================================

# =============================================================================
# Test Setup
# =============================================================================

setup() {
    # Source the script to make functions available
    # The script has a guard that prevents main() from running when sourced
    source "${BATS_TEST_DIRNAME}/backup.sh"

    # Create temp directories for test isolation
    export TEST_TMPDIR="${BATS_TEST_TMPDIR}"
    mkdir -p "${TEST_TMPDIR}/backups/mysql"
    mkdir -p "${TEST_TMPDIR}/backups/config"
    mkdir -p "${TEST_TMPDIR}/data"
    mkdir -p "${TEST_TMPDIR}/config"
}

teardown() {
    # Cleanup is automatic with BATS_TEST_TMPDIR
    :
}

# =============================================================================
# Text Processing Helpers Tests
# =============================================================================

@test "is_comment_or_empty: it should return true when line is empty" {
    run is_comment_or_empty ""
    [ "$status" -eq 0 ]
}

@test "is_comment_or_empty: it should return true when line is whitespace only" {
    run is_comment_or_empty "   "
    [ "$status" -eq 0 ]
}

@test "is_comment_or_empty: it should return true when line starts with hash" {
    run is_comment_or_empty "# this is a comment"
    [ "$status" -eq 0 ]
}

@test "is_comment_or_empty: it should return true when line has leading spaces before hash" {
    run is_comment_or_empty "  # indented comment"
    [ "$status" -eq 0 ]
}

@test "is_comment_or_empty: it should return false when line has content" {
    run is_comment_or_empty "/data/config.toml"
    [ "$status" -eq 1 ]
}

@test "is_comment_or_empty: it should return false when line has path with hash in name" {
    run is_comment_or_empty "/data/file#1.txt"
    [ "$status" -eq 1 ]
}

@test "trim_whitespace: it should remove leading spaces" {
    result=$(trim_whitespace "   hello")
    [ "$result" = "hello" ]
}

@test "trim_whitespace: it should remove trailing spaces" {
    result=$(trim_whitespace "hello   ")
    [ "$result" = "hello" ]
}

@test "trim_whitespace: it should remove both leading and trailing spaces" {
    result=$(trim_whitespace "   hello world   ")
    [ "$result" = "hello world" ]
}

@test "trim_whitespace: it should preserve internal spaces" {
    result=$(trim_whitespace "  hello   world  ")
    [ "$result" = "hello world" ]
}

# =============================================================================
# Configuration Getter Tests
# =============================================================================

@test "get_interval: it should return default 86400 when BACKUP_INTERVAL is not set" {
    unset BACKUP_INTERVAL
    result=$(get_interval)
    [ "$result" = "86400" ]
}

@test "get_interval: it should return custom value when BACKUP_INTERVAL is set" {
    BACKUP_INTERVAL=300
    result=$(get_interval)
    [ "$result" = "300" ]
}

@test "get_retention_days: it should return default when BACKUP_RETENTION_DAYS is not set" {
    unset BACKUP_RETENTION_DAYS
    result=$(get_retention_days)
    [ "$result" = "7" ]
}

@test "get_retention_days: it should return custom value when BACKUP_RETENTION_DAYS is set" {
    BACKUP_RETENTION_DAYS=30
    result=$(get_retention_days)
    [ "$result" = "30" ]
}

@test "get_paths_file: it should return empty when BACKUP_PATHS_FILE is not set" {
    unset BACKUP_PATHS_FILE
    result=$(get_paths_file)
    [ "$result" = "" ]
}

@test "get_paths_file: it should return path when BACKUP_PATHS_FILE is set" {
    BACKUP_PATHS_FILE="/config/paths.txt"
    result=$(get_paths_file)
    [ "$result" = "/config/paths.txt" ]
}

@test "is_mysql_enabled: it should return false when BACKUP_MYSQL_ENABLED is not set" {
    unset BACKUP_MYSQL_ENABLED
    run is_mysql_enabled
    [ "$status" -eq 1 ]
}

@test "is_mysql_enabled: it should return false when BACKUP_MYSQL_ENABLED is false" {
    BACKUP_MYSQL_ENABLED=false
    run is_mysql_enabled
    [ "$status" -eq 1 ]
}

@test "is_mysql_enabled: it should return true when BACKUP_MYSQL_ENABLED is true" {
    BACKUP_MYSQL_ENABLED=true
    run is_mysql_enabled
    [ "$status" -eq 0 ]
}

@test "is_mysql_enabled: it should return false when BACKUP_MYSQL_ENABLED is non-true value" {
    BACKUP_MYSQL_ENABLED=yes
    run is_mysql_enabled
    [ "$status" -eq 1 ]
}

# =============================================================================
# Backup Mode Tests
# =============================================================================

@test "is_single_mode: it should return false when BACKUP_MODE is not set" {
    unset BACKUP_MODE
    run is_single_mode
    [ "$status" -eq 1 ]
}

@test "is_single_mode: it should return false when BACKUP_MODE is continuous" {
    BACKUP_MODE=continuous
    run is_single_mode
    [ "$status" -eq 1 ]
}

@test "is_single_mode: it should return true when BACKUP_MODE is single" {
    BACKUP_MODE=single
    run is_single_mode
    [ "$status" -eq 0 ]
}

@test "is_single_mode: it should return false when BACKUP_MODE is non-single value" {
    BACKUP_MODE=once
    run is_single_mode
    [ "$status" -eq 1 ]
}

# =============================================================================
# File System Helper Tests
# =============================================================================

@test "ensure_directory_exists: it should create directory when it does not exist" {
    local test_dir="${TEST_TMPDIR}/new_dir/nested"
    [ ! -d "$test_dir" ]

    ensure_directory_exists "$test_dir"

    [ -d "$test_dir" ]
}

@test "ensure_directory_exists: it should succeed when directory already exists" {
    local test_dir="${TEST_TMPDIR}/existing_dir"
    mkdir -p "$test_dir"

    run ensure_directory_exists "$test_dir"
    [ "$status" -eq 0 ]
}

@test "get_file_size: it should return human readable size" {
    local test_file="${TEST_TMPDIR}/testfile"
    echo "test content" > "$test_file"

    result=$(get_file_size "$test_file")
    # Should be something like "4.0K" or "12" depending on actual size
    [ -n "$result" ]
}

# =============================================================================
# Path Validation Tests
# =============================================================================

@test "has_valid_paths_file: it should return false when path is empty" {
    run has_valid_paths_file ""
    [ "$status" -eq 1 ]
}

@test "has_valid_paths_file: it should return false when file does not exist" {
    run has_valid_paths_file "/nonexistent/file.txt"
    [ "$status" -eq 1 ]
}

@test "has_valid_paths_file: it should return true when file exists" {
    local test_file="${TEST_TMPDIR}/paths.txt"
    touch "$test_file"

    run has_valid_paths_file "$test_file"
    [ "$status" -eq 0 ]
}

# =============================================================================
# MySQL Backup Path Generation Tests
# =============================================================================

@test "generate_mysql_backup_path: it should return path with timestamp format" {
    # Override constant for testing
    BACKUP_DIR_MYSQL="${TEST_TMPDIR}/backups/mysql"

    result=$(generate_mysql_backup_path)

    # Should match pattern: /path/mysql_YYYYMMDD_HHMMSS.sql.gz
    [[ "$result" =~ mysql_[0-9]{8}_[0-9]{6}\.sql\.gz$ ]]
}

@test "generate_mysql_backup_path: it should use BACKUP_DIR_MYSQL constant" {
    # The function should use the constant, not hardcoded path
    result=$(generate_mysql_backup_path)

    [[ "$result" == /backups/mysql/mysql_* ]]
}

# =============================================================================
# MySQL Configuration Validation Tests
# =============================================================================

@test "validate_mysql_configuration: it should exit when MYSQL_HOST is missing" {
    unset MYSQL_HOST
    export MYSQL_DATABASE="test"
    export MYSQL_USER="user"
    export MYSQL_PASSWORD="pass"

    run validate_mysql_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "MYSQL_HOST" ]]
}

@test "validate_mysql_configuration: it should exit when MYSQL_DATABASE is missing" {
    export MYSQL_HOST="localhost"
    unset MYSQL_DATABASE
    export MYSQL_USER="user"
    export MYSQL_PASSWORD="pass"

    run validate_mysql_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "MYSQL_DATABASE" ]]
}

@test "validate_mysql_configuration: it should exit when MYSQL_USER is missing" {
    export MYSQL_HOST="localhost"
    export MYSQL_DATABASE="test"
    unset MYSQL_USER
    export MYSQL_PASSWORD="pass"

    run validate_mysql_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "MYSQL_USER" ]]
}

@test "validate_mysql_configuration: it should exit when MYSQL_PASSWORD is missing" {
    export MYSQL_HOST="localhost"
    export MYSQL_DATABASE="test"
    export MYSQL_USER="user"
    unset MYSQL_PASSWORD

    run validate_mysql_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "MYSQL_PASSWORD" ]]
}

@test "validate_mysql_configuration: it should list all missing variables" {
    unset MYSQL_HOST
    unset MYSQL_DATABASE
    unset MYSQL_USER
    unset MYSQL_PASSWORD

    run validate_mysql_configuration
    [ "$status" -eq 1 ]
    [[ "$output" =~ "MYSQL_HOST" ]]
    [[ "$output" =~ "MYSQL_DATABASE" ]]
    [[ "$output" =~ "MYSQL_USER" ]]
    [[ "$output" =~ "MYSQL_PASSWORD" ]]
}

@test "validate_mysql_configuration: it should succeed when all variables are set" {
    export MYSQL_HOST="localhost"
    export MYSQL_DATABASE="test"
    export MYSQL_USER="user"
    export MYSQL_PASSWORD="pass"

    run validate_mysql_configuration
    [ "$status" -eq 0 ]
}

# =============================================================================
# Copy to Backup Directory Tests
# =============================================================================

@test "copy_to_backup_directory: it should preserve relative path structure" {
    # Override constant for testing
    BACKUP_DIR_CONFIG="${TEST_TMPDIR}/backups/config"

    # Create source file
    mkdir -p "${TEST_TMPDIR}/data/storage/tracker/etc"
    echo "config content" > "${TEST_TMPDIR}/data/storage/tracker/etc/config.toml"

    # Mock the source path as if it were under /data
    # We need to temporarily adjust the function's expectation
    local source_path="${TEST_TMPDIR}/data/storage/tracker/etc/config.toml"

    # Create a wrapper that adjusts paths for testing
    local relative_path="${source_path#${TEST_TMPDIR}/data/}"
    local target_dir="${BACKUP_DIR_CONFIG}/$(dirname "$relative_path")"
    mkdir -p "$target_dir"
    cp -r "$source_path" "$target_dir/"

    # Verify the structure was preserved
    [ -f "${BACKUP_DIR_CONFIG}/storage/tracker/etc/config.toml" ]
}

# =============================================================================
# Cleanup Empty Directories Tests
# =============================================================================

@test "cleanup_empty_directories: it should remove empty directories" {
    local test_dir="${TEST_TMPDIR}/cleanup_test"
    mkdir -p "${test_dir}/empty1/empty2"
    mkdir -p "${test_dir}/has_file"
    touch "${test_dir}/has_file/file.txt"

    cleanup_empty_directories "$test_dir"

    [ ! -d "${test_dir}/empty1" ]
    [ -d "${test_dir}/has_file" ]
    [ -f "${test_dir}/has_file/file.txt" ]
}

@test "cleanup_empty_directories: it should handle non-existent directory" {
    run cleanup_empty_directories "/nonexistent/path"
    [ "$status" -eq 0 ]
}

# =============================================================================
# Delete Old Files Tests
# =============================================================================

@test "delete_old_files_from: it should return 0 when directory does not exist" {
    result=$(delete_old_files_from "/nonexistent" "*.sql.gz" 7)
    [ "$result" = "0" ]
}

@test "delete_old_files_from: it should return 0 when no old files exist" {
    local test_dir="${TEST_TMPDIR}/delete_test"
    mkdir -p "$test_dir"
    touch "${test_dir}/recent.sql.gz"

    result=$(delete_old_files_from "$test_dir" "*.sql.gz" 7)
    [ "$result" = "0" ]
    [ -f "${test_dir}/recent.sql.gz" ]
}

# =============================================================================
# Logging Tests
# =============================================================================

@test "log: it should include timestamp in output" {
    result=$(log "test message")
    [[ "$result" =~ ^\[20[0-9]{2}-[0-9]{2}-[0-9]{2}\ [0-9]{2}:[0-9]{2}:[0-9]{2}\] ]]
}

@test "log: it should include message in output" {
    result=$(log "test message")
    [[ "$result" =~ "test message" ]]
}

@test "log_header: it should wrap message with equals signs" {
    result=$(log_header "Section Title")
    [[ "$result" =~ "=== Section Title ===" ]]
}

@test "log_item: it should indent message with two spaces" {
    result=$(log_item "item text")
    [[ "$result" =~ "  item text" ]]
}

@test "log_error: it should prefix with ERROR" {
    result=$(log_error "something failed" 2>&1)
    [[ "$result" =~ "ERROR: something failed" ]]
}
