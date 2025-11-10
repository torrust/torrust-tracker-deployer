//! Tests for Environment Creation Handler
//!
//! This module contains comprehensive tests for the environment creation
//! functionality, including integration tests and unit tests for helper functions.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tempfile::TempDir;

use super::config_loader::ConfigLoader;
use super::errors::CreateSubcommandError;
use super::handler::{execute_create_command, handle_environment_creation, load_configuration};
use crate::bootstrap::Container;
use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::progress::ProgressReporter;
use crate::presentation::user_output::test_support::TestUserOutput;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

use crate::application::command_handlers::CreateCommandHandler;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::clock::SystemClock;

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
    let result = handle_environment_creation(&config_path, working_dir, &context);

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

    let result = handle_environment_creation(&config_path, working_dir, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        CreateSubcommandError::ConfigFileNotFound { path } => {
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
    let result = handle_environment_creation(&config_path, working_dir, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        CreateSubcommandError::ConfigParsingFailed { .. } => {
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
    let result1 = handle_environment_creation(&config_path, working_dir, &context);
    assert!(result1.is_ok(), "First create should succeed");

    // Try to create same environment again (use new context to avoid any state issues)
    let user_output2 = TestUserOutput::wrapped(VerbosityLevel::Normal);
    let context2 = create_test_context(working_dir, user_output2);
    let result2 = handle_environment_creation(&config_path, working_dir, &context2);
    assert!(result2.is_err(), "Second create should fail");

    match result2.unwrap_err() {
        CreateSubcommandError::CommandFailed { .. } => {
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
    let result = handle_environment_creation(&config_path, &custom_working_dir, &context);

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
// UNIT TESTS - Helper Functions
// ============================================================================

mod load_configuration_tests {
    use super::*;

    #[test]
    fn it_should_load_valid_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-load-config"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(user_output, 5);
        let result = load_configuration(&mut progress, &config_path);

        assert!(result.is_ok(), "Should load valid configuration");
        let config = result.unwrap();
        assert_eq!(config.environment.name, "test-load-config");
    }

    #[test]
    fn it_should_return_error_for_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("missing.json");

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(user_output, 5);
        let result = load_configuration(&mut progress, &config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigFileNotFound { path } => {
                assert_eq!(path, config_path);
            }
            other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");
        fs::write(&config_path, r#"{"broken json"#).unwrap();

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(user_output, 5);
        let result = load_configuration(&mut progress, &config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigParsingFailed { .. } => {
                // Expected
            }
            other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
        }
    }
}

mod execute_create_command_tests {
    use super::*;

    #[test]
    fn it_should_execute_command_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-execute"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let _context = create_test_context(temp_dir.path(), user_output.clone());
        let loader = ConfigLoader;
        let config = loader.load_from_file(&config_path).unwrap();

        // Create command handler using manual dependency creation
        // TODO: Once Container is expanded, get these from context.container()
        let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
        let repository = repository_factory.create(temp_dir.path().to_path_buf());
        let clock = Arc::new(SystemClock);
        let command_handler = CreateCommandHandler::new(repository, clock);

        // Create ProgressReporter for the function call (use 5 total steps)
        let mut progress = ProgressReporter::new(user_output, 5);
        let result = execute_create_command(&mut progress, &command_handler, config);

        assert!(result.is_ok(), "Should execute command successfully");
        let environment = result.unwrap();
        assert_eq!(environment.name().as_str(), "test-execute");
    }

    #[test]
    fn it_should_return_error_for_duplicate_environment() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-duplicate"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let _context = create_test_context(temp_dir.path(), user_output.clone());
        let loader = ConfigLoader;
        let config = loader.load_from_file(&config_path).unwrap();

        // Create command handler using manual dependency creation
        // TODO: Once Container is expanded, get these from context.container()
        let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
        let repository = repository_factory.create(temp_dir.path().to_path_buf());
        let clock = Arc::new(SystemClock);
        let command_handler = CreateCommandHandler::new(repository, clock);

        // Create environment first time
        let mut progress1 = ProgressReporter::new(user_output.clone(), 5);
        let result1 = execute_create_command(&mut progress1, &command_handler, config.clone());
        assert!(result1.is_ok(), "First execution should succeed");

        // Try to create same environment again
        let mut progress2 = ProgressReporter::new(user_output, 5);
        let result2 = execute_create_command(&mut progress2, &command_handler, config);
        assert!(result2.is_err(), "Second execution should fail");

        match result2.unwrap_err() {
            CreateSubcommandError::CommandFailed { .. } => {
                // Expected
            }
            other => panic!("Expected CommandFailed, got: {other:?}"),
        }
    }
}
