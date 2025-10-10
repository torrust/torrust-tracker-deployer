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
use crate::shared::ssh::SshConfig;

/// Step that validates Docker installation on a remote host
pub struct ValidateDockerInstallationStep {
    ssh_config: SshConfig,
}

impl ValidateDockerInstallationStep {
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        Self { ssh_config }
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

        let docker_validator = DockerValidator::new(self.ssh_config.clone());

        docker_validator.execute(&self.ssh_config.host_ip()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use crate::shared::ssh::SshCredentials;
    use crate::shared::Username;

    use super::*;

    #[test]
    fn it_should_create_validate_docker_installation_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            Username::new("test_user").unwrap(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(ssh_credentials, host_ip);

        let step = ValidateDockerInstallationStep::new(ssh_config);

        // Test that the step can be created successfully
        assert_eq!(step.ssh_config.host_ip(), host_ip);
    }
}
