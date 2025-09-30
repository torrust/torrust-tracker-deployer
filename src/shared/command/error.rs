//! Command execution error types
//!
//! This module provides error types for command execution failures,
//! including startup errors and execution errors with detailed context.

use thiserror::Error;

/// Errors that can occur during command execution
#[derive(Error, Debug)]
pub enum CommandError {
    /// The command could not be started (e.g., command not found, permission denied)
    #[error("Failed to start command '{command}': {source}")]
    StartupFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// The command was started but exited with a non-zero status code
    #[error(
        "Command '{command}' failed with exit code {exit_code}\nStdout: {stdout}\nStderr: {stderr}"
    )]
    ExecutionFailed {
        command: String,
        exit_code: String,
        stdout: String,
        stderr: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn it_should_format_startup_failed_error_correctly() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let error = CommandError::StartupFailed {
            command: "nonexistent_command".to_string(),
            source: io_error,
        };

        let error_message = error.to_string();
        assert!(error_message.contains("Failed to start command 'nonexistent_command'"));
        assert!(error_message.contains("command not found"));
    }

    #[test]
    fn it_should_format_execution_failed_error_correctly() {
        let error = CommandError::ExecutionFailed {
            command: "false".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "command failed".to_string(),
        };

        let error_message = error.to_string();
        assert!(error_message.contains("Command 'false' failed with exit code 1"));
        assert!(error_message.contains("Stderr: command failed"));
    }

    #[test]
    fn it_should_preserve_source_error_chain() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error = CommandError::StartupFailed {
            command: "restricted_command".to_string(),
            source: io_error,
        };

        // Test that the source error is preserved
        assert!(error.source().is_some());
        assert_eq!(error.source().unwrap().to_string(), "permission denied");
    }
}
