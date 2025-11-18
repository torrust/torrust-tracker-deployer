//! Integration tests for Configure Command
//!
//! This module contains integration tests for the `ConfigureCommandHandler`.

use std::sync::Arc;

use super::builders::ConfigureCommandHandlerTestBuilder;
use crate::application::command_handlers::configure::ConfigureCommandHandlerError;
use crate::shared::command::CommandError;

#[test]
fn it_should_create_configure_command_handler_with_all_dependencies() {
    let (command_handler, _temp_dir) = ConfigureCommandHandlerTestBuilder::new().build();

    // Verify the command handler was created (basic structure test)
    // This test just verifies that the command handler can be created with the dependencies
    assert_eq!(Arc::strong_count(&command_handler.clock), 1);
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
