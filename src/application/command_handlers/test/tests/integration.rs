//! Integration tests for `TestCommandHandler`

use crate::infrastructure::remote_actions::RemoteActionError;
use crate::shared::command::CommandError;

use super::super::*;

#[test]
fn it_should_have_correct_error_type_conversions() {
    // Test that all error types can convert to TestCommandHandlerError
    let command_error = CommandError::StartupFailed {
        command: "test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    let test_error: TestCommandHandlerError = command_error.into();
    drop(test_error);

    let remote_action_error = RemoteActionError::ValidationFailed {
        action_name: "test".to_string(),
        message: "test error".to_string(),
    };
    let test_error: TestCommandHandlerError = remote_action_error.into();
    drop(test_error);
}
