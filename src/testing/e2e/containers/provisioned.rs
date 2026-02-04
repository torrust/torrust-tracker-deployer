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
//! This module uses explicit error types through [`ContainerError`] instead of
//! generic `anyhow` errors. Each error variant provides specific information about what
//! went wrong, making it easier to handle different failure modes appropriately.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::testing::e2e::containers::{
//!     StoppedProvisionedContainer, ContainerError,
//!     actions::{SshWaitAction, SshKeySetupAction}
//! };
//! use torrust_tracker_deployer_lib::shared::Username;
//! use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use std::net::SocketAddr;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start with stopped state
//!     let stopped = StoppedProvisionedContainer::default();
//!     
//!     // Transition to running state (expose SSH port only)
//!     let running = stopped.start(None, 22, &[]).await?;
//!     
//!     // Get connection details
//!     let socket_addr = running.ssh_socket_addr();
//!     
//!     // Wait for SSH server using action directly
//!     let ssh_wait_action = SshWaitAction::new(Duration::from_secs(30), 10);
//!     ssh_wait_action.execute(socket_addr)?;
//!     
//!     // Setup SSH keys with credentials using action directly
//!     let ssh_credentials = SshCredentials::new(
//!         PathBuf::from("/path/to/private_key"),
//!         PathBuf::from("/path/to/public_key.pub"),
//!         Username::new("torrust").unwrap(),
//!     );
//!     let ssh_key_setup_action = SshKeySetupAction::new();
//!     ssh_key_setup_action.execute(&running, &ssh_credentials).await?;
//!     
//!     // Transition back to stopped state
//!     let _stopped = running.stop();
//!     Ok(())
//! }
//! ```

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};
use tracing::info;

use super::config_builder::ContainerConfigBuilder;
#[cfg(test)]
use super::errors::ContainerNetworkingError;
use super::errors::{ContainerError, ContainerImageError, ContainerRuntimeError, Result};
use super::executor::ContainerExecutor;
use super::image_builder::ContainerImageBuilder;
use super::timeout::ContainerTimeouts;

/// Default Docker image name for provisioned instances
const DEFAULT_IMAGE_NAME: &str = "torrust-provisioned-instance";

/// Default Docker image tag for provisioned instances  
const DEFAULT_IMAGE_TAG: &str = "latest";

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
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::{StoppedProvisionedContainer, ContainerTimeouts};
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
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::StoppedProvisionedContainer;
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
            .with_context(std::path::PathBuf::from("docker/provisioned-instance"))
            .with_build_timeout(docker_build_timeout);
        builder.build().map_err(|e| {
            Box::new(ContainerError::ContainerImage {
                source: ContainerImageError::BuildFailed {
                    image_name: DEFAULT_IMAGE_NAME.to_string(),
                    image_tag: DEFAULT_IMAGE_TAG.to_string(),
                    reason: "Docker image build process failed".to_string(),
                    source: *e,
                },
            })
        })?;
        Ok(())
    }

    /// Start the container and transition to running state
    ///
    /// # Arguments
    ///
    /// * `container_name` - Optional name for the running container. If provided, the container will be named accordingly.
    /// * `ssh_port` - The internal SSH port to expose from the container
    /// * `additional_ports` - Additional TCP ports to expose (e.g., tracker API, HTTP tracker)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Docker image build fails
    /// - Container fails to start
    /// - Container networking setup fails
    pub async fn start(
        self,
        container_name: Option<String>,
        ssh_port: u16,
        additional_ports: &[u16],
    ) -> Result<RunningProvisionedContainer> {
        // First build the Docker image if needed
        Self::build_image(self.timeouts.docker_build)?;

        info!(
            ssh_port = %ssh_port,
            additional_ports = ?additional_ports,
            "Starting provisioned instance container with Docker-in-Docker support"
        );

        // Create and start the container using the configuration builder
        // Wait for both SSH and Docker daemon to be ready
        let mut config_builder =
            ContainerConfigBuilder::new(format!("{DEFAULT_IMAGE_NAME}:{DEFAULT_IMAGE_TAG}"))
                .with_exposed_port(ssh_port)
                .with_wait_condition(WaitFor::message_on_stdout("dockerd entered RUNNING state"));

        // Add additional ports (tracker API, HTTP tracker, etc.)
        for port in additional_ports {
            config_builder = config_builder.with_exposed_port(*port);
        }

        let image = config_builder.build().map_err(|source| {
            Box::new(ContainerError::ContainerRuntime {
                source: ContainerRuntimeError::InvalidConfiguration {
                    image_name: DEFAULT_IMAGE_NAME.to_string(),
                    image_tag: DEFAULT_IMAGE_TAG.to_string(),
                    reason: "Container configuration validation failed".to_string(),
                    source: *source,
                },
            })
        })?;

        // Start the container with privileged mode for Docker-in-Docker support
        // and optional container name
        let container = if let Some(name) = container_name {
            info!(container_name = %name, "Starting container with custom name and privileged mode");
            image
                .with_privileged(true)
                .with_container_name(name)
                .start()
                .await
        } else {
            image.with_privileged(true).start().await
        }
        .map_err(|source| {
            Box::new(ContainerError::ContainerRuntime {
                source: ContainerRuntimeError::StartupFailed {
                    image_name: DEFAULT_IMAGE_NAME.to_string(),
                    image_tag: DEFAULT_IMAGE_TAG.to_string(),
                    reason: "Container failed to start or reach expected state".to_string(),
                    source,
                },
            })
        })?;

        // Get the dynamically assigned ports from Docker's port mapping (bridge networking)
        let mapped_ssh_port = container.get_host_port_ipv4(ssh_port).await.map_err(|e| {
            Box::new(ContainerError::ContainerRuntime {
                source: ContainerRuntimeError::StartupFailed {
                    image_name: DEFAULT_IMAGE_NAME.to_string(),
                    image_tag: DEFAULT_IMAGE_TAG.to_string(),
                    reason: format!("Failed to get mapped SSH port: {e}"),
                    source: e,
                },
            })
        })?;

        // Get mapped ports for all additional ports (tracker services)
        let mut mapped_additional_ports = Vec::new();
        for port in additional_ports {
            let mapped_port = container.get_host_port_ipv4(*port).await.map_err(|e| {
                Box::new(ContainerError::ContainerRuntime {
                    source: ContainerRuntimeError::StartupFailed {
                        image_name: DEFAULT_IMAGE_NAME.to_string(),
                        image_tag: DEFAULT_IMAGE_TAG.to_string(),
                        reason: format!("Failed to get mapped port for {port}: {e}"),
                        source: e,
                    },
                })
            })?;
            mapped_additional_ports.push(mapped_port);
        }

        info!(
            container_id = %container.id(),
            mapped_ssh_port,
            mapped_additional_ports = ?mapped_additional_ports,
            "Container started successfully with bridge networking"
        );

        Ok(RunningProvisionedContainer::new(
            container,
            mapped_ssh_port,
            mapped_additional_ports,
        ))
    }
}

/// Running state - container is started and can be configured
pub struct RunningProvisionedContainer {
    container: ContainerAsync<GenericImage>,
    ssh_port: u16,
    additional_mapped_ports: Vec<u16>,
}

impl ContainerExecutor for RunningProvisionedContainer {
    async fn exec(
        &self,
        command: testcontainers::core::ExecCommand,
    ) -> std::result::Result<(), testcontainers::TestcontainersError> {
        self.container.exec(command).await.map(|_| ())
    }
}

impl RunningProvisionedContainer {
    pub(crate) fn new(
        container: ContainerAsync<GenericImage>,
        ssh_port: u16,
        additional_mapped_ports: Vec<u16>,
    ) -> Self {
        Self {
            container,
            ssh_port,
            additional_mapped_ports,
        }
    }

    /// Get the SSH connection details
    #[must_use]
    pub fn ssh_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.ssh_port)
    }

    /// Get the mapped additional ports (tracker API, HTTP tracker, UDP tracker, etc.)
    /// Returns ports in the same order they were requested when starting the container
    #[must_use]
    pub fn additional_mapped_ports(&self) -> &[u16] {
        &self.additional_mapped_ports
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
        let error = ContainerError::ContainerImage {
            source: ContainerImageError::BuildFailed {
                image_name: "test-image".to_string(),
                image_tag: "test-tag".to_string(),
                reason: "Docker build compilation failed".to_string(),
                source: crate::testing::e2e::containers::image_builder::ContainerBuildError::ContainerBuildFailed {
                    image_name: "test-image".to_string(),
                    tag: "test-tag".to_string(),
                    dockerfile_path: "/path/to/Dockerfile".to_string(),
                    context_path: "/build/context".to_string(),
                    build_duration_secs: 60,
                    stderr: "test error message".to_string(),
                },
            },
        };
        assert!(error.to_string().contains("Container image problem"));
        assert!(error.to_string().contains("Failed to build Docker image"));
        assert!(error.to_string().contains("test-image:test-tag"));
        assert!(error
            .to_string()
            .contains("Docker build compilation failed"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_docker_command_execution() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "docker not found");
        let image_error = ContainerImageError::DockerCommandFailed {
            image_name: "test-image".to_string(),
            image_tag: "test-tag".to_string(),
            reason: "Docker daemon not available".to_string(),
            source: io_error,
        };
        let error = ContainerError::ContainerImage {
            source: image_error,
        };

        assert!(error
            .to_string()
            .contains("Docker command execution failed"));
        assert!(error.to_string().contains("test-image:test-tag"));
        assert!(error.to_string().contains("Docker daemon not available"));
        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_convert_docker_build_error_to_provisioned_container_error() {
        use crate::testing::e2e::containers::image_builder::ContainerBuildError;

        let docker_build_error = ContainerBuildError::ContainerBuildFailed {
            image_name: "test-image".to_string(),
            tag: "v1.0".to_string(),
            dockerfile_path: "/path/to/Dockerfile".to_string(),
            context_path: "/build/context".to_string(),
            build_duration_secs: 60,
            stderr: "build failed".to_string(),
        };

        let image_error = ContainerImageError::BuildFailed {
            image_name: "test-image".to_string(),
            image_tag: "v1.0".to_string(),
            reason: "Docker image build process failed".to_string(),
            source: docker_build_error,
        };
        let provisioned_error = ContainerError::ContainerImage {
            source: image_error,
        };

        assert!(provisioned_error
            .to_string()
            .contains("Container image problem"));
        assert!(std::error::Error::source(&provisioned_error).is_some());
    }

    #[test]
    fn it_should_group_networking_errors_logically() {
        // Test port mapping error
        let testcontainers_error = testcontainers::TestcontainersError::other("port conflict");
        let networking_error = ContainerNetworkingError::PortMappingFailed {
            container_id: "container123".to_string(),
            internal_port: 22,
            reason: "Port already in use".to_string(),
            source: testcontainers_error,
        };
        let provisioned_error = ContainerError::ContainerNetworking {
            source: networking_error,
        };

        assert!(provisioned_error
            .to_string()
            .contains("Container networking problem"));
        assert!(provisioned_error
            .to_string()
            .contains("Failed to get mapped port 22"));
        assert!(provisioned_error
            .to_string()
            .contains("Port already in use"));
    }

    #[test]
    fn it_should_group_runtime_errors_logically() {
        let testcontainers_error = testcontainers::TestcontainersError::other("resource limit");
        let runtime_error = ContainerRuntimeError::StartupFailed {
            image_name: "test-image".to_string(),
            image_tag: "latest".to_string(),
            reason: "Insufficient memory".to_string(),
            source: testcontainers_error,
        };
        let provisioned_error = ContainerError::ContainerRuntime {
            source: runtime_error,
        };

        assert!(provisioned_error
            .to_string()
            .contains("Container runtime problem"));
        assert!(provisioned_error
            .to_string()
            .contains("Container failed to start"));
        assert!(provisioned_error
            .to_string()
            .contains("Insufficient memory"));
    }

    #[test]
    fn it_should_allow_matching_on_logical_error_categories() {
        let image_error = ContainerImageError::BuildFailed {
            image_name: "test".to_string(),
            image_tag: "latest".to_string(),
            reason: "Build failed".to_string(),
            source: crate::testing::e2e::containers::image_builder::ContainerBuildError::ImageNameRequired,
        };
        let error = ContainerError::ContainerImage {
            source: image_error,
        };

        // Test that we can match on logical error categories
        match error {
            ContainerError::ContainerImage { .. } => {
                // This should match - demonstrates logical error categorization
            }
            ContainerError::ContainerRuntime { .. } => {
                panic!("Should not match runtime category");
            }
            ContainerError::ContainerNetworking { .. } => {
                panic!("Should not match networking category");
            }
            ContainerError::SshSetup { .. } => {
                panic!("Should not match SSH category");
            }
        }
    }

    // Note: Integration tests that actually start containers would require Docker
    // and are better suited for the e2e test binaries
}
