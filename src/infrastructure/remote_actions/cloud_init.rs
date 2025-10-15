//! Cloud-init validation remote action
//!
//! This module provides the `CloudInitValidator` which checks whether cloud-init
//! has completed successfully on remote instances. This is crucial for ensuring
//! that instances are fully initialized before attempting further configuration.
//!
//! ## Key Features
//!
//! - Remote cloud-init status checking via SSH
//! - Validation of cloud-init completion status
//! - Comprehensive error reporting for initialization failures
//! - Integration with the remote action framework
//!
//! ## Usage Context
//!
//! This action is typically used early in the deployment workflow to verify
//! that newly provisioned instances have completed their initial setup process
//! before proceeding with software installation or configuration.

use std::net::IpAddr;
use tracing::{info, instrument};

use crate::adapters::ssh::SshClient;
use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Action that checks if cloud-init has completed successfully on the server
pub struct CloudInitValidator {
    ssh_client: SshClient,
}

impl CloudInitValidator {
    /// Create a new `CloudInitValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self { ssh_client }
    }
}

impl RemoteAction for CloudInitValidator {
    fn name(&self) -> &'static str {
        "cloud-init-validation"
    }

    #[instrument(
        name = "cloud_init_validation",
        skip(self),
        fields(
            action_type = "validation",
            component = "cloud_init",
            server_ip = %server_ip
        )
    )]
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "cloud_init_validation",
            "Validating cloud-init completion"
        );

        // Check if cloud-init is installed
        let cloud_init_installed = self
            .ssh_client
            .check_command("command -v cloud-init")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if cloud_init_installed {
            info!(
                action = "cloud_init_validation",
                check = "installation",
                "Cloud-init is installed, checking status"
            );

            // Check cloud-init status only if cloud-init is installed
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

            info!(
                action = "cloud_init_validation",
                check = "status_done",
                "Cloud-init status is 'done'"
            );
        } else {
            info!(
                action = "cloud_init_validation",
                check = "installation",
                "Cloud-init is not installed, skipping status check (container environment)"
            );
        }

        // Check for completion marker file (applies to both VM and container environments)
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
            check = "completion_marker",
            "Completion marker file exists"
        );

        info!(
            action = "cloud_init_validation",
            status = "success",
            "Cloud-init validation passed"
        );

        Ok(())
    }
}
