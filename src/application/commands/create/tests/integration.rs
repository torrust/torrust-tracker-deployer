//! Integration tests for Create Command
//!
//! These tests verify the complete behavior of `CreateCommand` including
//! interaction with the repository and proper error handling.

use tempfile::TempDir;

use crate::application::commands::create::tests::{
    create_valid_test_config, CreateCommandTestBuilder,
};
use crate::application::commands::create::CreateCommandError;
use crate::domain::environment::EnvironmentName;

#[test]
fn it_should_create_environment_with_valid_configuration() {
    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new().build();
    let config = create_valid_test_config(&temp_dir, "test-environment");

    // Act
    let result = command.execute(config);

    // Assert
    assert!(result.is_ok(), "Expected successful environment creation");
    let environment = result.unwrap();
    assert_eq!(environment.name().as_str(), "test-environment");
}

#[test]
fn it_should_fail_when_environment_already_exists() {
    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new()
        .with_existing_environment("test-environment")
        .build();

    let config = create_valid_test_config(&temp_dir, "test-environment");

    // Act
    let result = command.execute(config);

    // Assert
    assert!(result.is_err(), "Expected error for duplicate environment");
    match result.unwrap_err() {
        CreateCommandError::EnvironmentAlreadyExists { name } => {
            assert_eq!(name, "test-environment");
        }
        other => panic!("Expected EnvironmentAlreadyExists error, got: {other:?}"),
    }
}

#[test]
fn it_should_verify_repository_handles_directory_creation() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let (command, builder_temp_dir) = CreateCommandTestBuilder::new()
        .with_base_directory(temp_dir.path())
        .build();

    let config = create_valid_test_config(&builder_temp_dir, "test-environment");

    // Act
    let result = command.execute(config);

    // Assert
    assert!(result.is_ok(), "Expected successful environment creation");
    let environment = result.unwrap();

    // Verify the environment was created with the correct name
    assert_eq!(environment.name().as_str(), "test-environment");

    // The repository is responsible for directory creation during save.
    // We verify this by checking that the environment was persisted successfully,
    // which implies the necessary directories were created.
    let env_name = EnvironmentName::new("test-environment").unwrap();
    let loaded = command
        .environment_repository
        .load(&env_name)
        .expect("Failed to load environment");

    assert!(
        loaded.is_some(),
        "Environment should be persisted in repository"
    );
}

#[test]
fn it_should_persist_environment_state_to_repository() {
    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new().build();
    let config = create_valid_test_config(&temp_dir, "persistent-env");

    // Act
    let result = command.execute(config);

    // Assert creation succeeded
    assert!(result.is_ok(), "Expected successful environment creation");
    let created_environment = result.unwrap();

    // Verify environment was persisted by loading it back
    let env_name = EnvironmentName::new("persistent-env").unwrap();
    let loaded = command
        .environment_repository
        .load(&env_name)
        .expect("Failed to load environment")
        .expect("Environment should exist in repository");

    // Verify loaded environment matches created one
    assert_eq!(loaded.name().as_str(), created_environment.name().as_str());
}

#[test]
fn it_should_fail_with_invalid_environment_name() {
    use crate::domain::config::{
        EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    };
    use std::fs;

    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new().build();

    // Create config with invalid environment name (uppercase not allowed)
    let private_key = temp_dir.path().join("id_rsa");
    let public_key = temp_dir.path().join("id_rsa.pub");
    fs::write(&private_key, "test_private_key").unwrap();
    fs::write(&public_key, "test_public_key").unwrap();

    let config = EnvironmentCreationConfig::new(
        EnvironmentSection {
            name: "Invalid_Name".to_string(), // Invalid: contains uppercase
        },
        SshCredentialsConfig::new(
            private_key.to_string_lossy().to_string(),
            public_key.to_string_lossy().to_string(),
            "torrust".to_string(),
            22,
        ),
    );

    // Act
    let result = command.execute(config);

    // Assert
    assert!(
        result.is_err(),
        "Expected error for invalid environment name"
    );
    match result.unwrap_err() {
        CreateCommandError::InvalidConfiguration(_) => {
            // Expected error type
        }
        other => panic!("Expected InvalidConfiguration error, got: {other:?}"),
    }
}

#[test]
fn it_should_fail_when_ssh_private_key_not_found() {
    use crate::domain::config::{
        EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    };

    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new().build();

    // Create config with non-existent SSH key files
    let config = EnvironmentCreationConfig::new(
        EnvironmentSection {
            name: "test-env".to_string(),
        },
        SshCredentialsConfig::new(
            "/nonexistent/private_key".to_string(),
            temp_dir
                .path()
                .join("id_rsa.pub")
                .to_string_lossy()
                .to_string(),
            "torrust".to_string(),
            22,
        ),
    );

    // Act
    let result = command.execute(config);

    // Assert
    assert!(
        result.is_err(),
        "Expected error for non-existent SSH private key"
    );
    match result.unwrap_err() {
        CreateCommandError::InvalidConfiguration(_) => {
            // Expected error type
        }
        other => panic!("Expected InvalidConfiguration error, got: {other:?}"),
    }
}

#[test]
fn it_should_provide_helpful_error_messages() {
    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new()
        .with_existing_environment("existing-env")
        .build();

    let config = create_valid_test_config(&temp_dir, "existing-env");

    // Act
    let result = command.execute(config);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Verify error has help method
    let help = error.help();
    assert!(!help.is_empty(), "Help text should not be empty");
    assert!(
        help.contains("already exists") || help.contains("Troubleshooting"),
        "Help should contain actionable guidance"
    );
}

#[test]
fn it_should_create_multiple_different_environments() {
    // Arrange
    let (command, temp_dir) = CreateCommandTestBuilder::new().build();

    // Act: Create first environment
    let config1 = create_valid_test_config(&temp_dir, "environment-1");
    let result1 = command.execute(config1);
    assert!(result1.is_ok(), "First environment should be created");

    // Act: Create second environment
    let config2 = create_valid_test_config(&temp_dir, "environment-2");
    let result2 = command.execute(config2);
    assert!(result2.is_ok(), "Second environment should be created");

    // Assert: Both environments exist
    let env1_name = EnvironmentName::new("environment-1").unwrap();
    let env2_name = EnvironmentName::new("environment-2").unwrap();

    assert!(command.environment_repository.exists(&env1_name).unwrap());
    assert!(command.environment_repository.exists(&env2_name).unwrap());
}

#[test]
fn it_should_use_deterministic_timestamps_with_mock_clock() {
    use chrono::TimeZone;

    // Arrange
    let fixed_time = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
    let (command, temp_dir) = CreateCommandTestBuilder::new()
        .with_fixed_time(fixed_time)
        .build();

    let config = create_valid_test_config(&temp_dir, "test-env");

    // Act
    let _result = command.execute(config);

    // Assert: Clock maintains fixed time
    assert_eq!(command.clock.now(), fixed_time);
}
