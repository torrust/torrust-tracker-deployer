//! Real SSH server container using Docker

use std::net::{IpAddr, Ipv4Addr};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

use super::config::SshServerConfig;
use super::constants::SSH_CONTAINER_PORT;
use super::error::SshServerError;
use crate::shared::docker::DockerClient;

/// Real SSH server container using Docker
///
/// This implementation starts an actual Docker container running SSH server
/// for full integration testing. Use this when you need to test actual SSH
/// protocol connectivity and command execution.
pub struct RealSshServerContainer {
    config: SshServerConfig,
    #[allow(dead_code)]
    container: ContainerAsync<GenericImage>,
    host_ip: IpAddr,
    ssh_port: u16,
}

impl RealSshServerContainer {
    /// Start a real SSH server container with custom configuration
    ///
    /// This starts an actual Docker container running SSH server using the provided
    /// configuration. The Docker image is built automatically from the Dockerfile.
    /// This ensures the tests are self-contained and work in CI environments.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration specifying image name, credentials, ports, etc.
    ///
    /// # Returns
    ///
    /// A real container with SSH server running, or an error if Docker
    /// is not available or the image cannot be built/started.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Dockerfile is not found at the configured path
    /// - Docker build command fails
    /// - Container startup fails
    /// - Port mapping fails
    pub async fn start_with_config(config: SshServerConfig) -> Result<Self, SshServerError> {
        // Build the SSH server image from Dockerfile
        // This ensures tests are self-contained and work in CI
        let dockerfile_dir = &config.dockerfile_dir;

        if !dockerfile_dir.exists() || !dockerfile_dir.join("Dockerfile").exists() {
            return Err(SshServerError::DockerfileNotFound {
                expected_path: dockerfile_dir.join("Dockerfile"),
            });
        }

        // Build the Docker image using docker CLI
        println!(
            "Building SSH server Docker image from {}",
            dockerfile_dir.display()
        );

        let docker_client = DockerClient::new();
        docker_client
            .build_image(
                &config.dockerfile_dir,
                &config.image_name,
                &config.image_tag,
            )
            .map_err(|source| SshServerError::DockerClientError {
                source: Box::new(source),
            })?;

        println!("SSH server Docker image built successfully");

        // Start the container using the built image
        // Note: SSH always runs on port 22 inside the container
        let image = GenericImage::new(&config.image_name, &config.image_tag)
            .with_exposed_port(SSH_CONTAINER_PORT.tcp())
            .with_wait_for(WaitFor::seconds(config.startup_wait_secs));

        let container = image
            .start()
            .await
            .map_err(|source| SshServerError::ContainerStartFailed { source })?;

        // Get the mapped SSH port (testcontainers maps to random host port)
        let ssh_port: u16 = container
            .get_host_port_ipv4(SSH_CONTAINER_PORT)
            .await
            .map_err(|source| SshServerError::PortMappingFailed { source })?;

        Ok(Self {
            config,
            container,
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port,
        })
    }

    /// Start a real SSH server container with default configuration
    ///
    /// This is a convenience method that starts a container with default
    /// configuration values from constants. For custom configuration,
    /// use [`Self::start_with_config`].
    ///
    /// # Returns
    ///
    /// A real container with SSH server running, or an error if Docker
    /// is not available or the image cannot be built/started.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `docker/ssh-server/Dockerfile` is not found
    /// - Docker build command fails
    /// - Container startup fails
    /// - Port mapping fails
    pub async fn start() -> Result<Self, SshServerError> {
        Self::start_with_config(SshServerConfig::default()).await
    }

    /// Get the SSH port mapped by the container
    ///
    /// Returns the host port that maps to the container's SSH port (22).
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    /// Get the container's host IP address
    ///
    /// Returns the IP address to connect to the container from the host.
    #[must_use]
    pub fn host_ip(&self) -> IpAddr {
        self.host_ip
    }

    /// Get the test username configured in the container
    #[must_use]
    pub fn test_username(&self) -> &str {
        &self.config.username
    }

    /// Get the test password configured in the container
    #[must_use]
    pub fn test_password(&self) -> &str {
        &self.config.password
    }
}

impl super::SshServerContainer for RealSshServerContainer {
    fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    fn host_ip(&self) -> IpAddr {
        self.host_ip
    }

    fn test_username(&self) -> &str {
        &self.config.username
    }

    fn test_password(&self) -> &str {
        &self.config.password
    }
}
