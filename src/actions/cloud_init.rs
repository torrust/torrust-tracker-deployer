use std::net::IpAddr;
use std::path::Path;
use tracing::info;

use crate::actions::{RemoteAction, RemoteActionError};
use crate::command_wrappers::ssh::SshClient;

/// Action that checks if cloud-init has completed successfully on the server
pub struct CloudInitValidator {
    ssh_client: SshClient,
}

impl CloudInitValidator {
    /// Create a new `CloudInitValidator` with the specified SSH key
    ///
    /// # Arguments
    /// * `ssh_key_path` - Path to the SSH private key file
    /// * `username` - SSH username to use for connections
    /// * `host_ip` - IP address of the target host
    /// * `verbose` - Whether to enable verbose SSH output
    #[must_use]
    pub fn new(ssh_key_path: &Path, username: &str, host_ip: IpAddr, verbose: bool) -> Self {
        let ssh_client = SshClient::new(ssh_key_path, username, host_ip, verbose);
        Self { ssh_client }
    }
}

impl RemoteAction for CloudInitValidator {
    fn name(&self) -> &'static str {
        "cloud-init-validation"
    }

    async fn execute(&self, _server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!("🔍 Validating cloud-init completion...");

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

        info!("✅ Cloud-init validation passed");
        info!("   ✓ Cloud-init status is 'done'");
        info!("   ✓ Completion marker file exists");

        Ok(())
    }
}
