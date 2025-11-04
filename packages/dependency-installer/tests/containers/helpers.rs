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
pub fn exec_in_container(container_id: &str, command: &[&str]) -> String {
    let output = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .args(command)
        .output()
        .expect("Failed to execute docker exec command");

    // Combine stdout and stderr to capture all output (logs go to stderr)
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
/// The exit code of the command
pub fn exec_in_container_with_exit_code(container_id: &str, command: &[&str]) -> i32 {
    let status = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .args(command)
        .status()
        .expect("Failed to execute docker exec command");

    status.code().unwrap_or(0)
}
