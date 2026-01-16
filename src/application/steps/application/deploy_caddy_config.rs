//! Caddy configuration deployment step
//!
//! This module provides the `DeployCaddyConfigStep` which handles deployment
//! of the Caddyfile configuration file to remote hosts via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Creates Caddy storage directories on remote host
//! - Deploys Caddyfile from build directory to remote host
//! - Sets appropriate ownership and permissions
//! - Verifies successful deployment with assertions
//! - Only executes when HTTPS/TLS is configured in environment
//!
//! ## Deployment Flow
//!
//! 1. Create storage directories (/opt/torrust/storage/caddy/{etc,data,config})
//! 2. Copy Caddyfile from build directory to remote host
//! 3. Set file permissions (0644) and ownership
//! 4. Verify file exists and has correct properties
//!
//! ## File Locations
//!
//! - **Source**: `{build_dir}/caddy/Caddyfile`
//! - **Destination**: `/opt/torrust/storage/caddy/etc/Caddyfile`
//! - **Container Mount**: Mounted as `/etc/caddy/Caddyfile`

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that deploys Caddy configuration to a remote host via Ansible
///
/// This step creates the necessary storage directories and copies the rendered
/// Caddyfile configuration file from the build directory to the remote host's
/// Caddy configuration directory.
pub struct DeployCaddyConfigStep {
    ansible_client: Arc<AnsibleClient>,
}

impl DeployCaddyConfigStep {
    /// Create a new Caddy configuration deployment step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the configuration deployment
    ///
    /// Creates Caddy storage directories and runs the Ansible playbook that
    /// deploys the Caddyfile configuration file.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Directory creation fails
    /// - File copying fails
    /// - Permission setting fails
    /// - Verification assertions fail
    #[instrument(
        name = "deploy_caddy_config",
        skip_all,
        fields(step_type = "deployment", component = "caddy", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "deploy_caddy_config",
            action = "deploy_file",
            "Deploying Caddy configuration to remote host"
        );

        match self.ansible_client.run_playbook("deploy-caddy-config", &[]) {
            Ok(_) => {
                info!(
                    step = "deploy_caddy_config",
                    status = "success",
                    "Caddy configuration deployed successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "deploy_caddy_config",
                    error = %e,
                    "Failed to deploy Caddy configuration"
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_should_create_deploy_caddy_config_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = DeployCaddyConfigStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
