use std::process::Command;

use crate::errors::CommandError;

/// Check if a command exists in the system PATH
///
/// # Examples
///
/// ```rust
/// use torrust_dependency_installer::command::command_exists;
///
/// // Check if 'cargo' is installed
/// let exists = command_exists("cargo").unwrap();
/// assert!(exists);
/// ```
///
/// # Errors
///
/// Returns an error if the 'which' command fails to execute
pub fn command_exists(command: &str) -> Result<bool, CommandError> {
    // Use 'which' on Unix-like systems to check if command exists
    let output =
        Command::new("which")
            .arg(command)
            .output()
            .map_err(|e| CommandError::ExecutionFailed {
                command: format!("which {command}"),
                source: e,
            })?;

    Ok(output.status.success())
}

/// Execute a command and return its stdout as a string
///
/// # Examples
///
/// ```rust,no_run
/// use torrust_dependency_installer::command::execute_command;
///
/// // Get cargo version
/// let version = execute_command("cargo", &["--version"]).unwrap();
/// println!("Cargo version: {}", version);
/// ```
///
/// # Errors
///
/// Returns an error if the command is not found or fails to execute
pub fn execute_command(command: &str, args: &[&str]) -> Result<String, CommandError> {
    let output = Command::new(command).args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CommandError::CommandNotFound {
                command: command.to_string(),
            }
        } else {
            CommandError::ExecutionFailed {
                command: format!("{command} {}", args.join(" ")),
                source: e,
            }
        }
    })?;

    if !output.status.success() {
        return Err(CommandError::ExecutionFailed {
            command: format!("{command} {}", args.join(" ")),
            source: std::io::Error::other(format!("Command exited with status: {}", output.status)),
        });
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| CommandError::ExecutionFailed {
            command: format!("{command} {}", args.join(" ")),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })
}
