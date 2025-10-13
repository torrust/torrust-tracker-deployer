//! Error types for SSH server container operations

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with SSH server containers
#[derive(Debug, Error)]
pub enum SshServerError {
    /// SSH server Dockerfile not found at expected location
    #[error(
        "SSH server Dockerfile not found at '{expected_path}'
Tip: Ensure 'docker/ssh-server/Dockerfile' exists in the project root"
    )]
    DockerfileNotFound { expected_path: PathBuf },

    /// Docker build command failed
    #[error("Docker build command failed for image '{image_name}:{image_tag}'
Tip: Run 'docker build -t {image_name}:{image_tag} {dockerfile_dir}' manually to see detailed errors")]
    DockerBuildFailed {
        image_name: String,
        image_tag: String,
        dockerfile_dir: String,
        stdout: String,
        stderr: String,
    },

    /// Failed to start SSH server container
    #[error(
        "Failed to start SSH server container
Tip: Check if Docker daemon is running with 'docker ps'"
    )]
    ContainerStartFailed {
        #[source]
        source: testcontainers::core::error::TestcontainersError,
    },

    /// Failed to get mapped port for SSH container
    #[error(
        "Failed to get mapped port for SSH container
Tip: Verify container is running with 'docker ps'"
    )]
    PortMappingFailed {
        #[source]
        source: testcontainers::core::error::TestcontainersError,
    },

    /// Docker command execution failed
    #[error(
        "Docker command execution failed: {command}
Tip: Verify Docker is installed and accessible: 'docker --version'"
    )]
    DockerCommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// Invalid UTF-8 in Dockerfile path
    #[error(
        "Invalid UTF-8 in Dockerfile path: '{path}'
Tip: Avoid non-ASCII characters in file paths"
    )]
    InvalidUtf8InPath { path: String },
}

impl SshServerError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::RealSshServerContainer;
    ///
    /// # tokio_test::block_on(async {
    /// if let Err(e) = RealSshServerContainer::start().await {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # });
    /// ```
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerfileNotFound { .. } => {
                "Dockerfile Not Found - Detailed Troubleshooting:

1. Verify the Dockerfile exists:
   ls -la docker/ssh-server/Dockerfile

2. Check you're running from project root:
   pwd  # Should show the torrust-tracker-deployer directory

3. If using a custom Dockerfile location:
   - Update the DOCKERFILE_DIR constant in constants.rs
   - Ensure the path is relative to the project root

For more information, see the SSH server documentation."
            }

            Self::DockerBuildFailed { .. } => {
                "Docker Build Failed - Detailed Troubleshooting:

1. Run the build command manually to see full output:
   docker build -t torrust-ssh-server:latest docker/ssh-server

2. Common issues:
   - Check Dockerfile syntax
   - Verify base image is accessible: docker pull ubuntu:22.04
   - Check network connectivity for package downloads
   - Review build logs for specific error messages

3. Check Docker daemon status:
   systemctl status docker  # Linux systemd
   docker info  # General information

4. Try cleaning Docker build cache:
   docker builder prune

For more information, see Docker documentation."
            }

            Self::ContainerStartFailed { .. } => {
                "Container Start Failed - Detailed Troubleshooting:

1. Check if Docker daemon is running:
   docker ps

2. Verify sufficient resources:
   docker system df  # Check disk space
   docker info  # Check memory/CPU limits

3. Check for port conflicts:
   netstat -tlnp | grep :22  # Linux
   ss -tlnp | grep :22  # Alternative
   lsof -i :22  # macOS

4. Review Docker logs:
   docker logs <container_id>

For more information, see testcontainers documentation."
            }

            Self::PortMappingFailed { .. } => {
                "Port Mapping Failed - Detailed Troubleshooting:

1. Verify container is running:
   docker ps

2. Check container port configuration:
   docker port <container_id>

3. Check if the required port is already in use:
   netstat -tlnp  # Linux
   ss -tlnp  # Alternative
   lsof -i :22  # macOS

For more information, see Docker networking documentation."
            }

            Self::DockerCommandFailed { .. } => {
                "Docker Command Execution Failed - Detailed Troubleshooting:

1. Verify Docker is installed:
   docker --version

2. Check Docker daemon is running:
   systemctl status docker  # Linux systemd
   docker ps  # Quick check

3. Verify user permissions:
   groups  # Check if user is in 'docker' group
   sudo usermod -aG docker $USER  # Add user to docker group
   # Log out and log back in for group changes to take effect

4. Try running Docker with sudo (temporary workaround):
   sudo docker ps

For more information, see Docker installation documentation."
            }

            Self::InvalidUtf8InPath { .. } => {
                "Invalid UTF-8 in Path - Detailed Troubleshooting:

1. Check the Dockerfile path contains only valid UTF-8 characters
2. Avoid special characters, emoji, or non-ASCII characters in paths
3. Use ASCII characters for file and directory names

This is typically a configuration or system encoding issue."
            }
        }
    }
}
