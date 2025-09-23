//! Provisioned Instance Container for E2E Testing
//!
//! This module provides a state machine pattern for managing Docker containers
//! that represent provisioned instances in the deployment workflow.
//!
//! ## State Machine Pattern
//!
//! The container follows a state machine pattern similar to the Torrust Tracker `MySQL` driver:
//! - `StoppedProvisionedContainer` - Initial state, can only be started
//! - `RunningProvisionedContainer` - Running state, can be queried, configured, and stopped
//! - State transitions are enforced at compile time through different types
//!
//! ## Usage
//!
//! ```rust,no_run
//! use anyhow::Result;
//! use torrust_tracker_deploy::e2e::provisioned_container::StoppedProvisionedContainer;
//!
//! fn example() -> Result<()> {
//!     // Start with stopped state
//!     let stopped = StoppedProvisionedContainer::default();
//!     
//!     // Transition to running state
//!     let running = stopped.start()?;
//!     
//!     // Operations only available when running
//!     running.wait_for_ssh()?;
//!     running.setup_ssh_keys()?;
//!     let (host, port) = running.ssh_details();
//!     
//!     // Transition back to stopped state
//!     let _stopped = running.stop();
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use std::time::Duration;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::SyncRunner,
    Container, GenericImage, ImageExt,
};
use tracing::info;

/// Container configuration following state machine pattern
///
/// Following the pattern from Torrust Tracker `MySQL` driver, where different states
/// have different capabilities enforced at compile time.
/// Initial state - container is stopped/not started yet
#[derive(Debug, Default)]
pub struct StoppedProvisionedContainer {}

impl StoppedProvisionedContainer {
    /// Build the Docker image if needed
    fn build_docker_image() -> Result<()> {
        info!("Building torrust-provisioned-instance Docker image");

        let output = std::process::Command::new("docker")
            .args([
                "build",
                "-t",
                "torrust-provisioned-instance:latest",
                "-f",
                "docker/provisioned-instance/Dockerfile",
                ".",
            ])
            .output()
            .context("Failed to execute docker build command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Docker build failed: {}", stderr));
        }

        info!("Docker image built successfully");
        Ok(())
    }

    /// Start the container and transition to running state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Docker image build fails
    /// - Container fails to start
    /// - Container networking setup fails
    pub fn start(self) -> Result<RunningProvisionedContainer> {
        // First build the Docker image if needed
        Self::build_docker_image()?;

        info!("Starting provisioned instance container");

        // Create and start the container with fixed port mapping (22:22)
        let image = GenericImage::new("torrust-provisioned-instance", "latest")
            .with_exposed_port(22.tcp())
            .with_wait_for(WaitFor::message_on_stdout("sshd entered RUNNING state"));

        let container = image
            .with_mapped_port(22, 22.tcp())
            .start()
            .context("Failed to start container")?;

        // Use fixed port 22 since we're mapping 22:22
        let ssh_port = 22_u16;

        info!(
            container_id = %container.id(),
            ssh_port = ssh_port,
            "Container started successfully"
        );

        Ok(RunningProvisionedContainer::new(container, ssh_port))
    }
}

/// Running state - container is started and can be configured
pub struct RunningProvisionedContainer {
    container: Container<GenericImage>,
    ssh_port: u16,
}

impl RunningProvisionedContainer {
    pub(crate) fn new(container: Container<GenericImage>, ssh_port: u16) -> Self {
        Self {
            container,
            ssh_port,
        }
    }

    /// Get the SSH connection details for Ansible
    #[must_use]
    pub fn ssh_details(&self) -> (String, u16) {
        ("127.0.0.1".to_string(), self.ssh_port)
    }

    /// Wait for SSH server to be ready (only available when running)
    ///
    /// # Errors
    ///
    /// Currently always returns Ok, but may return errors in future implementations
    /// if SSH connectivity checks fail.
    pub fn wait_for_ssh(&self) -> Result<()> {
        info!(port = self.ssh_port, "Waiting for SSH server to be ready");

        // Simple wait - in a real implementation, we could ping SSH port
        std::thread::sleep(Duration::from_secs(5));

        info!("SSH server should be ready");
        Ok(())
    }

    /// Setup SSH key authentication (only available when running)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Docker exec command fails
    /// - SSH key file operations fail within the container
    pub fn setup_ssh_keys(&self) -> Result<()> {
        info!("Setting up SSH key authentication");

        // Read the public key from fixtures
        let project_root = std::env::current_dir().context("Failed to get current directory")?;
        let public_key_path = project_root.join("fixtures/testing_rsa.pub");
        let public_key_content =
            std::fs::read_to_string(&public_key_path).context("Failed to read SSH public key")?;

        // Copy the public key into the container's authorized_keys
        let exec_result = self.container.exec(testcontainers::core::ExecCommand::new([
            "sh",
            "-c",
            &format!("echo '{}' >> /home/torrust/.ssh/authorized_keys && chmod 600 /home/torrust/.ssh/authorized_keys", public_key_content.trim()),
        ]));

        match exec_result {
            Ok(_) => {
                info!("SSH key authentication configured");
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to setup SSH keys in container: {}",
                e
            )),
        }
    }

    /// Get the container ID for logging/debugging
    #[must_use]
    pub fn container_id(&self) -> &str {
        self.container.id()
    }

    /// Stop the container and transition back to stopped state
    pub fn stop(self) -> StoppedProvisionedContainer {
        info!(container_id = %self.container.id(), "Stopping container");
        // Container will be automatically cleaned up when dropped
        StoppedProvisionedContainer::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_default_stopped_container() {
        let container = StoppedProvisionedContainer::default();
        assert!(std::ptr::eq(
            std::ptr::addr_of!(container),
            std::ptr::addr_of!(container)
        )); // Just test it exists
    }

    // Note: Integration tests that actually start containers would require Docker
    // and are better suited for the e2e test binaries
}
