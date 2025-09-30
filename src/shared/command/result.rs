//! Command execution result
//!
//! This module provides the `CommandResult` struct that represents the complete
//! result of a successfully executed command, including exit status, stdout, and stderr.

use std::process::ExitStatus;

/// Represents the complete result of a successfully executed command
///
/// This struct provides complete information about a command execution,
/// including the exit status and both stdout and stderr output streams.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// The exit status of the command
    pub exit_status: ExitStatus,

    /// The standard output (stdout) of the command
    pub stdout: String,

    /// The standard error output (stderr) of the command  
    pub stderr: String,
}

impl CommandResult {
    /// Creates a new `CommandResult` instance
    #[must_use]
    pub fn new(exit_status: ExitStatus, stdout: String, stderr: String) -> Self {
        Self {
            exit_status,
            stdout,
            stderr,
        }
    }

    /// Returns true if the command executed successfully (exit code 0)
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.exit_status.success()
    }

    /// Returns the exit code if available
    #[must_use]
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_status.code()
    }

    /// Returns the stdout output, trimmed of leading/trailing whitespace
    #[must_use]
    pub fn stdout_trimmed(&self) -> &str {
        self.stdout.trim()
    }

    /// Returns the stderr output, trimmed of leading/trailing whitespace
    #[must_use]
    pub fn stderr_trimmed(&self) -> &str {
        self.stderr.trim()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn it_should_provide_complete_command_result_information() {
        // Create a simple command to get real ExitStatus
        let output = Command::new("echo")
            .arg("test")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute echo command");

        let command_result = CommandResult::new(
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        );

        // Test that we have access to all execution information
        assert!(command_result.is_success());
        assert_eq!(command_result.exit_code(), Some(0));
        assert_eq!(command_result.stdout_trimmed(), "test");
        assert_eq!(command_result.stderr_trimmed(), "");

        // Test that raw outputs are also available
        assert!(command_result.stdout.contains("test"));
        assert!(command_result.stderr.is_empty());
    }

    #[test]
    fn it_should_handle_command_with_stderr() {
        // Use a command that writes to stderr (ls on a non-existent directory)
        let output = Command::new("ls")
            .arg("/nonexistent_directory_xyz123")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute ls command");

        let command_result = CommandResult::new(
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        );

        // This command should fail
        assert!(!command_result.is_success());
        assert!(command_result.exit_code().is_some());
        assert_ne!(command_result.exit_code(), Some(0));

        // Should have error in stderr
        assert!(!command_result.stderr_trimmed().is_empty());
    }

    #[test]
    fn it_should_trim_whitespace_correctly() {
        // Create a command result with whitespace
        let output = Command::new("echo")
            .arg("  spaced  ")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute echo command");

        let command_result = CommandResult::new(
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        );

        // Test trimming
        assert_eq!(command_result.stdout_trimmed(), "spaced");
        // Raw stdout should preserve whitespace
        assert!(command_result.stdout.contains("  spaced  "));
    }
}
