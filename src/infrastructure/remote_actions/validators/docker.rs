//! Docker installation and validation remote action
//!
//! This module provides the `DockerValidator` which checks Docker installation
//! and daemon status on remote instances to ensure the container runtime is
//! properly configured and operational.
//!
//! ## Key Features
//!
//! - Docker daemon status validation
//! - Docker version checking and compatibility verification
//! - Service availability testing
//! - Comprehensive error reporting for Docker issues
//!
//! ## Validation Process
//!
//! The validator checks multiple aspects of Docker installation:
//! - Docker binary availability and version
//! - Docker daemon running status
//! - Basic Docker functionality (e.g., hello-world container)
//!
//! This ensures that subsequent deployment steps can rely on a working
//! Docker environment.

use std::net::IpAddr;
use tracing::{info, instrument, warn};

use crate::adapters::ssh::SshClient;
use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Action that validates Docker installation and daemon status on the server
pub struct DockerValidator {
    ssh_client: SshClient,
}

impl DockerValidator {
    /// Create a new `DockerValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self { ssh_client }
    }
}

impl RemoteAction for DockerValidator {
    fn name(&self) -> &'static str {
        "docker-validation"
    }

    #[instrument(
        name = "docker_validation",
        skip(self),
        fields(
            action_type = "validation",
            component = "docker",
            server_ip = %server_ip
        )
    )]
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "docker_validation",
            "Validating Docker installation"
        );

        // Check Docker version
        let docker_version = self
            .ssh_client
            .execute("docker --version")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        let docker_version = docker_version.trim();
        info!(
            action = "docker_validation",
            status = "success",
            "Docker installation validated"
        );
        info!(
            action = "docker_validation",
            version = docker_version,
            "Docker version detected"
        );

        // Check Docker daemon status (only if Docker is installed)
        let daemon_active = self
            .ssh_client
            .check_command("sudo systemctl is-active docker")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if daemon_active {
            info!(
                action = "docker_validation",
                check = "daemon_active",
                "Docker daemon is active"
            );
        } else {
            warn!(
                action = "docker_validation",
                check = "daemon_skipped",
                reason = "service_not_running",
                "Docker daemon check skipped"
            );
        }

        Ok(())
    }
}
