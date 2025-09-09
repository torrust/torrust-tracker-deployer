use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tracing::info;

use crate::actions::RemoteAction;
use crate::ssh::SshClient;

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
    /// * `verbose` - Whether to enable verbose SSH output
    #[must_use]
    pub fn new(ssh_key_path: &Path, username: &str, verbose: bool) -> Self {
        let ssh_client = SshClient::new(ssh_key_path, username, verbose);
        Self { ssh_client }
    }
}

impl RemoteAction for CloudInitValidator {
    fn name(&self) -> &'static str {
        "cloud-init-validation"
    }

    async fn execute(&self, server_ip: &str) -> Result<()> {
        info!("🔍 Validating cloud-init completion...");

        // Check cloud-init status
        let status_output = self
            .ssh_client
            .execute(server_ip, "cloud-init status")
            .context("Failed to check cloud-init status")?;

        if !status_output.contains("status: done") {
            return Err(anyhow!(
                "Cloud-init status is not 'done': {}",
                status_output
            ));
        }

        // Check for completion marker file
        let marker_exists = self
            .ssh_client
            .check_command(server_ip, "test -f /var/lib/cloud/instance/boot-finished")
            .context("Failed to check cloud-init completion marker")?;

        if !marker_exists {
            return Err(anyhow!("Cloud-init completion marker file not found"));
        }

        info!("✅ Cloud-init validation passed");
        info!("   ✓ Cloud-init status is 'done'");
        info!("   ✓ Completion marker file exists");

        Ok(())
    }
}
