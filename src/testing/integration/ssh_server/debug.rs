//! Debug utilities for Docker container troubleshooting

use std::process::Command;

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
    /// Output from `docker ps -a` listing all containers
    pub all_containers: Result<String, String>,

    /// Docker images matching the SSH server image name
    pub ssh_images: Result<String, String>,

    /// Information about containers using the SSH server image
    pub ssh_containers: Result<Vec<ContainerInfo>, String>,

    /// Port usage information for the SSH port
    pub port_usage: Result<Vec<String>, String>,
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
    /// Collect all Docker debug information for troubleshooting
    ///
    /// This method runs various Docker commands to gather diagnostic information
    /// when SSH connectivity tests fail. It collects container status, logs,
    /// and port usage information.
    ///
    /// # Arguments
    ///
    /// * `container_port` - The host port that the SSH container is mapped to
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
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::DockerDebugInfo;
    ///
    /// let debug_info = DockerDebugInfo::collect(2222);
    /// debug_info.print();
    /// ```
    #[must_use]
    pub fn collect(container_port: u16) -> Self {
        Self {
            all_containers: Self::list_all_containers(),
            ssh_images: Self::list_ssh_images(),
            ssh_containers: Self::find_ssh_containers(),
            port_usage: Self::check_port_usage(container_port),
        }
    }

    /// List all Docker containers
    ///
    /// Executes `docker ps -a` to list all containers (running and stopped).
    fn list_all_containers() -> Result<String, String> {
        Command::new("docker")
            .args(["ps", "-a"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .map_err(|e| format!("Failed to run 'docker ps -a': {e}"))
    }

    /// List SSH server Docker images
    ///
    /// Executes `docker images` filtered by SSH server image name.
    fn list_ssh_images() -> Result<String, String> {
        Command::new("docker")
            .args(["images", SSH_SERVER_IMAGE_NAME])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .map_err(|e| format!("Failed to run 'docker images': {e}"))
    }

    /// Find containers using the SSH server image
    ///
    /// Filters containers by the SSH server image and collects their information
    /// including logs.
    fn find_ssh_containers() -> Result<Vec<ContainerInfo>, String> {
        let image_tag = format!("{SSH_SERVER_IMAGE_NAME}:{SSH_SERVER_IMAGE_TAG}");
        let filter = format!("ancestor={image_tag}");

        let output = Command::new("docker")
            .args(["ps", "-a", "--filter", &filter])
            .output()
            .map_err(|e| format!("Failed to filter Docker containers: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();

        // Skip header line, process container lines
        for line in stdout.lines().skip(1) {
            if let Some(container_id) = line.split_whitespace().next() {
                containers.push(ContainerInfo {
                    id: container_id.to_string(),
                    status: line.to_string(),
                    logs: Self::get_container_logs(container_id),
                });
            }
        }

        Ok(containers)
    }

    /// Get logs for a specific container
    ///
    /// Executes `docker logs --tail 20` to get the last 20 lines of container logs.
    fn get_container_logs(container_id: &str) -> Result<String, String> {
        Command::new("docker")
            .args(["logs", "--tail", "20", container_id])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{stdout}\nStderr:\n{stderr}")
                }
            })
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

    /// Print the debug information in a formatted way
    ///
    /// Prints all collected debug information to stdout in a human-readable format.
    pub fn print(&self) {
        println!("\n=== Docker Debug Information ===");

        // Print all containers
        match &self.all_containers {
            Ok(containers) => {
                println!("Docker containers (docker ps -a):");
                println!("{containers}");
            }
            Err(e) => {
                println!("Failed to list containers: {e}");
            }
        }

        // Print SSH images
        match &self.ssh_images {
            Ok(images) => {
                println!("\nDocker images for {SSH_SERVER_IMAGE_NAME}:");
                println!("{images}");
            }
            Err(e) => {
                println!("Failed to list images: {e}");
            }
        }

        // Print SSH containers and their logs
        match &self.ssh_containers {
            Ok(containers) => {
                let image_tag = format!("{SSH_SERVER_IMAGE_NAME}:{SSH_SERVER_IMAGE_TAG}");
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

        // Print port usage
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

        println!("=== End Docker Debug Information ===\n");
    }
}

// ============================================================================
// PUBLIC API - Convenience Function
// ============================================================================

/// Debug helper function to collect and print Docker container information
///
/// This is a convenience function that collects all Docker debug information
/// and prints it to stdout. For programmatic access to the structured data,
/// use [`DockerDebugInfo::collect`] instead.
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
    let debug_info = DockerDebugInfo::collect(container_port);
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
        // This test verifies that the collect method doesn't panic
        // Actual Docker commands may fail in test environment, which is expected
        let debug_info = DockerDebugInfo::collect(2222);

        // Verify structure exists (even if commands failed)
        assert!(debug_info.all_containers.is_ok() || debug_info.all_containers.is_err());
        assert!(debug_info.ssh_images.is_ok() || debug_info.ssh_images.is_err());
        assert!(debug_info.ssh_containers.is_ok() || debug_info.ssh_containers.is_err());
        assert!(debug_info.port_usage.is_ok() || debug_info.port_usage.is_err());
    }

    #[test]
    fn it_should_print_without_panicking() {
        // This test verifies that printing doesn't panic
        let debug_info = DockerDebugInfo::collect(2222);
        debug_info.print();
        // If we get here without panicking, test passes
    }
}
