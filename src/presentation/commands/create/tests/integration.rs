//! Integration Tests for Create Command CLI
//!
//! This module tests the complete create command workflow including
//! configuration loading, validation, and command execution.

use tempfile::TempDir;

use crate::presentation::cli::CreateAction;
use crate::presentation::commands::create;

use super::fixtures;

/// Helper function to call the environment creation handler
fn handle_environment_creation(
    config_path: &std::path::Path,
    working_dir: &std::path::Path,
) -> Result<(), create::CreateSubcommandError> {
    let action = CreateAction::Environment {
        env_file: config_path.to_path_buf(),
    };
    create::handle_create_command(action, working_dir)
}

#[test]
fn it_should_create_environment_from_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = fixtures::create_valid_config(temp_dir.path(), "integration-test-env");

    let result = handle_environment_creation(&config_path, temp_dir.path());

    assert!(
        result.is_ok(),
        "Should create environment successfully: {:?}",
        result.err()
    );

    // Verify environment state file was created by repository
    // Repository creates: <base_dir>/{env-name}/environment.json
    let env_state_file = temp_dir
        .path()
        .join("integration-test-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state file should be created at: {}",
        env_state_file.display()
    );
}

#[test]
fn it_should_reject_nonexistent_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent.json");

    let result = handle_environment_creation(&nonexistent_path, temp_dir.path());

    assert!(result.is_err(), "Should fail for nonexistent file");
    match result.unwrap_err() {
        create::CreateSubcommandError::ConfigFileNotFound { path } => {
            assert_eq!(path, nonexistent_path);
        }
        other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
    }
}

#[test]
fn it_should_reject_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = fixtures::create_invalid_json_config(temp_dir.path());

    let result = handle_environment_creation(&config_path, temp_dir.path());

    assert!(result.is_err(), "Should fail for invalid JSON");
    match result.unwrap_err() {
        create::CreateSubcommandError::ConfigParsingFailed { path, .. } => {
            assert_eq!(path, config_path);
        }
        other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_reject_invalid_environment_name() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = fixtures::create_config_with_invalid_name(temp_dir.path());

    let result = handle_environment_creation(&config_path, temp_dir.path());

    assert!(result.is_err(), "Should fail for invalid environment name");
    match result.unwrap_err() {
        create::CreateSubcommandError::ConfigValidationFailed(_) => {
            // Expected
        }
        other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_reject_missing_ssh_keys() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = fixtures::create_config_with_missing_keys(temp_dir.path());

    let result = handle_environment_creation(&config_path, temp_dir.path());

    assert!(result.is_err(), "Should fail for missing SSH keys");
    match result.unwrap_err() {
        create::CreateSubcommandError::ConfigValidationFailed(_) => {
            // Expected
        }
        other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_reject_duplicate_environment() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = fixtures::create_valid_config(temp_dir.path(), "duplicate-test-env");

    // Create environment first time
    let result1 = handle_environment_creation(&config_path, temp_dir.path());
    assert!(result1.is_ok(), "First create should succeed");

    // Try to create same environment again
    let result2 = handle_environment_creation(&config_path, temp_dir.path());
    assert!(result2.is_err(), "Second create should fail");

    match result2.unwrap_err() {
        create::CreateSubcommandError::CommandFailed(_) => {
            // Expected - environment already exists
        }
        other => panic!("Expected CommandFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_create_environment_in_custom_working_dir() {
    let temp_dir = TempDir::new().unwrap();
    let custom_working_dir = temp_dir.path().join("custom");
    std::fs::create_dir(&custom_working_dir).unwrap();

    let config_path = fixtures::create_valid_config(temp_dir.path(), "custom-dir-env");

    let result = handle_environment_creation(&config_path, &custom_working_dir);

    assert!(result.is_ok(), "Should create in custom working dir");

    // Verify environment was created in custom location
    // Repository creates: <base_dir>/{env-name}/environment.json
    let env_state_file = custom_working_dir.join("custom-dir-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state should be in custom working directory: {}",
        env_state_file.display()
    );
}

#[test]
fn it_should_provide_help_for_all_error_types() {
    let temp_dir = TempDir::new().unwrap();

    // Test ConfigFileNotFound
    let nonexistent = temp_dir.path().join("nonexistent.json");
    if let Err(e) = handle_environment_creation(&nonexistent, temp_dir.path()) {
        let help = e.help();
        assert!(!help.is_empty());
        assert!(help.contains("File Not Found") || help.contains("Check that the file path"));
    }

    // Test ConfigParsingFailed
    let invalid_json = fixtures::create_invalid_json_config(temp_dir.path());
    if let Err(e) = handle_environment_creation(&invalid_json, temp_dir.path()) {
        let help = e.help();
        assert!(!help.is_empty());
        assert!(help.contains("JSON") || help.contains("syntax"));
    }

    // Test ConfigValidationFailed
    let invalid_name = fixtures::create_config_with_invalid_name(temp_dir.path());
    if let Err(e) = handle_environment_creation(&invalid_name, temp_dir.path()) {
        let help = e.help();
        assert!(!help.is_empty());
        // Should delegate to config error help
    }
}
