//! Docker Compose installation validation step
//!
//! This module provides the `ValidateDockerComposeInstallationStep` which validates
//! that Docker Compose is properly installed and operational on remote hosts.
//! This step ensures the container orchestration tool is ready for deployment.
//!
//! ## Key Features
//!
//! - Docker Compose installation verification via remote validation
//! - Version compatibility checking and functionality testing
//! - Integration with Docker engine validation
//! - Integration with SSH-based remote actions
//!
//! ## Validation Process
//!
//! The step uses the `DockerComposeValidator` remote action to perform
//! comprehensive checks including version verification, basic functionality
//! testing, and integration with the Docker engine to ensure complete
//! container orchestration capabilities.

use tracing::{info, instrument};

use crate::infrastructure::remote_actions::{
    DockerComposeValidator, RemoteAction, RemoteActionError,
};
use crate::shared::ssh::SshConnection;

/// Step that validates Docker Compose installation on a remote host
pub struct ValidateDockerComposeInstallationStep {
    ssh_connection: SshConnection,
}

impl ValidateDockerComposeInstallationStep {
    #[must_use]
    pub fn new(ssh_connection: SshConnection) -> Self {
        Self { ssh_connection }
    }

    /// Execute the Docker Compose installation validation step
    ///
    /// This will validate that Docker Compose is properly installed and accessible
    /// on the remote host by checking the Docker Compose version.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * SSH connection to the remote host fails
    /// * Docker Compose validation fails
    /// * The remote action execution fails for any other reason
    ///
    /// # Notes
    ///
    /// - In CI environments with network limitations, Docker Compose installation
    ///   validation may be skipped gracefully
    /// - The validation checks Docker Compose version and availability
    #[instrument(
        name = "validate_docker_compose",
        skip_all,
        fields(step_type = "validation", component = "docker_compose")
    )]
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(
            component = "docker_compose",
            "Validating Docker Compose installation"
        );

        let docker_compose_validator = DockerComposeValidator::new(self.ssh_connection.clone());

        docker_compose_validator
            .execute(&self.ssh_connection.host_ip)
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
    fn it_should_create_validate_docker_compose_installation_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            "test_user".to_string(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_connection = ssh_credentials.with_host(host_ip);

        let step = ValidateDockerComposeInstallationStep::new(ssh_connection);

        // Test that the step can be created successfully
        assert_eq!(step.ssh_connection.host_ip, host_ip);
    }
}
