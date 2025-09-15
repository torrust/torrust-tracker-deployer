use std::net::IpAddr;
use tracing::info;

use crate::actions::{DockerValidator, RemoteAction, RemoteActionError};
use crate::command_wrappers::ssh::SshCredentials;

/// Step that validates Docker installation on a remote host
pub struct ValidateDockerInstallationStep {
    ssh_credentials: SshCredentials,
    host_ip: IpAddr,
}

impl ValidateDockerInstallationStep {
    #[must_use]
    pub fn new(ssh_credentials: SshCredentials, host_ip: IpAddr) -> Self {
        Self {
            ssh_credentials,
            host_ip,
        }
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
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(
            stage = "validation",
            component = "docker",
            "Validating Docker installation"
        );

        let docker_ssh_connection = self.ssh_credentials.clone().with_host(self.host_ip);
        let docker_validator = DockerValidator::new(docker_ssh_connection);

        docker_validator.execute(&self.host_ip).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_should_create_validate_docker_installation_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            "test_user".to_string(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let step = ValidateDockerInstallationStep::new(ssh_credentials, host_ip);

        // Test that the step can be created successfully
        assert_eq!(step.host_ip, host_ip);
    }
}
