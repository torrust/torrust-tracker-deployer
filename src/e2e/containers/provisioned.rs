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
//! ## Error Handling
//!
//! This module uses explicit error types through [`ProvisionedContainerError`] instead of
//! generic `anyhow` errors. Each error variant provides specific information about what
//! went wrong, making it easier to handle different failure modes appropriately.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::e2e::containers::{
//!     StoppedProvisionedContainer, ProvisionedContainerError
//! };
//! use torrust_tracker_deploy::infrastructure::adapters::ssh::SshCredentials;
//! use std::path::PathBuf;
//!
//! fn example() -> Result<(), ProvisionedContainerError> {
//!     // Start with stopped state
//!     let stopped = StoppedProvisionedContainer::default();
//!     
//!     // Transition to running state
//!     let running = stopped.start()?;
//!     
//!     // Wait for SSH server
//!     running.wait_for_ssh()?;
//!     
//!     // Setup SSH keys with credentials
//!     let ssh_credentials = SshCredentials::new(
//!         PathBuf::from("/path/to/private_key"),
//!         PathBuf::from("/path/to/public_key.pub"),
//!         "torrust".to_string(),
//!     );
//!     running.setup_ssh_keys(&ssh_credentials)?;
//!     
//!     let (host, port) = running.ssh_details();
//!     
//!     // Transition back to stopped state
//!     let _stopped = running.stop();
//!     Ok(())
//! }
//! ```

use std::time::Duration;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::SyncRunner,
    Container, GenericImage,
};
use tracing::info;

use super::actions::{SshKeySetupAction, SshWaitAction};
use super::config_builder::ContainerConfigBuilder;
use super::executor::ContainerExecutor;
use super::image_builder::ContainerImageBuilder;
use crate::infrastructure::adapters::ssh::SshCredentials;

/// Default Docker image name for provisioned instances
const DEFAULT_IMAGE_NAME: &str = "torrust-provisioned-instance";

/// Default Docker image tag for provisioned instances  
const DEFAULT_IMAGE_TAG: &str = "latest";

/// Specific error types for provisioned container operations
#[derive(Debug, thiserror::Error)]
pub enum ProvisionedContainerError {
    /// Docker image builder error
    #[error("Docker image build failed: {source}")]
    DockerImageBuildFailed {
        #[from]
        source: super::image_builder::ContainerBuildError,
    },

    /// Docker build command execution failed
    #[error("Failed to execute docker build command: {source}")]
    DockerBuildExecution {
        #[source]
        source: std::io::Error,
    },

    /// Docker build process failed with non-zero exit code
    #[error("Docker build failed with stderr: {stderr}")]
    DockerBuildFailed { stderr: String },

    /// Container failed to start
    #[error("Failed to start container: {source}")]
    ContainerStartFailed {
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to get mapped SSH port from container
    #[error("Failed to get mapped SSH port: {source}")]
    SshPortMappingFailed {
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to read SSH public key file
    #[error("Failed to read SSH public key from {path}: {source}")]
    SshKeyFileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to execute SSH key setup command in container
    #[error("Failed to setup SSH keys in container: {source}")]
    SshKeySetupFailed {
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// SSH key setup action failed
    #[error("SSH key setup action failed: {source}")]
    SshKeyActionFailed {
        #[from]
        source: super::actions::ssh_key_setup::SshKeySetupError,
    },

    /// SSH wait action failed
    #[error("SSH wait action failed: {source}")]
    SshWaitActionFailed {
        #[from]
        source: super::actions::ssh_wait::SshWaitError,
    },
}

/// Result type alias for provisioned container operations
pub type Result<T> = std::result::Result<T, ProvisionedContainerError>;

/// Container configuration following state machine pattern
///
/// Following the pattern from Torrust Tracker `MySQL` driver, where different states
/// have different capabilities enforced at compile time.
/// Initial state - container is stopped/not started yet
#[derive(Debug, Default)]
pub struct StoppedProvisionedContainer {}

impl StoppedProvisionedContainer {
    /// Build the Docker image if needed using the `ContainerImageBuilder`
    fn build_image() -> Result<()> {
        let builder = ContainerImageBuilder::new()
            .with_name(DEFAULT_IMAGE_NAME)
            .with_tag(DEFAULT_IMAGE_TAG)
            .with_dockerfile(std::path::PathBuf::from(
                "docker/provisioned-instance/Dockerfile",
            ));
        builder.build()?;
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
        Self::build_image()?;

        info!("Starting provisioned instance container");

        // Create and start the container using the configuration builder
        let image =
            ContainerConfigBuilder::new(format!("{DEFAULT_IMAGE_NAME}:{DEFAULT_IMAGE_TAG}"))
                .with_exposed_port(22)
                .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
                .build();

        let container = image
            .start()
            .map_err(|source| ProvisionedContainerError::ContainerStartFailed { source })?;

        // Get the actual mapped port from testcontainers
        let ssh_port = container
            .get_host_port_ipv4(22.tcp())
            .map_err(|source| ProvisionedContainerError::SshPortMappingFailed { source })?;

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

impl ContainerExecutor for RunningProvisionedContainer {
    fn exec(
        &self,
        command: testcontainers::core::ExecCommand,
    ) -> std::result::Result<(), testcontainers::TestcontainersError> {
        self.container.exec(command).map(|_| ())
    }
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
    /// Returns an error if SSH connectivity cannot be established within
    /// the timeout period or if SSH connection tests fail.
    pub fn wait_for_ssh(&self) -> Result<()> {
        let (host, port) = self.ssh_details();
        let action = SshWaitAction::new(Duration::from_secs(30), 10);
        action
            .execute(&host, port)
            .map_err(|source| ProvisionedContainerError::SshWaitActionFailed { source })
    }

    /// Setup SSH key authentication (only available when running)
    ///
    /// # Arguments
    ///
    /// * `ssh_credentials` - SSH credentials containing the public key path and username
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SSH public key file cannot be read
    /// - Docker exec command fails
    /// - SSH key file operations fail within the container
    pub fn setup_ssh_keys(&self, ssh_credentials: &SshCredentials) -> Result<()> {
        let action = SshKeySetupAction::new();
        action.execute(self, ssh_credentials)?;
        Ok(())
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
    use std::error::Error;
    use std::path::PathBuf;

    #[test]
    fn it_should_create_default_stopped_container() {
        let container = StoppedProvisionedContainer::default();
        assert!(std::ptr::eq(
            std::ptr::addr_of!(container),
            std::ptr::addr_of!(container)
        )); // Just test it exists
    }

    #[test]
    fn it_should_have_proper_error_display_messages() {
        let error = ProvisionedContainerError::DockerBuildFailed {
            stderr: "test error message".to_string(),
        };
        assert!(error.to_string().contains("Docker build failed"));
        assert!(error.to_string().contains("test error message"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_docker_build_execution() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "docker not found");
        let error = ProvisionedContainerError::DockerBuildExecution { source: io_error };

        assert!(error
            .to_string()
            .contains("Failed to execute docker build command"));
        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_preserve_error_chain_for_ssh_key_file_read() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = ProvisionedContainerError::SshKeyFileRead {
            path: "/path/to/key".to_string(),
            source: io_error,
        };

        assert!(error.to_string().contains("Failed to read SSH public key"));
        assert!(error.to_string().contains("/path/to/key"));
        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_convert_docker_build_error_to_provisioned_container_error() {
        use crate::e2e::containers::image_builder::ContainerBuildError;

        let docker_build_error = ContainerBuildError::ContainerBuildFailed {
            image_name: "test-image".to_string(),
            tag: "v1.0".to_string(),
            stderr: "build failed".to_string(),
        };

        let provisioned_error: ProvisionedContainerError = docker_build_error.into();

        assert!(provisioned_error
            .to_string()
            .contains("Docker image build failed"));
        assert!(std::error::Error::source(&provisioned_error).is_some());
    }

    // Note: Integration tests that actually start containers would require Docker
    // and are better suited for the e2e test binaries

    // Helper function to create mock SSH credentials for testing
    #[allow(dead_code)]
    fn create_mock_ssh_credentials() -> SshCredentials {
        SshCredentials::new(
            PathBuf::from("/mock/path/to/private_key"),
            PathBuf::from("/mock/path/to/public_key.pub"),
            "testuser".to_string(),
        )
    }
}
