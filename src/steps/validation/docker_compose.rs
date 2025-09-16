use tracing::info;

use crate::command_wrappers::ssh::SshConnection;
use crate::remote_actions::{DockerComposeValidator, RemoteAction, RemoteActionError};

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
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(
            stage = "validation",
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

    use crate::command_wrappers::ssh::SshCredentials;

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
