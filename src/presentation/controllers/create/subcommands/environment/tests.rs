//! Tests for Environment Creation Handler
//!
//! This module contains comprehensive tests for the environment creation
//! functionality, including integration tests and unit tests for helper functions.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use tempfile::TempDir;

use super::errors::CreateEnvironmentCommandError;
use super::handler::handle;
use crate::bootstrap::Container;
use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::user_output::test_support::TestUserOutput;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

fn create_test_context(
    _working_dir: &Path,
    _user_output: Arc<Mutex<UserOutput>>,
) -> ExecutionContext {
    let container = Container::new(VerbosityLevel::Silent);
    ExecutionContext::new(Arc::new(container))
}

#[test]
fn it_should_create_environment_from_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    // Write a valid configuration file
    let config_json = format!(
        r#"{{
        "environment": {{
            "name": "test-create-env"
        }},
        "ssh_credentials": {{
            "private_key_path": "{private_key_path}",
            "public_key_path": "{public_key_path}"
        }}
    }}"#
    );
    fs::write(&config_path, config_json).unwrap();

    let working_dir = temp_dir.path();
    let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context = create_test_context(working_dir, user_output);
    let result = handle(&config_path, working_dir, &context);

    assert!(
        result.is_ok(),
        "Should successfully create environment: {:?}",
        result.err()
    );

    // Verify environment state file was created by repository
    // Repository creates: <base_dir>/{env-name}/environment.json
    let env_state_file = working_dir.join("test-create-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state file should be created at: {}",
        env_state_file.display()
    );
}

#[test]
fn it_should_return_error_for_missing_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.json");
    let working_dir = temp_dir.path();
    let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context = create_test_context(working_dir, user_output);

    let result = handle(&config_path, working_dir, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        CreateEnvironmentCommandError::ConfigFileNotFound { path } => {
            assert_eq!(path, config_path);
        }
        other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
    }
}

#[test]
fn it_should_return_error_for_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    fs::write(&config_path, r#"{"invalid json"#).unwrap();

    let working_dir = temp_dir.path();
    let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context = create_test_context(working_dir, user_output);
    let result = handle(&config_path, working_dir, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        CreateEnvironmentCommandError::ConfigParsingFailed { .. } => {
            // Expected
        }
        other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_return_error_for_duplicate_environment() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    let config_json = format!(
        r#"{{
        "environment": {{
            "name": "duplicate-env"
        }},
        "ssh_credentials": {{
            "private_key_path": "{private_key_path}",
            "public_key_path": "{public_key_path}"
        }}
    }}"#
    );
    fs::write(&config_path, config_json).unwrap();

    let working_dir = temp_dir.path();
    let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context = create_test_context(working_dir, user_output);

    // Create environment first time
    let result1 = handle(&config_path, working_dir, &context);
    assert!(result1.is_ok(), "First create should succeed");

    // Try to create same environment again (use new context to avoid any state issues)
    let user_output2 = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context2 = create_test_context(working_dir, user_output2);
    let result2 = handle(&config_path, working_dir, &context2);
    assert!(result2.is_err(), "Second create should fail");

    match result2.unwrap_err() {
        CreateEnvironmentCommandError::CommandFailed { .. } => {
            // Expected - environment already exists
        }
        other => panic!("Expected CommandFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_create_environment_in_custom_working_dir() {
    let temp_dir = TempDir::new().unwrap();
    let custom_working_dir = temp_dir.path().join("custom");
    fs::create_dir(&custom_working_dir).unwrap();

    let config_path = temp_dir.path().join("config.json");

    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    let config_json = format!(
        r#"{{
        "environment": {{
            "name": "custom-location-env"
        }},
        "ssh_credentials": {{
            "private_key_path": "{private_key_path}",
            "public_key_path": "{public_key_path}"
        }}
    }}"#
    );
    fs::write(&config_path, config_json).unwrap();

    let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context = create_test_context(&custom_working_dir, user_output);
    let result = handle(&config_path, &custom_working_dir, &context);

    assert!(result.is_ok(), "Should create in custom working dir");

    // Verify environment was created in custom location
    // Repository creates: <base_dir>/{env-name}/environment.json
    let env_state_file = custom_working_dir.join("custom-location-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state should be in custom working directory at: {}",
        env_state_file.display()
    );
}

// ============================================================================
// Test Coverage Notes
// ============================================================================
//
// All functionality that was previously tested in commented unit test blocks
// is now covered by:
//
// 1. Configuration loading unit tests in `config_loader.rs` module
// 2. Integration tests above that test the full workflow through the public API
//
// This provides better test organization and coverage:
// - Unit tests are in the appropriate module (config_loader.rs)
// - Integration tests use the public API (more realistic)
// - No redundant tests for internal implementation details
