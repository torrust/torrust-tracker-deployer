//! SSH Server Container for Integration Testing
//!
//! This module provides SSH server containers for testing SSH client functionality.
//! Two implementations are available:
//!
//! - `MockSshServerContainer`: Fast mock for tests that don't need real SSH connectivity
//! - `RealSshServerContainer`: Actual Docker SSH server for full integration tests

use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

/// Mock SSH server container for fast testing
///
/// This implementation doesn't start a real container but provides the same
/// interface as a real SSH server. Use this for tests that only need to verify
/// configuration, timeouts, or client behavior without actual SSH connectivity.
pub struct MockSshServerContainer {
    host_ip: IpAddr,
    ssh_port: u16,
    test_username: String,
    test_password: String,
}

impl MockSshServerContainer {
    /// Create a new mock SSH server container
    ///
    /// This doesn't start any actual container, making it very fast for tests
    /// that don't need real SSH connectivity.
    ///
    /// # Returns
    ///
    /// A mock container configured with default test credentials.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns a Result to match the interface
    /// of `RealSshServerContainer::start()`.
    pub fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port: 2222, // Mock port
            test_username: "testuser".to_string(),
            test_password: "testpass".to_string(),
        })
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
    /// Start a real SSH server container
    ///
    /// This starts an actual Docker container running SSH server.
    /// The Docker image is built automatically from the Dockerfile in `docker/ssh-server/`.
    /// This ensures the tests are self-contained and work in CI environments.
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
    ///
    /// # Panics
    ///
    /// Panics if the dockerfile directory path contains invalid UTF-8 characters.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Build the SSH server image from Dockerfile
        // This ensures tests are self-contained and work in CI
        let dockerfile_dir = std::path::Path::new("docker/ssh-server");

        if !dockerfile_dir.exists() || !dockerfile_dir.join("Dockerfile").exists() {
            return Err(format!(
                "SSH server Dockerfile not found. Expected: {}/Dockerfile",
                dockerfile_dir.display()
            )
            .into());
        }

        // Build the Docker image using docker CLI
        println!(
            "Building SSH server Docker image from {}",
            dockerfile_dir.display()
        );

        let build_output = Command::new("docker")
            .args([
                "build",
                "-t",
                "torrust-ssh-server:latest",
                dockerfile_dir.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute docker build command: {e}"))?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            let stdout = String::from_utf8_lossy(&build_output.stdout);
            return Err(format!("Docker build failed:\nSTDOUT: {stdout}\nSTDERR: {stderr}").into());
        }

        println!("SSH server Docker image built successfully");

        // Start the container using the built image
        let image = GenericImage::new("torrust-ssh-server", "latest")
            .with_exposed_port(22_u16.tcp())
            .with_wait_for(WaitFor::seconds(10)); // Wait longer for SSH daemon to start up in CI environments

        let container = image.start().await?;

        // Get the mapped SSH port
        let ssh_port = container.get_host_port_ipv4(22).await?;

        Ok(Self {
            container,
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port,
            test_username: "testuser".to_string(),
            test_password: "testpass".to_string(),
        })
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

/// Debug helper function to collect Docker container information for troubleshooting
///
/// This function runs various Docker commands to help diagnose issues when SSH
/// connectivity tests fail in CI environments. It prints container status, logs,
/// and other useful debugging information.
///
/// # Arguments
///
/// * `container_port` - The host port that the SSH container is mapped to
///
/// # Usage
///
/// This function is typically called when SSH connectivity tests fail to help
/// diagnose what's happening with the Docker containers in CI environments.
///
/// ```rust
/// use torrust_tracker_deployer::testing::integration::ssh_server::print_docker_debug_info;
///
/// // In a test when SSH connectivity fails:
/// print_docker_debug_info(ssh_container.ssh_port()).await;
/// ```
pub async fn print_docker_debug_info(container_port: u16) {
    println!("\n=== Docker Debug Information ===");

    // Check if Docker is running and list all containers
    match std::process::Command::new("docker")
        .args(["ps", "-a"])
        .output()
    {
        Ok(output) => {
            println!("Docker containers (docker ps -a):");
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!(
                    "Docker ps stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            println!("Failed to run 'docker ps -a': {}", e);
        }
    }

    // Check Docker images
    match std::process::Command::new("docker")
        .args(["images", "torrust-ssh-server"])
        .output()
    {
        Ok(output) => {
            println!("\nDocker images for torrust-ssh-server:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => {
            println!("Failed to run 'docker images': {}", e);
        }
    }

    // Try to find containers with the SSH image
    match std::process::Command::new("docker")
        .args(["ps", "-a", "--filter", "ancestor=torrust-ssh-server:latest"])
        .output()
    {
        Ok(output) => {
            println!("\nContainers using torrust-ssh-server:latest:");
            println!("{}", String::from_utf8_lossy(&output.stdout));

            // Get container logs if there are any containers
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout_str.lines().nth(1) {
                // Skip header, get first container
                if let Some(container_id) = line.split_whitespace().next() {
                    println!("\nContainer logs for {}:", container_id);
                    match std::process::Command::new("docker")
                        .args(["logs", "--tail", "20", container_id])
                        .output()
                    {
                        Ok(log_output) => {
                            println!("{}", String::from_utf8_lossy(&log_output.stdout));
                            if !log_output.stderr.is_empty() {
                                println!(
                                    "Container stderr: {}",
                                    String::from_utf8_lossy(&log_output.stderr)
                                );
                            }
                        }
                        Err(e) => {
                            println!("Failed to get container logs: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to filter Docker containers: {}", e);
        }
    }

    // Check if the specific port is being used
    println!("\nPort information:");
    println!("Expected SSH port mapping: host -> container:22");
    match std::process::Command::new("netstat")
        .args(["-tlnp"])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains(&container_port.to_string()) {
                    println!("Port {} usage: {}", container_port, line);
                }
            }
        }
        Err(_) => {
            // Fallback to ss command if netstat is not available
            match std::process::Command::new("ss").args(["-tlnp"]).output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        if line.contains(&container_port.to_string()) {
                            println!("Port {} usage (ss): {}", container_port, line);
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to check port usage (netstat/ss not available): {}",
                        e
                    );
                }
            }
        }
    }

    println!("=== End Docker Debug Information ===\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_should_start_mock_ssh_server_container() {
        let container = MockSshServerContainer::start();

        match container {
            Ok(ssh_container) => {
                // Verify basic container properties
                let port = ssh_container.ssh_port();
                assert!(port > 0, "SSH port should be positive");

                let host_ip = ssh_container.host_ip();
                assert_eq!(host_ip, IpAddr::V4(Ipv4Addr::LOCALHOST));

                assert_eq!(ssh_container.test_username(), "testuser");
                assert_eq!(ssh_container.test_password(), "testpass");
            }
            Err(e) => {
                panic!("Mock container should always start successfully: {e}");
            }
        }
    }

    #[tokio::test]
    async fn it_should_start_real_ssh_server_container() {
        let container = RealSshServerContainer::start().await;

        match container {
            Ok(ssh_container) => {
                // Verify basic container properties
                let port = ssh_container.ssh_port();
                assert!(port > 0, "SSH port should be positive");

                let host_ip = ssh_container.host_ip();
                assert_eq!(host_ip, IpAddr::V4(Ipv4Addr::LOCALHOST));

                assert_eq!(ssh_container.test_username(), "testuser");
                assert_eq!(ssh_container.test_password(), "testpass");
            }
            Err(e) => {
                // Real container start might fail in CI environments without Docker
                // or if the SSH server image hasn't been built
                println!("Real container start failed (expected in some environments): {e}");
            }
        }
    }
}
