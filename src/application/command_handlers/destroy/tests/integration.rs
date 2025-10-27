//! Integration tests for Destroy Command
//!
//! This module contains integration tests for the `DestroyCommandHandler`.

use std::sync::Arc;

use super::builders::DestroyCommandHandlerTestBuilder;
use crate::adapters::tofu::client::OpenTofuError;
use crate::application::command_handlers::destroy::{
    DestroyCommandHandler, DestroyCommandHandlerError,
};
use crate::shared::command::CommandError;

#[test]
fn it_should_create_destroy_command_handler_with_all_dependencies() {
    let (command_handler, _temp_dir) = DestroyCommandHandlerTestBuilder::new().build();

    // Verify the command handler was created (basic structure test)
    // This test just verifies that the command handler can be created with the dependencies
    assert_eq!(Arc::strong_count(&command_handler.repository), 1);
}

#[test]
fn it_should_have_correct_error_type_conversions() {
    // Test that all error types can convert to DestroyCommandHandlerError
    let command_error = CommandError::StartupFailed {
        command: "test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    let opentofu_error = OpenTofuError::CommandError(command_error);
    let destroy_error: DestroyCommandHandlerError = opentofu_error.into();
    drop(destroy_error);

    let command_error_direct = CommandError::ExecutionFailed {
        command: "test".to_string(),
        exit_code: "1".to_string(),
        stdout: String::new(),
        stderr: "test error".to_string(),
    };
    let destroy_error: DestroyCommandHandlerError = command_error_direct.into();
    drop(destroy_error);
}

#[test]
fn it_should_skip_infrastructure_destruction_when_tofu_build_dir_does_not_exist() {
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    // Arrange: Create environment in Created state with no OpenTofu build directory
    let (created_env, _data_dir, _build_dir, _temp_dir) =
        EnvironmentTestBuilder::new().build_with_custom_paths();

    // Transition to Destroying state
    let destroying_env = created_env.start_destroying();

    // Verify tofu_build_dir does not exist
    assert!(
        !destroying_env.tofu_build_dir().exists(),
        "OpenTofu build directory should not exist for Created state"
    );

    // Act: Check if infrastructure should be destroyed
    let should_destroy = DestroyCommandHandler::should_destroy_infrastructure(&destroying_env);

    // Assert: Infrastructure destruction should be skipped
    assert!(
        !should_destroy,
        "Infrastructure destruction should be skipped when tofu_build_dir does not exist"
    );
}

#[test]
fn it_should_attempt_infrastructure_destruction_when_tofu_build_dir_exists() {
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    // Arrange: Create environment with OpenTofu build directory
    let (created_env, _data_dir, _build_dir, _temp_dir) =
        EnvironmentTestBuilder::new().build_with_custom_paths();

    // Create the OpenTofu build directory to simulate provisioned state
    let tofu_build_dir = created_env.tofu_build_dir();
    std::fs::create_dir_all(&tofu_build_dir).expect("Failed to create tofu build dir");

    // Transition to Destroying state
    let destroying_env = created_env.start_destroying();

    // Verify tofu_build_dir exists
    assert!(
        destroying_env.tofu_build_dir().exists(),
        "OpenTofu build directory should exist for provisioned environment"
    );

    // Act: Check if infrastructure should be destroyed
    let should_destroy = DestroyCommandHandler::should_destroy_infrastructure(&destroying_env);

    // Assert: Infrastructure destruction should be attempted
    assert!(
        should_destroy,
        "Infrastructure destruction should be attempted when tofu_build_dir exists"
    );
}

#[test]
fn it_should_clean_up_state_files_regardless_of_infrastructure_state() {
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    // Arrange: Create environment with data and build directories
    let (created_env, data_dir, build_dir, _temp_dir) =
        EnvironmentTestBuilder::new().build_with_custom_paths();

    // Create the directories
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build dir");

    // Create some files in the directories
    std::fs::write(data_dir.join("environment.json"), "{}").expect("Failed to write file");
    std::fs::write(build_dir.join("test.txt"), "test").expect("Failed to write file");

    // Verify directories exist before cleanup
    assert!(data_dir.exists(), "Data directory should exist");
    assert!(build_dir.exists(), "Build directory should exist");

    // Act: Clean up state files
    let result = DestroyCommandHandler::cleanup_state_files(&created_env);

    // Assert: Cleanup succeeded
    assert!(
        result.is_ok(),
        "State file cleanup should succeed: {:?}",
        result.err()
    );

    // Assert: Directories were removed
    assert!(
        !data_dir.exists(),
        "Data directory should be removed after cleanup"
    );
    assert!(
        !build_dir.exists(),
        "Build directory should be removed after cleanup"
    );
}
