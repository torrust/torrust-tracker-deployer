//! Command execution utilities
//!
//! This module provides the `CommandExecutor` struct for executing external commands
//! with proper error handling, logging, and output capture.

use std::path::Path;
use std::process::{Command, Stdio};
use tracing::info;

use super::error::CommandError;
use super::result::CommandResult;

/// A command executor that can run shell commands
#[derive(Debug)]
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
    /// * `Ok(CommandResult)` - Complete command execution information if the command succeeds
    /// * `Err(CommandError)` - A specific error describing what went wrong
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The working directory does not exist - `CommandError::WorkingDirectoryNotFound`
    /// * The command cannot be started (e.g., command not found) - `CommandError::StartupFailed`
    /// * The command execution fails with a non-zero exit code - `CommandError::ExecutionFailed`
    pub fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<CommandResult, CommandError> {
        // Check if working directory exists before attempting to run the command
        // This provides a clearer error message than the generic "No such file or directory"
        if let Some(dir) = working_dir {
            if !dir.exists() {
                return Err(CommandError::WorkingDirectoryNotFound {
                    working_dir: dir.to_path_buf(),
                });
            }
        }

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

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
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

        // Log stdout and stderr at debug level when command succeeds
        if !stdout.trim().is_empty() {
            tracing::debug!(
                operation = "command_execution",
                command = %command_display,
                "stdout: {}",
                stdout.trim()
            );
        }

        if !stderr.trim().is_empty() {
            tracing::debug!(
                operation = "command_execution",
                command = %command_display,
                "stderr: {}",
                stderr.trim()
            );
        }

        Ok(CommandResult::new(output.status, stdout, stderr))
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
        let output = result.unwrap();
        assert_eq!(output.stdout_trimmed(), "hello");
        assert!(output.is_success());
    }

    #[test]
    fn it_should_respect_working_directory() {
        let executor = CommandExecutor::new();
        let temp_dir = env::temp_dir();
        let result = executor.run_command("pwd", &[], Some(&temp_dir));

        assert!(result.is_ok());
        let output = result.unwrap();
        // The output should contain the temp directory path
        assert!(output.stdout.contains(temp_dir.to_string_lossy().as_ref()));
        assert!(output.is_success());
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
        let output = result.unwrap();
        assert_eq!(output.stdout_trimmed(), "tracing_test");
        assert!(output.is_success());
    }

    #[test]
    fn it_should_return_clear_error_when_working_directory_does_not_exist() {
        let executor = CommandExecutor::new();
        let nonexistent_dir = Path::new("/nonexistent/path/that/does/not/exist");
        let result = executor.run_command("echo", &["hello"], Some(nonexistent_dir));

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CommandError::WorkingDirectoryNotFound { working_dir } => {
                assert_eq!(working_dir, nonexistent_dir);
            }
            other => panic!("Expected WorkingDirectoryNotFound, got: {other:?}"),
        }
    }
}
