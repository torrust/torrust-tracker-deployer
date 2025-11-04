//! Ubuntu container helper for testing the dependency-installer CLI

use std::path::{Path, PathBuf};

use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};

use super::helpers::{copy_file_to_container, exec_in_container, exec_in_container_with_exit_code};

/// Helper for managing Ubuntu test containers
///
/// This struct provides a fluent API for creating and managing Ubuntu containers
/// used in integration tests. It handles container lifecycle, file copying,
/// and command execution.
pub struct UbuntuTestContainer {
    // Keep a reference to the container so it stays alive
    #[allow(dead_code)]
    container: ContainerAsync<GenericImage>,
    container_id: String,
}

impl UbuntuTestContainer {
    /// Execute a command in the container and return stdout
    ///
    /// # Arguments
    ///
    /// * `command` - Command and arguments to execute
    ///
    /// # Returns
    ///
    /// The stdout output as a string
    pub fn exec(&self, command: &[&str]) -> String {
        exec_in_container(&self.container_id, command)
    }

    /// Execute a command and return the exit code
    ///
    /// # Arguments
    ///
    /// * `command` - Command and arguments to execute
    ///
    /// # Returns
    ///
    /// The exit code of the command
    pub fn exec_with_exit_code(&self, command: &[&str]) -> i32 {
        exec_in_container_with_exit_code(&self.container_id, command)
    }
}

/// Builder for Ubuntu containers
pub struct UbuntuContainerBuilder;

impl UbuntuContainerBuilder {
    /// Create a new Ubuntu container builder
    pub fn new() -> Self {
        Self
    }

    /// Add the binary to the container
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to the binary on the host
    ///
    /// # Returns
    ///
    /// A builder that can start the container with the binary
    #[allow(clippy::unused_self)]
    pub fn with_binary(self, binary_path: &Path) -> UbuntuContainerWithBinary {
        UbuntuContainerWithBinary {
            binary_path: binary_path.to_path_buf(),
        }
    }
}

impl Default for UbuntuContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder with binary path specified
pub struct UbuntuContainerWithBinary {
    binary_path: PathBuf,
}

impl UbuntuContainerWithBinary {
    /// Start the container with the binary
    ///
    /// This method:
    /// 1. Starts an Ubuntu 24.04 container
    /// 2. Copies the binary into the container
    /// 3. Makes the binary executable
    ///
    /// # Returns
    ///
    /// A running container ready for test execution
    pub async fn start(self) -> UbuntuTestContainer {
        // Create Ubuntu 24.04 image
        let image = GenericImage::new("ubuntu", "24.04")
            .with_wait_for(WaitFor::seconds(2))
            .with_cmd(vec!["sleep", "infinity"]);

        // Start the container
        let container = image.start().await.expect("Failed to start container");

        // Get container ID for docker CLI operations
        let container_id = container.id().to_string();

        // Copy the binary into the container
        copy_file_to_container(
            &container_id,
            &self.binary_path,
            "/usr/local/bin/dependency-installer",
        );

        // Make the binary executable
        exec_in_container(
            &container_id,
            &["chmod", "+x", "/usr/local/bin/dependency-installer"],
        );

        UbuntuTestContainer {
            container,
            container_id,
        }
    }
}
