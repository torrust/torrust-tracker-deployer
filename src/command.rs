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

/// A command executor that can run shell commands with optional verbosity
pub struct CommandExecutor {
    verbose: bool,
}

impl CommandExecutor {
    /// Creates a new `CommandExecutor`
    #[must_use]
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
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

        if self.verbose {
            info!("🔧 Running: {}", command_display);
            if let Some(dir) = working_dir {
                info!("   Working directory: {}", dir.display());
            }
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
        let executor = CommandExecutor::new(false);
        let result = executor.run_command("echo", &["hello"], None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn it_should_respect_working_directory() {
        let executor = CommandExecutor::new(false);
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
        let executor = CommandExecutor::new(false);
        let result = executor.run_command("nonexistent_command_xyz123", &[], None);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_return_error_for_failing_command() {
        let executor = CommandExecutor::new(false);
        let result = executor.run_command("false", &[], None);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("failed with exit code"));
    }

    #[test]
    fn it_should_use_verbose_logging_when_enabled() {
        // This test verifies that verbose mode uses tracing instead of println
        // We can't easily test the tracing output in unit tests without a subscriber
        // but we can verify the executor runs correctly in verbose mode
        let executor = CommandExecutor::new(true);
        let result = executor.run_command("echo", &["verbose_test"], None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "verbose_test");
    }
}
