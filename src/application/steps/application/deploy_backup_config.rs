//! Backup configuration deployment step
//!
//! This module provides the `DeployBackupConfigStep` which handles deployment
//! of backup configuration files to remote hosts via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Deploys backup.conf and backup-paths.txt from build directory to remote host
//! - Sets appropriate ownership and permissions
//! - Verifies successful deployment with assertions
//! - Only executes when backup is enabled in environment configuration
//!
//! ## Deployment Flow
//!
//! 1. Copy backup.conf and backup-paths.txt from build directory to remote host
//! 2. Set file permissions (0644) and ownership
//! 3. Verify files exist and have correct properties
//!
//! ## File Locations
//!
//! - **Source**: `{build_dir}/backup/backup.conf` and `backup-paths.txt`
//! - **Destination**: `/opt/torrust/storage/backup/etc/backup.conf` and `backup-paths.txt`
//! - **Container Mount**: Mounted as `/backup/etc/backup.conf` and `/backup/etc/backup-paths.txt`

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that deploys backup configuration to a remote host via Ansible
///
/// This step copies the rendered backup configuration files from the
/// build directory to the remote host's backup configuration directory.
pub struct DeployBackupConfigStep {
    ansible_client: Arc<AnsibleClient>,
}

impl DeployBackupConfigStep {
    /// Create a new backup configuration deployment step
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
    /// Runs the Ansible playbook that deploys the backup configuration files.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - File copying fails
    /// - Permission setting fails
    /// - Verification assertions fail
    #[instrument(
        name = "deploy_backup_config",
        skip_all,
        fields(step_type = "deployment", component = "backup", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "deploy_backup_config",
            action = "deploy_files",
            "Deploying backup configuration to remote host"
        );

        match self
            .ansible_client
            .run_playbook("deploy-backup-config", &[])
        {
            Ok(_) => {
                info!(
                    step = "deploy_backup_config",
                    status = "success",
                    "Backup configuration deployed successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "deploy_backup_config",
                    error = %e,
                    "Failed to deploy backup configuration"
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
    fn it_should_create_deploy_backup_config_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = DeployBackupConfigStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
