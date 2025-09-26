//! Cloud-init completion validation step
//!
//! This module provides the `ValidateCloudInitCompletionStep` which validates
//! that cloud-init has completed successfully on remote instances. This step
//! ensures instances are fully initialized before proceeding with deployment.
//!
//! ## Key Features
//!
//! - Cloud-init completion status verification via remote validation
//! - System initialization readiness checking
//! - Integration with SSH-based remote actions
//! - Comprehensive error handling for initialization failures
//!
//! ## Validation Process
//!
//! The step uses the `CloudInitValidator` remote action to check cloud-init
//! status and ensure all system initialization tasks have completed successfully,
//! providing confidence that the instance is ready for configuration and
//! software installation.

use tracing::{info, instrument};

use crate::infrastructure::remote_actions::{CloudInitValidator, RemoteAction, RemoteActionError};
use crate::shared::ssh::SshConnection;

/// Step that validates cloud-init completion on a remote host
pub struct ValidateCloudInitCompletionStep {
    ssh_connection: SshConnection,
}

impl ValidateCloudInitCompletionStep {
    #[must_use]
    pub fn new(ssh_connection: SshConnection) -> Self {
        Self { ssh_connection }
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
    #[instrument(
        name = "validate_cloud_init",
        skip_all,
        fields(step_type = "validation", component = "cloud_init")
    )]
    pub async fn execute(&self) -> Result<(), RemoteActionError> {
        info!(component = "cloud_init", "Validating cloud-init completion");

        let cloud_init_validator = CloudInitValidator::new(self.ssh_connection.clone());

        cloud_init_validator
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
    use crate::shared::Username;

    use super::*;

    #[test]
    fn it_should_create_validate_cloud_init_completion_step() {
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("test_key"),
            PathBuf::from("test_key.pub"),
            Username::new("test_user").unwrap(),
        );
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_connection = SshConnection::with_default_port(ssh_credentials, host_ip);

        let step = ValidateCloudInitCompletionStep::new(ssh_connection);

        // Test that the step can be created successfully
        assert_eq!(step.ssh_connection.host_ip(), host_ip);
    }
}
