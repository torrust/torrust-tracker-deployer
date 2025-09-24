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
//!     StoppedProvisionedContainer, ProvisionedContainerError,
//!     actions::{SshWaitAction, SshKeySetupAction}
//! };
//! use torrust_tracker_deploy::shared::ssh::SshCredentials;
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use std::net::SocketAddr;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start with stopped state
//!     let stopped = StoppedProvisionedContainer::default();
//!     
//!     // Transition to running state
//!     let running = stopped.start()?;
//!     
//!     // Get connection details
//!     let socket_addr = running.ssh_details();
//!     
//!     // Wait for SSH server using action directly
//!     let ssh_wait_action = SshWaitAction::new(Duration::from_secs(30), 10);
//!     ssh_wait_action.execute(socket_addr)?;
//!     
//!     // Setup SSH keys with credentials using action directly
//!     let ssh_credentials = SshCredentials::new(
//!         PathBuf::from("/path/to/private_key"),
//!         PathBuf::from("/path/to/public_key.pub"),
//!         "torrust".to_string(),
//!     );
//!     let ssh_key_setup_action = SshKeySetupAction::new();
//!     ssh_key_setup_action.execute(&running, &ssh_credentials)?;
//!     
//!     // Transition back to stopped state
//!     let _stopped = running.stop();
//!     Ok(())
//! }
//! ```

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::SyncRunner,
    Container, GenericImage,
};
use tracing::info;

use super::config_builder::ContainerConfigBuilder;
use super::executor::ContainerExecutor;
use super::image_builder::ContainerImageBuilder;

/// Default Docker image name for provisioned instances
const DEFAULT_IMAGE_NAME: &str = "torrust-provisioned-instance";

/// Default Docker image tag for provisioned instances  
const DEFAULT_IMAGE_TAG: &str = "latest";

/// Container timeout configurations for different operations
///
/// This struct provides configurable timeouts for various container operations
/// to make the system more flexible and adaptable to different environments.
#[derive(Debug, Clone)]
pub struct ContainerTimeouts {
    /// Timeout for Docker image build operations
    pub docker_build: Duration,
    /// Timeout for container startup operations
    pub container_start: Duration,
    /// Timeout for SSH connectivity to become available
    pub ssh_ready: Duration,
    /// Timeout for SSH key setup operations
    pub ssh_setup: Duration,
}

impl Default for ContainerTimeouts {
    fn default() -> Self {
        Self {
            docker_build: Duration::from_secs(300),   // 5 minutes
            container_start: Duration::from_secs(60), // 1 minute
            ssh_ready: Duration::from_secs(30),       // 30 seconds
            ssh_setup: Duration::from_secs(15),       // 15 seconds
        }
    }
}

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
    #[error(
        "Failed to execute docker build command for image '{image_name}:{image_tag}': {source}"
    )]
    DockerBuildExecution {
        image_name: String,
        image_tag: String,
        #[source]
        source: std::io::Error,
    },

    /// Docker build process failed with non-zero exit code
    #[error("Docker build failed for image '{image_name}:{image_tag}' with stderr: {stderr}")]
    DockerBuildFailed {
        image_name: String,
        image_tag: String,
        stderr: String,
    },

    /// Container failed to start
    #[error("Failed to start container for image '{image_name}:{image_tag}': {source}")]
    ContainerStartFailed {
        image_name: String,
        image_tag: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to get mapped SSH port from container
    #[error("Failed to get mapped SSH port {internal_port} from container '{container_id}' (image '{image_name}:{image_tag}'): {source}")]
    SshPortMappingFailed {
        container_id: String,
        image_name: String,
        image_tag: String,
        internal_port: u16,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// SSH setup timeout
    #[error("SSH setup timeout after {timeout_secs}s for container '{container_id}' (image '{image_name}:{image_tag}') on port {ssh_port}")]
    SshSetupTimeout {
        container_id: String,
        image_name: String,
        image_tag: String,
        ssh_port: u16,
        timeout_secs: u64,
    },

    /// SSH key setup failed
    #[error("SSH key setup failed for container '{container_id}' (image '{image_name}:{image_tag}'): {source}")]
    SshKeySetupFailed {
        container_id: String,
        image_name: String,
        image_tag: String,
        #[source]
        source: super::actions::ssh_key_setup::SshKeySetupError,
    },

    /// SSH connectivity wait failed
    #[error("SSH connectivity wait failed for container '{container_id}' (image '{image_name}:{image_tag}') on port {ssh_port}: {source}")]
    SshWaitFailed {
        container_id: String,
        image_name: String,
        image_tag: String,
        ssh_port: u16,
        #[source]
        source: super::actions::ssh_wait::SshWaitError,
    },

    /// Container configuration building failed
    #[error("Container configuration build failed for image '{image_name}:{image_tag}': {source}")]
    ContainerConfigBuildFailed {
        image_name: String,
        image_tag: String,
        #[source]
        source: super::config_builder::ContainerConfigError,
    },
}

/// Result type alias for provisioned container operations
pub type Result<T> = std::result::Result<T, Box<ProvisionedContainerError>>;

/// Container configuration following state machine pattern
///
/// Following the pattern from Torrust Tracker `MySQL` driver, where different states
/// have different capabilities enforced at compile time.
/// Initial state - container is stopped/not started yet
#[derive(Debug)]
pub struct StoppedProvisionedContainer {
    /// Timeout configurations for container operations
    pub timeouts: ContainerTimeouts,
}

#[allow(clippy::derivable_impls)]
impl Default for StoppedProvisionedContainer {
    fn default() -> Self {
        Self {
            timeouts: ContainerTimeouts::default(),
        }
    }
}

impl StoppedProvisionedContainer {
    /// Create a new stopped container with custom timeout configurations
    ///
    /// # Arguments
    /// * `timeouts` - Custom timeout configuration for container operations
    ///
    /// # Example
    /// ```rust,no_run
    /// use torrust_tracker_deploy::e2e::containers::{StoppedProvisionedContainer, ContainerTimeouts};
    /// use std::time::Duration;
    ///
    /// let mut timeouts = ContainerTimeouts::default();
    /// timeouts.ssh_ready = Duration::from_secs(60);
    ///
    /// let container = StoppedProvisionedContainer::with_timeouts(timeouts);
    /// ```
    #[must_use]
    pub fn with_timeouts(timeouts: ContainerTimeouts) -> Self {
        Self { timeouts }
    }

    /// Create a new stopped container with custom SSH ready timeout
    ///
    /// This is a convenience method for the most commonly customized timeout.
    ///
    /// # Arguments
    /// * `ssh_ready_timeout` - How long to wait for SSH to become available
    ///
    /// # Example
    /// ```rust,no_run
    /// use torrust_tracker_deploy::e2e::containers::StoppedProvisionedContainer;
    /// use std::time::Duration;
    ///
    /// let container = StoppedProvisionedContainer::with_ssh_ready_timeout(
    ///     Duration::from_secs(60)
    /// );
    /// ```
    #[must_use]
    pub fn with_ssh_ready_timeout(ssh_ready_timeout: Duration) -> Self {
        let timeouts = ContainerTimeouts {
            ssh_ready: ssh_ready_timeout,
            ..ContainerTimeouts::default()
        };
        Self { timeouts }
    }

    /// Build the Docker image if needed using the `ContainerImageBuilder`
    fn build_image(docker_build_timeout: Duration) -> Result<()> {
        let builder = ContainerImageBuilder::new()
            .with_name(DEFAULT_IMAGE_NAME)
            .with_tag(DEFAULT_IMAGE_TAG)
            .with_dockerfile(std::path::PathBuf::from(
                "docker/provisioned-instance/Dockerfile",
            ))
            .with_build_timeout(docker_build_timeout);
        builder.build().map_err(|e| {
            Box::new(ProvisionedContainerError::DockerImageBuildFailed { source: *e })
        })?;
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
        Self::build_image(self.timeouts.docker_build)?;

        info!("Starting provisioned instance container");

        // Create and start the container using the configuration builder
        let image =
            ContainerConfigBuilder::new(format!("{DEFAULT_IMAGE_NAME}:{DEFAULT_IMAGE_TAG}"))
                .with_exposed_port(22)
                .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
                .build()
                .map_err(|source| {
                    Box::new(ProvisionedContainerError::ContainerConfigBuildFailed {
                        image_name: DEFAULT_IMAGE_NAME.to_string(),
                        image_tag: DEFAULT_IMAGE_TAG.to_string(),
                        source: *source,
                    })
                })?;

        let container = image.start().map_err(|source| {
            Box::new(ProvisionedContainerError::ContainerStartFailed {
                image_name: DEFAULT_IMAGE_NAME.to_string(),
                image_tag: DEFAULT_IMAGE_TAG.to_string(),
                source,
            })
        })?;

        // Get the actual mapped port from testcontainers
        let ssh_port = container.get_host_port_ipv4(22.tcp()).map_err(|source| {
            Box::new(ProvisionedContainerError::SshPortMappingFailed {
                container_id: container.id().to_string(),
                image_name: DEFAULT_IMAGE_NAME.to_string(),
                image_tag: DEFAULT_IMAGE_TAG.to_string(),
                internal_port: 22,
                source,
            })
        })?;

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
    pub fn ssh_details(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.ssh_port)
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
            image_name: "test-image".to_string(),
            image_tag: "test-tag".to_string(),
            stderr: "test error message".to_string(),
        };
        assert!(error.to_string().contains("Docker build failed"));
        assert!(error.to_string().contains("test error message"));
        assert!(error.to_string().contains("test-image:test-tag"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_docker_build_execution() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "docker not found");
        let error = ProvisionedContainerError::DockerBuildExecution {
            image_name: "test-image".to_string(),
            image_tag: "test-tag".to_string(),
            source: io_error,
        };

        assert!(error
            .to_string()
            .contains("Failed to execute docker build command"));
        assert!(error.to_string().contains("test-image:test-tag"));
        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_convert_docker_build_error_to_provisioned_container_error() {
        use crate::e2e::containers::image_builder::ContainerBuildError;

        let docker_build_error = ContainerBuildError::ContainerBuildFailed {
            image_name: "test-image".to_string(),
            tag: "v1.0".to_string(),
            dockerfile_path: "/path/to/Dockerfile".to_string(),
            context_path: "/build/context".to_string(),
            build_duration_secs: 60,
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
}
