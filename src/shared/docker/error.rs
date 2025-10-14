//! Docker-specific error types

use thiserror::Error;

use crate::shared::command::CommandError;

/// Errors that can occur during Docker operations
#[derive(Debug, Error)]
pub enum DockerError {
    /// Docker build command failed
    #[error(
        "Docker build failed for image '{image}'
Tip: Run 'docker build -t {image} <path>' manually to see detailed output"
    )]
    BuildFailed {
        image: String,
        #[source]
        source: CommandError,
    },

    /// Failed to list Docker images
    #[error(
        "Failed to list Docker images
Tip: Verify Docker is installed and running: 'docker ps'"
    )]
    ListImagesFailed(#[source] CommandError),

    /// Failed to list Docker containers
    #[error(
        "Failed to list Docker containers
Tip: Verify Docker is installed and running: 'docker ps'"
    )]
    ListContainersFailed(#[source] CommandError),

    /// Failed to get container logs
    #[error(
        "Failed to get logs for container '{container_id}'
Tip: Verify the container exists: 'docker ps -a'"
    )]
    GetLogsFailed {
        container_id: String,
        #[source]
        source: CommandError,
    },
}

impl DockerError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    ///
    /// # fn example() {
    /// let docker = DockerClient::new();
    ///
    /// if let Err(e) = docker.build_image(".", "my-app", "latest") {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::BuildFailed { .. } => {
                "Docker Build Failed - Detailed Troubleshooting:

1. Run the build command manually to see full output:
   docker build -t <image>:<tag> <path>

2. Check Dockerfile syntax and verify base image availability:
   - Ensure FROM statement uses a valid base image
   - Check that the base image exists locally or can be pulled

3. Verify network connectivity for package downloads:
   - Test internet connection: ping google.com
   - Check if corporate proxy is blocking Docker Hub
   - Try with --network=host if behind firewall

4. Check Docker daemon logs for system-level issues:
   journalctl -u docker  # Linux systemd
   docker info  # General Docker information

5. Try rebuilding without cache to avoid stale layers:
   docker build --no-cache -t <image>:<tag> <path>

6. Verify sufficient disk space:
   df -h  # Check available space
   docker system df  # Check Docker disk usage
   docker system prune  # Clean up unused resources

For more information, see Docker documentation: https://docs.docker.com/engine/reference/commandline/build/"
            }

            Self::ListImagesFailed(_) | Self::ListContainersFailed(_) => {
                "Docker List Command Failed - Detailed Troubleshooting:

1. Verify Docker is installed and check version:
   docker --version

2. Check if Docker daemon is running:
   docker ps  # Quick check
   systemctl status docker  # Linux systemd
   docker info  # Detailed daemon information

3. Verify user permissions (avoid running as root):
   groups  # Check if user is in 'docker' group

   If not in docker group, add yourself:
   sudo usermod -aG docker $USER
   # Log out and log back in for changes to take effect

4. Try with sudo as temporary workaround (not recommended for regular use):
   sudo docker ps

5. Check Docker socket permissions:
   ls -la /var/run/docker.sock
   # Should show: srw-rw---- 1 root docker

6. Restart Docker daemon if needed:
   sudo systemctl restart docker  # Linux systemd

For more information, see Docker installation guide: https://docs.docker.com/engine/install/"
            }

            Self::GetLogsFailed { .. } => {
                "Docker Logs Failed - Detailed Troubleshooting:

1. Verify the container exists:
   docker ps -a  # List all containers (including stopped)

2. Check if container ID or name is correct:
   - Container IDs can be abbreviated (first 12 characters)
   - Container names are case-sensitive

3. Try viewing logs with Docker CLI directly:
   docker logs <container-id>
   docker logs --tail 100 <container-id>  # Last 100 lines
   docker logs --follow <container-id>  # Follow log output

4. Check if container was removed:
   - Containers may be automatically removed with --rm flag
   - Check if container exists in stopped state

5. Verify Docker daemon is responsive:
   docker ps  # Should list running containers

If the container doesn't exist, it may have been removed or never created successfully."
            }
        }
    }
}
