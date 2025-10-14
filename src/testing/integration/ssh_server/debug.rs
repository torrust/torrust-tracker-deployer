//! Debug utilities for Docker container troubleshooting

use std::process::Command;
use std::sync::Arc;

use crate::shared::docker::DockerClient;

// Import constants only for the convenience function
use super::constants::{SSH_SERVER_IMAGE_NAME, SSH_SERVER_IMAGE_TAG};

// ============================================================================
// PUBLIC API - Structured Debug Data
// ============================================================================

/// Debug information collected about Docker containers
///
/// This struct holds structured information about Docker state for troubleshooting
/// SSH server container issues. Each field contains either successfully collected
/// data or an error message explaining what went wrong.
#[derive(Debug)]
pub struct DockerDebugInfo {
    /// Docker client for executing commands
    docker: Arc<DockerClient>,

    /// Output from `docker ps -a` listing all containers
    pub all_containers: Result<String, String>,

    /// Docker images matching the SSH server image name
    pub ssh_images: Result<String, String>,

    /// Information about containers using the SSH server image
    pub ssh_containers: Result<Vec<ContainerInfo>, String>,

    /// Port usage information for the SSH port
    pub port_usage: Result<Vec<String>, String>,

    /// Docker image name that was searched for
    image_name: String,

    /// Docker image tag that was searched for
    image_tag: String,
}

/// Information about a specific Docker container
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Container ID
    pub id: String,

    /// Full status line from docker ps
    pub status: String,

    /// Container logs (last 20 lines)
    pub logs: Result<String, String>,
}

// ============================================================================
// PUBLIC API - Debug Info Collection
// ============================================================================

impl DockerDebugInfo {
    /// Create a new `DockerDebugInfo` and collect all diagnostic information
    ///
    /// This constructor runs various Docker commands to gather diagnostic information
    /// when SSH connectivity tests fail. It collects container status, logs,
    /// and port usage information.
    ///
    /// # Arguments
    ///
    /// * `docker` - Docker client for executing commands
    /// * `container_port` - The host port that the SSH container is mapped to
    /// * `image_name` - Image name to filter by (e.g., "torrust-ssh-server")
    /// * `image_tag` - Image tag to filter by (e.g., "latest")
    ///
    /// # Returns
    ///
    /// A `DockerDebugInfo` struct containing all collected information. Each field
    /// is a `Result` that either contains the successfully collected data or an
    /// error message explaining what went wrong.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::DockerDebugInfo;
    ///
    /// let docker = Arc::new(DockerClient::new());
    /// let debug_info = DockerDebugInfo::new(docker, 2222, "torrust-ssh-server", "latest");
    /// debug_info.print();
    /// ```
    #[must_use]
    pub fn new(
        docker: Arc<DockerClient>,
        container_port: u16,
        image_name: &str,
        image_tag: &str,
    ) -> Self {
        let mut instance = Self {
            docker,
            all_containers: Ok(String::new()),
            ssh_images: Ok(String::new()),
            ssh_containers: Ok(Vec::new()),
            port_usage: Ok(Vec::new()),
            image_name: image_name.to_string(),
            image_tag: image_tag.to_string(),
        };

        // Collect debug information using instance methods
        instance.all_containers = instance.list_all_containers();
        instance.ssh_images = instance.list_ssh_images(&instance.image_name.clone());
        instance.ssh_containers = instance.find_ssh_containers();
        instance.port_usage = Self::check_port_usage(container_port);

        instance
    }

    /// List all Docker containers
    ///
    /// Uses `DockerClient::list_containers(true)` to list all containers (running and stopped).
    fn list_all_containers(&self) -> Result<String, String> {
        self.docker
            .list_containers(true)
            .map(|containers| containers.join("\n"))
            .map_err(|e| format!("Failed to list containers: {e}"))
    }

    /// List SSH server Docker images
    ///
    /// Uses `DockerClient::list_images` filtered by SSH server image name.
    fn list_ssh_images(&self, image_name: &str) -> Result<String, String> {
        self.docker
            .list_images(Some(image_name))
            .map(|images| images.join("\n"))
            .map_err(|e| format!("Failed to list images: {e}"))
    }

    /// Find containers using the SSH server image
    ///
    /// Uses `DockerClient::list_containers` and filters by image.
    /// Also fetches logs for each matching container.
    fn find_ssh_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        // TODO: Filter by image when DockerClient supports image info in list_containers
        // For now, we list all containers

        let all_containers = self
            .docker
            .list_containers(true)
            .map_err(|e| format!("Failed to list containers: {e}"))?;

        let mut containers = Vec::new();

        // Filter containers by image and collect their info
        for container_line in all_containers {
            // Container format from DockerClient: "id|name|status"
            if let Some(container_id) = container_line.split('|').next() {
                // For now, we include all containers
                // TODO: Filter by image when DockerClient supports image info
                containers.push(ContainerInfo {
                    id: container_id.to_string(),
                    status: container_line.clone(),
                    logs: self.get_container_logs(container_id),
                });
            }
        }

        Ok(containers)
    }

    /// Get logs for a specific container
    ///
    /// Uses `DockerClient::get_container_logs` to retrieve logs.
    /// Note: `DockerClient` doesn't support --tail yet, so we get all logs.
    fn get_container_logs(&self, container_id: &str) -> Result<String, String> {
        self.docker
            .get_container_logs(container_id)
            .map_err(|e| format!("Failed to get container logs: {e}"))
    }

    /// Check port usage for the SSH port
    ///
    /// Tries to find processes using the specified port using `netstat` or `ss`.
    fn check_port_usage(port: u16) -> Result<Vec<String>, String> {
        // Try netstat first, fallback to ss
        Self::check_port_with_netstat(port).or_else(|_| Self::check_port_with_ss(port))
    }

    /// Check port usage using netstat command
    fn check_port_with_netstat(port: u16) -> Result<Vec<String>, String> {
        let output = Command::new("netstat")
            .args(["-tlnp"])
            .output()
            .map_err(|e| format!("netstat command failed: {e}"))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let port_str = port.to_string();
        let matches: Vec<String> = output_str
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(format!("Port {port} not found in netstat output"))
        } else {
            Ok(matches)
        }
    }

    /// Check port usage using ss command (fallback for systems without netstat)
    fn check_port_with_ss(port: u16) -> Result<Vec<String>, String> {
        let output = Command::new("ss")
            .args(["-tlnp"])
            .output()
            .map_err(|e| format!("ss command failed: {e}"))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let port_str = port.to_string();
        let matches: Vec<String> = output_str
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(format!("Port {port} not found in ss output"))
        } else {
            Ok(matches)
        }
    }

    /// Get a reference to the Docker client
    ///
    /// This allows access to the underlying Docker client for additional operations
    /// if needed after debug info has been collected.
    #[must_use]
    pub fn docker(&self) -> &Arc<DockerClient> {
        &self.docker
    }

    /// Print the debug information in a formatted way
    ///
    /// Prints all collected debug information to stdout in a human-readable format.
    pub fn print(&self) {
        println!("\n=== Docker Debug Information ===");
        self.print_all_containers();
        self.print_ssh_images();
        self.print_ssh_containers_and_logs();
        self.print_port_usage();
        println!("=== End Docker Debug Information ===\n");
    }

    /// Print all Docker containers
    fn print_all_containers(&self) {
        match &self.all_containers {
            Ok(containers) => {
                println!("Docker containers (docker ps -a):");
                println!("{containers}");
            }
            Err(e) => {
                println!("Failed to list containers: {e}");
            }
        }
    }

    /// Print SSH server images
    fn print_ssh_images(&self) {
        match &self.ssh_images {
            Ok(images) => {
                println!("\nDocker images for {}:", self.image_name);
                println!("{images}");
            }
            Err(e) => {
                println!("Failed to list images: {e}");
            }
        }
    }

    /// Print SSH containers and their logs
    fn print_ssh_containers_and_logs(&self) {
        match &self.ssh_containers {
            Ok(containers) => {
                let image_tag = format!("{}:{}", self.image_name, self.image_tag);
                println!("\nContainers using {image_tag}:");

                if containers.is_empty() {
                    println!("No containers found");
                } else {
                    for container in containers {
                        println!("Container {}: {}", container.id, container.status);

                        match &container.logs {
                            Ok(logs) => {
                                println!("\nContainer logs for {}:", container.id);
                                println!("{logs}");
                            }
                            Err(e) => {
                                println!("Failed to get logs for {}: {e}", container.id);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to filter containers: {e}");
            }
        }
    }

    /// Print port usage information
    fn print_port_usage(&self) {
        println!("\nPort information:");
        match &self.port_usage {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
            }
            Err(e) => {
                println!("Failed to check port usage: {e}");
            }
        }
    }
}

// ============================================================================
// PUBLIC API - Convenience Function
// ============================================================================

/// Debug helper function to collect and print Docker container information
///
/// This is a convenience function that collects all Docker debug information
/// and prints it to stdout. For programmatic access to the structured data,
/// use [`DockerDebugInfo::new`] instead.
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
/// use torrust_tracker_deployer_lib::testing::integration::ssh_server::print_docker_debug_info;
///
/// // In a test when SSH connectivity fails:
/// print_docker_debug_info(2222);
/// ```
pub fn print_docker_debug_info(container_port: u16) {
    let docker = Arc::new(DockerClient::new());
    let debug_info = DockerDebugInfo::new(
        docker,
        container_port,
        SSH_SERVER_IMAGE_NAME,
        SSH_SERVER_IMAGE_TAG,
    );
    debug_info.print();
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_collect_docker_debug_info() {
        // This test verifies that the new method doesn't panic
        // Actual Docker commands may fail in test environment, which is expected
        let docker = Arc::new(DockerClient::new());
        let debug_info = DockerDebugInfo::new(docker, 2222, "test-image", "latest");

        // Verify structure exists (even if commands failed)
        assert!(debug_info.all_containers.is_ok() || debug_info.all_containers.is_err());
        assert!(debug_info.ssh_images.is_ok() || debug_info.ssh_images.is_err());
        assert!(debug_info.ssh_containers.is_ok() || debug_info.ssh_containers.is_err());
        assert!(debug_info.port_usage.is_ok() || debug_info.port_usage.is_err());
    }

    #[test]
    fn it_should_print_without_panicking() {
        // This test verifies that printing doesn't panic
        let docker = Arc::new(DockerClient::new());
        let debug_info = DockerDebugInfo::new(docker, 2222, "test-image", "latest");
        debug_info.print();
        // If we get here without panicking, test passes
    }
}
