//! Environment Creation Tests
//!
//! This module tests the environment creation workflow including
//! configuration loading, validation, and command execution.

use crate::bootstrap::Container;
use crate::presentation::controllers::create;
use crate::presentation::controllers::tests::{
    create_config_with_invalid_name, create_config_with_missing_keys, create_invalid_json_config,
    create_valid_config, TestContext,
};
use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::input::cli::CreateAction;
use crate::presentation::views::VerbosityLevel;

/// Helper function to call the environment creation handler
async fn handle_environment_creation(
    config_path: &std::path::Path,
    working_dir: &std::path::Path,
) -> Result<(), create::CreateCommandError> {
    let action = CreateAction::Environment {
        env_file: config_path.to_path_buf(),
    };
    let container = Container::new(VerbosityLevel::Silent, working_dir);
    let context = ExecutionContext::new(std::sync::Arc::new(container));
    create::route_command(action, working_dir, &context).await
}

#[tokio::test]
async fn it_should_create_environment_from_valid_config() {
    let context = TestContext::new();
    let config_path = create_valid_config(context.working_dir(), "integration-test-env");

    let result = handle_environment_creation(&config_path, context.working_dir()).await;

    assert!(
        result.is_ok(),
        "Should create environment successfully: {:?}",
        result.err()
    );

    // Verify environment state file was created by repository
    // Repository creates: <base_dir>/data/{env-name}/environment.json
    let env_state_file = context
        .working_dir()
        .join("data/integration-test-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state file should be created at: {}",
        env_state_file.display()
    );
}

#[tokio::test]
async fn it_should_reject_nonexistent_config_file() {
    let context = TestContext::new();
    let nonexistent_path = context.working_dir().join("nonexistent.json");

    let result = handle_environment_creation(&nonexistent_path, context.working_dir()).await;

    assert!(result.is_err(), "Should fail for nonexistent file");
    match result.unwrap_err() {
        create::CreateCommandError::Environment(
            create::CreateEnvironmentCommandError::ConfigFileNotFound { path },
        ) => {
            assert_eq!(path, nonexistent_path);
        }
        other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn it_should_reject_invalid_json() {
    let context = TestContext::new();
    let config_path = create_invalid_json_config(context.working_dir());

    let result = handle_environment_creation(&config_path, context.working_dir()).await;

    assert!(result.is_err(), "Should fail for invalid JSON");
    match result.unwrap_err() {
        create::CreateCommandError::Environment(
            create::CreateEnvironmentCommandError::ConfigParsingFailed { path, .. },
        ) => {
            assert_eq!(path, config_path);
        }
        other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn it_should_reject_invalid_environment_name() {
    let context = TestContext::new();
    let config_path = create_config_with_invalid_name(context.working_dir());

    let result = handle_environment_creation(&config_path, context.working_dir()).await;

    assert!(result.is_err(), "Should fail for invalid environment name");
    match result.unwrap_err() {
        create::CreateCommandError::Environment(
            create::CreateEnvironmentCommandError::ConfigValidationFailed { .. },
        ) => {
            // Expected
        }
        other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
    }
}

// Note: SSH key file existence validation removed - checked at runtime instead
#[tokio::test]
#[ignore = "SSH key file existence no longer validated during config parsing"]
async fn it_should_reject_missing_ssh_keys() {
    let context = TestContext::new();
    let config_path = create_config_with_missing_keys(context.working_dir());

    let result = handle_environment_creation(&config_path, context.working_dir()).await;

    assert!(result.is_err(), "Should fail for missing SSH keys");
    match result.unwrap_err() {
        create::CreateCommandError::Environment(
            create::CreateEnvironmentCommandError::ConfigValidationFailed { .. },
        ) => {
            // Expected
        }
        other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn it_should_reject_duplicate_environment() {
    let context = TestContext::new();
    let config_path = create_valid_config(context.working_dir(), "duplicate-test-env");

    // Create environment first time
    let result1 = handle_environment_creation(&config_path, context.working_dir()).await;
    assert!(result1.is_ok(), "First create should succeed");

    // Try to create same environment again
    let result2 = handle_environment_creation(&config_path, context.working_dir()).await;
    assert!(result2.is_err(), "Second create should fail");

    match result2.unwrap_err() {
        create::CreateCommandError::Environment(
            create::CreateEnvironmentCommandError::CommandFailed { .. },
        ) => {
            // Expected - environment already exists
        }
        other => panic!("Expected CommandFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn it_should_create_environment_in_custom_working_dir() {
    let context = TestContext::new();
    let custom_working_dir = context.working_dir().join("custom");
    std::fs::create_dir(&custom_working_dir).unwrap();

    let config_path = create_valid_config(context.working_dir(), "custom-dir-env");

    let result = handle_environment_creation(&config_path, &custom_working_dir).await;

    assert!(result.is_ok(), "Should create in custom working dir");

    // Verify environment was created in custom location
    // Repository creates: <base_dir>/data/{env-name}/environment.json
    let env_state_file = custom_working_dir.join("data/custom-dir-env/environment.json");
    assert!(
        env_state_file.exists(),
        "Environment state should be in custom working directory: {}",
        env_state_file.display()
    );
}

#[tokio::test]
async fn it_should_provide_help_for_all_error_types() {
    let context = TestContext::new();

    // Test ConfigFileNotFound
    let nonexistent = context.working_dir().join("nonexistent.json");
    if let Err(e) = handle_environment_creation(&nonexistent, context.working_dir()).await {
        let help = e.help();
        assert!(!help.is_empty());
        assert!(help.contains("File Not Found") || help.contains("Check that the file path"));
    }

    // Test ConfigParsingFailed
    let invalid_json = create_invalid_json_config(context.working_dir());
    if let Err(e) = handle_environment_creation(&invalid_json, context.working_dir()).await {
        let help = e.help();
        assert!(!help.is_empty());
        assert!(help.contains("JSON") || help.contains("syntax"));
    }

    // Test ConfigValidationFailed
    let invalid_name = create_config_with_invalid_name(context.working_dir());
    if let Err(e) = handle_environment_creation(&invalid_name, context.working_dir()).await {
        let help = e.help();
        assert!(!help.is_empty());
        // Should delegate to config error help
    }
}
