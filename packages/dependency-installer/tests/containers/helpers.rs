//! Helper utilities for working with testcontainers
//!
//! Provides utility functions for common container operations like
//! copying files into running containers and executing commands.

use std::path::Path;
use std::process::Command;

/// Copy a file from the host into a running container
///
/// This function uses Docker CLI to copy files into a running container.
/// The container must be running when this function is called.
///
/// # Arguments
///
/// * `container_id` - The Docker container ID
/// * `source_path` - Path to the file on the host system
/// * `dest_path` - Destination path inside the container
///
/// # Panics
///
/// Panics if the Docker copy command fails
pub fn copy_file_to_container(container_id: &str, source_path: &Path, dest_path: &str) {
    // Use docker cp command to copy the file
    let output = Command::new("docker")
        .arg("cp")
        .arg(source_path)
        .arg(format!("{}:{}", container_id, dest_path))
        .output()
        .expect("Failed to execute docker cp command");

    if !output.status.success() {
        panic!(
            "Failed to copy file to container: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Execute a command in a container and return output
///
/// # Arguments
///
/// * `container_id` - The Docker container ID
/// * `command` - Command and arguments to execute
///
/// # Returns
///
/// The combined stdout and stderr output as a string
///
/// # Note
///
/// The output combines stderr and stdout because the CLI uses tracing which writes
/// logs to stderr, while user-facing messages go to stdout. We need both for
/// comprehensive test assertions. Stderr is placed first to maintain chronological
/// order of log messages relative to output.
pub fn exec_in_container(container_id: &str, command: &[&str]) -> String {
    let output = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .args(command)
        .output()
        .expect("Failed to execute docker exec command");

    // Combine stderr (logs) and stdout (user messages) to capture all output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}{}", stderr, stdout)
}

/// Execute a command in a container and return exit code
///
/// # Arguments
///
/// * `container_id` - The Docker container ID
/// * `command` - Command and arguments to execute
///
/// # Returns
///
/// The exit code of the command, or 1 if the process was terminated by a signal
///
/// # Note
///
/// If the process was terminated by a signal (returns None from code()), we return 1
/// to indicate failure rather than 0, which would incorrectly suggest success.
pub fn exec_in_container_with_exit_code(container_id: &str, command: &[&str]) -> i32 {
    let status = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .args(command)
        .status()
        .expect("Failed to execute docker exec command");

    // Return 1 (failure) if terminated by signal, otherwise use actual exit code
    status.code().unwrap_or(1)
}
