//! Docker installation validation step
//!
//! This module provides the `ValidateDockerInstallationStep` which validates that
//! Docker is properly installed and operational on remote hosts. This step ensures
//! the container runtime is ready for application deployment.
//!
//! ## Key Features
//!
//! - Docker installation verification via remote validation
//! - Docker daemon status and functionality checking
//! - Version compatibility verification
//! - Integration with SSH-based remote actions
//!
//! ## Validation Process
//!
//! The step uses the `DockerValidator` remote action to perform comprehensive
//! checks including Docker version, daemon status, and basic functionality
//! to ensure the container environment is properly configured.

use tracing::{info, instrument};

use crate::infrastructure::remote_actions::{DockerValidator, RemoteAction, RemoteActionError};
use crate::shared::ssh::SshConnection;

/// Step that validates Docker installation on a remote host
pub struct ValidateDockerInstallationStep {
    ssh_connection: SshConnection,
}

impl ValidateDockerInstallationStep {
    #[must_use]
    pub fn new(ssh_connection: SshConnection) -> Self {
        Self { ssh_connection }
    }

    /// Execute the Docker installation validation step
    ///
    /// This will validate that Docker is properly installed and running
    /// on the remote host by checking the Docker version and daemon status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * SSH connection to the remote host fails
    /// * Docker validation fails
    /// * The remote action execution fails for any other reason
    ///
    /// # Notes
    ///
    /// - In CI environments with network limitations, Docker installation
    ///   validation may be skipped gracefully
    /// - The validation checks both Docker version and daemon status
    #[instrument(
        name = "validate_docker",
        skip_all,
        fields(step_type = "validation", component = "docker")
    )]
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(component = "docker", "Validating Docker installation");

        let docker_validator = DockerValidator::new(self.ssh_connection.clone());

        docker_validator
            .execute(&self.ssh_connection.host_ip())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use crate::shared::ssh::SshCredentials;

    use super::*;

    #[test]
    fn it_should_create_validate_docker_installation_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            "test_user".to_string(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_connection = ssh_credentials.with_host(host_ip);

        let step = ValidateDockerInstallationStep::new(ssh_connection);

        // Test that the step can be created successfully
        assert_eq!(step.ssh_connection.host_ip(), host_ip);
    }
}
