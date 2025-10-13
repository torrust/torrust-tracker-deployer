//! Real SSH server container using Docker

use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

use super::config::SshServerConfig;
use super::error::SshServerError;

/// Real SSH server container using Docker
///
/// This implementation starts an actual Docker container running SSH server
/// for full integration testing. Use this when you need to test actual SSH
/// protocol connectivity and command execution.
pub struct RealSshServerContainer {
    #[allow(dead_code)]
    container: ContainerAsync<GenericImage>,
    host_ip: IpAddr,
    ssh_port: u16,
    test_username: String,
    test_password: String,
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

        let image_tag = format!("{}:{}", config.image_name, config.image_tag);

        let dockerfile_dir_str =
            dockerfile_dir
                .to_str()
                .ok_or_else(|| SshServerError::InvalidUtf8InPath {
                    path: dockerfile_dir.display().to_string(),
                })?;

        let build_output = Command::new("docker")
            .args(["build", "-t", &image_tag, dockerfile_dir_str])
            .output()
            .map_err(|source| SshServerError::DockerCommandFailed {
                command: format!("docker build -t {image_tag} {dockerfile_dir_str}"),
                source,
            })?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&build_output.stdout).to_string();
            return Err(SshServerError::DockerBuildFailed {
                image_name: config.image_name.clone(),
                image_tag: config.image_tag.clone(),
                dockerfile_dir: dockerfile_dir_str.to_string(),
                stdout,
                stderr,
            });
        }

        println!("SSH server Docker image built successfully");

        // Start the container using the built image
        let image = GenericImage::new(&config.image_name, &config.image_tag)
            .with_exposed_port(config.container_port.tcp())
            .with_wait_for(WaitFor::seconds(config.startup_wait_secs));

        let container = image
            .start()
            .await
            .map_err(|source| SshServerError::ContainerStartFailed { source })?;

        // Get the mapped SSH port
        let ssh_port: u16 = container
            .get_host_port_ipv4(config.container_port)
            .await
            .map_err(|source| SshServerError::PortMappingFailed { source })?;

        Ok(Self {
            container,
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port,
            test_username: config.username,
            test_password: config.password,
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
        &self.test_username
    }

    /// Get the test password configured in the container
    #[must_use]
    pub fn test_password(&self) -> &str {
        &self.test_password
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
        &self.test_username
    }

    fn test_password(&self) -> &str {
        &self.test_password
    }
}
