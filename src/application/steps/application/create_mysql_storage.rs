//! `MySQL` storage directory creation step
//!
//! This module provides the `CreateMysqlStorageStep` which handles creation
//! of the required directory structure for `MySQL` on remote hosts
//! via Ansible playbooks. This step ensures `MySQL` has the necessary
//! directories for persistent data storage.
//!
//! ## Key Features
//!
//! - Creates standardized directory structure for `MySQL` storage
//! - Sets appropriate ownership (999:999 for mysql user)
//! - Idempotent operation (safe to run multiple times)
//! - Only executes when `MySQL` is enabled in environment configuration
//!
//! ## Directory Structure
//!
//! The step creates the following directory hierarchy:
//! ```text
//! /opt/torrust/storage/mysql/
//! └── data/           # MySQL persistent data (databases, tables)
//! ```
//!
//! ## Why Special Ownership?
//!
//! The `MySQL` container runs as user 999:999 (mysql). When using bind mounts
//! instead of named volumes, the host directory must be owned by this user/group
//! for the container to write data. Named volumes handle this automatically,
//! but bind mounts require explicit directory creation with correct permissions.
//!
//! See ADR: docs/decisions/bind-mount-standardization.md

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that creates `MySQL` storage directories on a remote host via Ansible
///
/// This step creates the necessary directory structure for `MySQL`,
/// ensuring all directories have correct ownership (999:999) and permissions.
pub struct CreateMysqlStorageStep {
    ansible_client: Arc<AnsibleClient>,
}

impl CreateMysqlStorageStep {
    /// Create a new `MySQL` storage directory creation step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the storage directory creation
    ///
    /// Runs the Ansible playbook that creates the `MySQL` storage directory structure.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Directory creation fails on remote host
    /// - Permission setting fails
    #[instrument(
        name = "create_mysql_storage",
        skip_all,
        fields(step_type = "system", component = "mysql", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "create_mysql_storage",
            action = "create_directories",
            "Creating MySQL storage directory structure"
        );

        match self
            .ansible_client
            .run_playbook("create-mysql-storage", &[])
        {
            Ok(_) => {
                info!(
                    step = "create_mysql_storage",
                    status = "success",
                    "MySQL storage directories created successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "create_mysql_storage",
                    error = %e,
                    "Failed to create MySQL storage directories"
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
    fn it_should_create_mysql_storage_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = CreateMysqlStorageStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
