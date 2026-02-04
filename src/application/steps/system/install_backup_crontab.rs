//! Backup crontab installation step
//!
//! This module provides the `InstallBackupCrontabStep` which handles installation
//! of the backup crontab entry and maintenance script on remote hosts via Ansible playbooks.
//! This step ensures that scheduled backups are configured to run automatically.
//!
//! ## Key Features
//!
//! - Copies maintenance-backup.sh to /usr/local/bin/ with executable permissions
//! - Installs crontab entry to /etc/cron.d/tracker-backup
//! - Creates backup log file with proper permissions
//! - Verifies all files are properly installed
//!
//! ## Configuration Process
//!
//! The step executes the "install-backup-crontab" Ansible playbook which handles:
//! - Copying the maintenance script to /usr/local/bin/
//! - Installing the crontab entry to /etc/cron.d/
//! - Creating the backup log file
//! - Verifying all files exist and have correct permissions

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that installs backup crontab and maintenance script via Ansible
///
/// This step installs the backup crontab entry and the maintenance script
/// that will orchestrate scheduled backups. The crontab entry runs on the
/// configured schedule to stop the tracker, perform backup, and restart.
pub struct InstallBackupCrontabStep {
    ansible_client: Arc<AnsibleClient>,
}

impl InstallBackupCrontabStep {
    /// Create a new backup crontab installation step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the backup crontab installation
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Files cannot be copied to remote host
    /// - Permissions cannot be set correctly
    /// - Verification checks fail
    #[instrument(
        name = "install_backup_crontab",
        skip_all,
        fields(step_type = "system", component = "backup", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "install_backup_crontab",
            action = "install_crontab",
            "Installing backup crontab and maintenance script"
        );

        match self
            .ansible_client
            .run_playbook("install-backup-crontab", &[])
        {
            Ok(_) => {
                info!(
                    step = "install_backup_crontab",
                    action = "install_crontab",
                    status = "completed",
                    "Backup crontab and script installed successfully"
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ansible::AnsibleClient;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn it_should_create_step_with_ansible_client() {
        let build_dir = PathBuf::from("/tmp/test-build");
        let ansible_client = Arc::new(AnsibleClient::new(build_dir));
        let step = InstallBackupCrontabStep::new(ansible_client);
        assert!(Arc::strong_count(&step.ansible_client) >= 1);
    }
}
