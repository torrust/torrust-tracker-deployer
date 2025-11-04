//! Running container with an installed binary

use std::path::Path;
use std::process::Command;

use testcontainers::{ContainerAsync, GenericImage};

use super::command_output::CommandOutput;
use super::container_id::ContainerId;

/// Exit code returned by a command execution
pub type ExitCode = i32;

/// A running Ubuntu container with a binary installed and ready to execute
///
/// This struct provides methods for executing commands and managing a running
/// Ubuntu container that has been prepared with a binary. It handles the container
/// lifecycle, ensuring the container stays alive while tests run, and provides
/// convenient methods for command execution and file operations.
pub struct RunningBinaryContainer {
    // Keep a reference to the container so it stays alive
    #[allow(dead_code)]
    container: ContainerAsync<GenericImage>,
    container_id: ContainerId,
}

impl RunningBinaryContainer {
    /// Create a new running binary container
    ///
    /// # Arguments
    ///
    /// * `container` - The running Docker container
    /// * `container_id` - The validated container ID
    pub(super) fn new(container: ContainerAsync<GenericImage>, container_id: ContainerId) -> Self {
        Self {
            container,
            container_id,
        }
    }

    /// Execute a command in the container and return the output
    ///
    /// # Arguments
    ///
    /// * `command` - Command and arguments to execute
    ///
    /// # Returns
    ///
    /// A `CommandOutput` containing both stdout and stderr streams
    ///
    /// # Note
    ///
    /// The CLI uses tracing which writes logs to stderr, while user-facing messages
    /// go to stdout. The `CommandOutput` type allows tests to inspect either stream
    /// individually or combined.
    pub fn exec(&self, command: &[&str]) -> CommandOutput {
        let output = Command::new("docker")
            .arg("exec")
            .arg(&self.container_id)
            .args(command)
            .output()
            .expect("Failed to execute docker exec command");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        CommandOutput::new(stdout, stderr)
    }

    /// Execute a command and return the exit code
    ///
    /// # Arguments
    ///
    /// * `command` - Command and arguments to execute
    ///
    /// # Returns
    ///
    /// The exit code of the command, or 1 if the process was terminated by a signal
    ///
    /// # Note
    ///
    /// If the process was terminated by a signal (returns None from `code()`), we return 1
    /// to indicate failure rather than 0, which would incorrectly suggest success.
    pub fn exec_with_exit_code(&self, command: &[&str]) -> ExitCode {
        let status = Command::new("docker")
            .arg("exec")
            .arg(&self.container_id)
            .args(command)
            .status()
            .expect("Failed to execute docker exec command");

        // Return 1 (failure) if terminated by signal, otherwise use actual exit code
        status.code().unwrap_or(1)
    }

    /// Copy a file from the host into this running container
    ///
    /// This method uses Docker CLI to copy files into the running container.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file on the host system
    /// * `dest_path` - Destination path inside the container
    ///
    /// # Panics
    ///
    /// Panics if the Docker copy command fails
    pub(super) fn copy_file_to_container(&self, source_path: &Path, dest_path: &str) {
        let output = Command::new("docker")
            .arg("cp")
            .arg(source_path)
            .arg(format!("{}:{dest_path}", self.container_id))
            .output()
            .expect("Failed to execute docker cp command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Failed to copy file to container: {stderr}");
        }
    }
}
