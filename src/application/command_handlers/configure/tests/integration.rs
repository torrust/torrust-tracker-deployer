//! Integration tests for Configure Command
//!
//! This module contains integration tests for the `ConfigureCommandHandler`.

use std::sync::Arc;

use chrono::{TimeZone, Utc};

use super::builders::{create_test_environment, ConfigureCommandHandlerTestBuilder};
use crate::application::command_handlers::configure::ConfigureCommandHandlerError;
use crate::domain::environment::state::ConfigureStep;
use crate::shared::command::CommandError;

#[test]
fn it_should_create_configure_command_handler_with_all_dependencies() {
    let (command_handler, _temp_dir) = ConfigureCommandHandlerTestBuilder::new().build();

    // Verify the command handler was created (basic structure test)
    // This test just verifies that the command handler can be created with the dependencies
    assert_eq!(Arc::strong_count(&command_handler.ansible_client), 1);
}

#[test]
fn it_should_have_correct_error_type_conversions() {
    // Test that all error types can convert to ConfigureCommandHandlerError
    let command_error = CommandError::StartupFailed {
        command: "test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    let configure_error: ConfigureCommandHandlerError = command_error.into();
    drop(configure_error);
}

#[test]
fn it_should_build_failure_context_from_command_error() {
    let (command, temp_dir) = ConfigureCommandHandlerTestBuilder::new().build();

    // Create test environment for trace generation
    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

    let error = ConfigureCommandHandlerError::Command(CommandError::ExecutionFailed {
        command: "test".to_string(),
        exit_code: "1".to_string(),
        stdout: String::new(),
        stderr: "test error".to_string(),
    });

    let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let current_step = ConfigureStep::InstallDocker;
    let context = command.build_failure_context(&environment, &error, current_step, started_at);
    assert_eq!(context.failed_step, ConfigureStep::InstallDocker);
    assert_eq!(
        context.error_kind,
        crate::shared::ErrorKind::CommandExecution
    );
    assert_eq!(context.base.execution_started_at, started_at);
}
