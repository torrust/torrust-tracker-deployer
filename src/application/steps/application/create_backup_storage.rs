//! Backup storage creation step
//!
//! This module provides the `CreateBackupStorageStep` which handles creation
//! of backup storage directories on remote hosts via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Creates `/opt/torrust/storage/backup/etc` directory structure
//! - Sets appropriate ownership and permissions
//! - Verifies successful creation with assertions
//!
//! ## Usage Context
//!
//! This step is executed during the release workflow, before backup
//! configuration files are deployed.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `CreateBackupStorageStep` handles storage creation
//! - **Action** (Level 3): Ansible playbook execution on remote host

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that creates backup storage directories on the remote host
///
/// Creates the backup configuration directory structure required for
/// backup operations. This must be executed before deploying backup
/// configuration files.
///
/// # Directory Structure
///
/// ```text
/// /opt/torrust/storage/backup/
/// └── etc/              # Backup configuration files
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::adapters::ansible::AnsibleClient;
/// use torrust_tracker_deployer_lib::application::steps::application::CreateBackupStorageStep;
///
/// let ansible_client = Arc::new(AnsibleClient::new(std::path::PathBuf::from("/workspace")));
/// let step = CreateBackupStorageStep::new(ansible_client);
///
/// // Execute the step
/// step.execute().expect("Failed to create backup storage");
/// ```
pub struct CreateBackupStorageStep {
    ansible_client: Arc<AnsibleClient>,
}

impl CreateBackupStorageStep {
    /// Creates a new `CreateBackupStorageStep`
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - The Ansible client for executing playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Executes the step to create backup storage directories
    ///
    /// Runs the `create-backup-storage` Ansible playbook to create
    /// the backup directory structure on the remote host.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - The Ansible playbook execution fails
    /// - The remote host is unreachable
    /// - Directory creation fails due to permissions
    #[instrument(skip(self), fields(playbook = "create-backup-storage"))]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!("Creating backup storage directories on remote host");

        self.ansible_client
            .run_playbook("create-backup-storage", &[])?;

        info!("Backup storage directories created successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_should_create_create_backup_storage_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = CreateBackupStorageStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
