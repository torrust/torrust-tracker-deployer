use std::net::IpAddr;
use tracing::info;

use crate::actions::{CloudInitValidator, RemoteAction, RemoteActionError};
use crate::command_wrappers::ssh::SshCredentials;

/// Step that validates cloud-init completion on a remote host
pub struct ValidateCloudInitCompletionStep {
    ssh_credentials: SshCredentials,
    host_ip: IpAddr,
}

impl ValidateCloudInitCompletionStep {
    #[must_use]
    pub fn new(ssh_credentials: SshCredentials, host_ip: IpAddr) -> Self {
        Self {
            ssh_credentials,
            host_ip,
        }
    }

    /// Execute the cloud-init completion validation step
    ///
    /// This will validate that cloud-init has finished running on the remote host
    /// by checking cloud-init status and ensuring all initialization is complete.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * SSH connection to the remote host fails
    /// * Cloud-init validation fails
    /// * The remote action execution fails for any other reason
    ///
    /// # Notes
    ///
    /// - This validation ensures that all cloud-init modules have completed
    /// - Critical for ensuring the system is ready for further configuration
    /// - Checks both cloud-init status and completion markers
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(
            stage = "validation",
            component = "cloud_init",
            "Validating cloud-init completion"
        );

        let cloud_init_ssh_connection = self.ssh_credentials.clone().with_host(self.host_ip);
        let cloud_init_validator = CloudInitValidator::new(cloud_init_ssh_connection);

        cloud_init_validator.execute(&self.host_ip).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_should_create_validate_cloud_init_completion_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            "test_user".to_string(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let step = ValidateCloudInitCompletionStep::new(ssh_credentials, host_ip);

        // Test that the step can be created successfully
        assert_eq!(step.host_ip, host_ip);
    }
}
