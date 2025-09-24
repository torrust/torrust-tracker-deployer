//! Error types for provisioned container operations
//!
//! This module contains all error types related to container provisioning and management.
//! The errors are organized by logical problem categories to make troubleshooting easier.

/// Top-level error type for container operations
///
/// This error type organizes failures by what logically went wrong, making it easier
/// to understand and troubleshoot issues during container provisioning.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// Problems with the Docker image (build failures, Docker daemon issues)
    #[error("Container image problem: {source}")]
    ContainerImage {
        #[from]
        source: ContainerImageError,
    },

    /// Problems with container runtime (startup, configuration)
    #[error("Container runtime problem: {source}")]
    ContainerRuntime {
        #[from]
        source: ContainerRuntimeError,
    },

    /// Problems with container networking (port mapping, connectivity)
    #[error("Container networking problem: {source}")]
    ContainerNetworking {
        #[from]
        source: ContainerNetworkingError,
    },

    /// Problems with SSH setup and connectivity
    #[error("SSH setup problem: {source}")]
    SshSetup {
        #[from]
        source: SshSetupError,
    },
}

/// Container image-related errors
///
/// These errors occur when there are problems with the Docker image that needs to run.
/// This includes building the image, Docker daemon issues, or missing dependencies.
#[derive(Debug, thiserror::Error)]
pub enum ContainerImageError {
    /// Docker image build process failed
    #[error("Failed to build Docker image '{image_name}:{image_tag}': {reason}")]
    BuildFailed {
        image_name: String,
        image_tag: String,
        reason: String,
        #[source]
        source: super::image_builder::ContainerBuildError,
    },

    /// Docker daemon or command execution issues
    #[error("Docker command execution failed for image '{image_name}:{image_tag}': {reason}")]
    DockerCommandFailed {
        image_name: String,
        image_tag: String,
        reason: String,
        #[source]
        source: std::io::Error,
    },
}

/// Container runtime errors
///
/// These errors occur during container lifecycle operations - starting, configuring,
/// or managing the running container.
#[derive(Debug, thiserror::Error)]
pub enum ContainerRuntimeError {
    /// Container configuration is invalid or malformed
    #[error("Invalid container configuration for image '{image_name}:{image_tag}': {reason}")]
    InvalidConfiguration {
        image_name: String,
        image_tag: String,
        reason: String,
        #[source]
        source: super::config_builder::ContainerConfigError,
    },

    /// Container failed to start or reach expected state
    #[error("Container failed to start for image '{image_name}:{image_tag}': {reason}")]
    StartupFailed {
        image_name: String,
        image_tag: String,
        reason: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },
}

/// Container networking errors
///
/// These errors occur when there are problems with container network access,
/// port mapping, or connectivity.
#[derive(Debug, thiserror::Error)]
pub enum ContainerNetworkingError {
    /// Failed to get mapped ports from container
    #[error("Failed to get mapped port {internal_port} from container '{container_id}': {reason}")]
    PortMappingFailed {
        container_id: String,
        internal_port: u16,
        reason: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },
}

/// SSH setup and connectivity errors
///
/// These errors occur when setting up or testing SSH access to the container.
/// This includes SSH service readiness, key setup, and connection testing.
#[derive(Debug, thiserror::Error)]
pub enum SshSetupError {
    /// SSH service not ready within timeout
    #[error(
        "SSH service not ready on container '{container_id}' port {ssh_port} after {timeout_secs}s"
    )]
    ServiceTimeout {
        container_id: String,
        ssh_port: u16,
        timeout_secs: u64,
    },

    /// SSH key setup failed inside container
    #[error("Failed to setup SSH keys in container '{container_id}': {reason}")]
    KeySetupFailed {
        container_id: String,
        reason: String,
        #[source]
        source: super::actions::ssh_key_setup::SshKeySetupError,
    },

    /// SSH connectivity test failed
    #[error(
        "SSH connectivity test failed for container '{container_id}' on port {ssh_port}: {reason}"
    )]
    ConnectivityTestFailed {
        container_id: String,
        ssh_port: u16,
        reason: String,
        #[source]
        source: super::actions::ssh_wait::SshWaitError,
    },
}

/// Result type alias for container operations
pub type Result<T> = std::result::Result<T, Box<ContainerError>>;
