//! Command execution utilities with error handling and logging
//!
//! This module provides utilities for executing external commands with proper error handling,
//! logging, and output capture. It supports both verbose and quiet execution modes and provides
//! structured error types for different failure scenarios.
//!
//! ## Key Features
//!
//! - Structured error handling with detailed context
//! - Optional verbose output logging
//! - Working directory support
//! - Comprehensive error categorization (startup vs execution failures)

use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;
use tracing::info;

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

/// A command executor that can run shell commands
pub struct CommandExecutor {}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExecutor {
    /// Creates a new `CommandExecutor`
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Runs a command with the given arguments and optional working directory
    ///
    /// # Arguments
    /// * `cmd` - The command to execute
    /// * `args` - Arguments to pass to the command
    /// * `working_dir` - Optional working directory to run the command in
    ///
    /// # Returns
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - A specific error describing what went wrong
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The command cannot be started (e.g., command not found) - `CommandError::StartupFailed`
    /// * The command execution fails with a non-zero exit code - `CommandError::ExecutionFailed`
    pub fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<String, CommandError> {
        let mut command = Command::new(cmd);
        let command_display = format!("{} {}", cmd, args.join(" "));

        command.args(args);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        info!(
            operation = "command_execution",
            command = %command_display,
            "Running command"
        );
        if let Some(dir) = working_dir {
            info!(
                operation = "command_execution",
                working_directory = %dir.display(),
                "Working directory set"
            );
        }

        let output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|source| CommandError::StartupFailed {
                command: command_display.clone(),
                source,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let exit_code = output
                .status
                .code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string());

            return Err(CommandError::ExecutionFailed {
                command: command_display,
                exit_code,
                stdout,
                stderr,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn it_should_execute_simple_command_successfully() {
        let executor = CommandExecutor::new();
        let result = executor.run_command("echo", &["hello"], None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn it_should_respect_working_directory() {
        let executor = CommandExecutor::new();
        let temp_dir = env::temp_dir();
        let result = executor.run_command("pwd", &[], Some(&temp_dir));

        assert!(result.is_ok());
        // The output should contain the temp directory path
        assert!(result
            .unwrap()
            .contains(temp_dir.to_string_lossy().as_ref()));
    }

    #[test]
    fn it_should_return_error_for_nonexistent_command() {
        let executor = CommandExecutor::new();
        let result = executor.run_command("nonexistent_command_xyz123", &[], None);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_return_error_for_failing_command() {
        let executor = CommandExecutor::new();
        let result = executor.run_command("false", &[], None);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("failed with exit code"));
    }

    #[test]
    fn it_should_use_tracing_for_logging() {
        // This test verifies that the command executor uses tracing for logging
        // We can't easily test the tracing output in unit tests without a subscriber
        // but we can verify the executor runs correctly and uses tracing internally
        let executor = CommandExecutor::new();
        let result = executor.run_command("echo", &["tracing_test"], None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "tracing_test");
    }
}
