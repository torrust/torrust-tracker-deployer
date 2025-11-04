//! Command output representation for container execution results.
//!
//! This module provides a type to represent the output from executing commands
//! in Docker containers, capturing both stdout and stderr streams.

use std::fmt;

/// Output from executing a command in a container.
///
/// This type captures both stdout and stderr streams separately, allowing
/// tests to inspect either stream individually or combined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Standard output stream
    stdout: String,
    /// Standard error stream
    stderr: String,
}

impl CommandOutput {
    /// Create a new command output from stdout and stderr strings.
    pub fn new(stdout: String, stderr: String) -> Self {
        Self { stdout, stderr }
    }

    /// Get the stdout content.
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Get the stderr content.
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Get both stdout and stderr combined.
    ///
    /// Returns stderr followed by stdout, matching the typical order
    /// where logs (stderr) appear before user output (stdout).
    pub fn combined(&self) -> String {
        format!("{}{}", self.stderr, self.stdout)
    }

    /// Check if either stdout or stderr contains the given string.
    ///
    /// This is useful in tests where you don't care which stream
    /// contains the expected output.
    pub fn contains(&self, needle: &str) -> bool {
        self.stdout.contains(needle) || self.stderr.contains(needle)
    }
}

impl fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.combined())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_command_output_with_separate_stdout_and_stderr() {
        let output = CommandOutput::new("hello".to_string(), "error".to_string());
        assert_eq!(output.stdout(), "hello");
        assert_eq!(output.stderr(), "error");
    }

    #[test]
    fn it_should_combine_stderr_and_stdout_with_stderr_first() {
        let output =
            CommandOutput::new("stdout content".to_string(), "stderr content\n".to_string());
        assert_eq!(output.combined(), "stderr content\nstdout content");
    }

    #[test]
    fn it_should_find_text_in_stdout_when_checking_contains() {
        let output = CommandOutput::new("hello world".to_string(), String::new());
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(!output.contains("missing"));
    }

    #[test]
    fn it_should_find_text_in_stderr_when_checking_contains() {
        let output = CommandOutput::new(String::new(), "error message".to_string());
        assert!(output.contains("error"));
        assert!(output.contains("message"));
        assert!(!output.contains("missing"));
    }

    #[test]
    fn it_should_find_text_in_either_stream_when_checking_contains() {
        let output = CommandOutput::new("stdout text".to_string(), "stderr text".to_string());
        assert!(output.contains("stdout"));
        assert!(output.contains("stderr"));
        assert!(output.contains("text"));
    }
}
