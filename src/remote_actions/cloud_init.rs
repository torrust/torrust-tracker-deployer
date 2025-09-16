use std::net::IpAddr;
use tracing::info;

use crate::command_wrappers::ssh::SshClient;
use crate::command_wrappers::ssh::SshConnection;
use crate::remote_actions::{RemoteAction, RemoteActionError};

/// Action that checks if cloud-init has completed successfully on the server
pub struct CloudInitValidator {
    ssh_client: SshClient,
}

impl CloudInitValidator {
    /// Create a new `CloudInitValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_connection` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_connection: SshConnection) -> Self {
        let ssh_client = SshClient::new(ssh_connection);
        Self { ssh_client }
    }
}

impl RemoteAction for CloudInitValidator {
    fn name(&self) -> &'static str {
        "cloud-init-validation"
    }

    async fn execute(&self, _server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "cloud_init_validation",
            "Validating cloud-init completion"
        );

        // Check cloud-init status
        let status_output = self
            .ssh_client
            .execute("cloud-init status")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if !status_output.contains("status: done") {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!("Cloud-init status is not 'done': {status_output}"),
            });
        }

        // Check for completion marker file
        let marker_exists = self
            .ssh_client
            .check_command("test -f /var/lib/cloud/instance/boot-finished")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if !marker_exists {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: "Cloud-init completion marker file not found".to_string(),
            });
        }

        info!(
            action = "cloud_init_validation",
            status = "success",
            "Cloud-init validation passed"
        );
        info!(
            action = "cloud_init_validation",
            check = "status_done",
            "Cloud-init status is 'done'"
        );
        info!(
            action = "cloud_init_validation",
            check = "completion_marker",
            "Completion marker file exists"
        );

        Ok(())
    }
}
