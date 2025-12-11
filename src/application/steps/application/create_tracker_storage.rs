//! Tracker storage directory creation step
//!
//! This module provides the `CreateTrackerStorageStep` which handles creation
//! of the required directory structure for the Torrust Tracker on remote hosts
//! via Ansible playbooks. This step ensures the tracker has the necessary
//! directories for configuration, data storage, and logging.
//!
//! ## Key Features
//!
//! - Creates standardized directory structure for tracker storage
//! - Sets appropriate ownership and permissions
//! - Idempotent operation (safe to run multiple times)
//!
//! ## Directory Structure
//!
//! The step creates the following directory hierarchy:
//! ```text
//! /opt/torrust/storage/tracker/
//! ├── etc/           # Configuration files (tracker.toml)
//! ├── lib/           # Application data
//! │   └── database/  # SQLite database files
//! └── log/           # Log files
//! ```

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that creates tracker storage directories on a remote host via Ansible
///
/// This step creates the necessary directory structure for the Torrust Tracker,
/// ensuring all directories have correct ownership and permissions.
pub struct CreateTrackerStorageStep {
    ansible_client: Arc<AnsibleClient>,
}

impl CreateTrackerStorageStep {
    /// Create a new tracker storage directory creation step
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
    /// Runs the Ansible playbook that creates the tracker storage directory structure.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Directory creation fails on remote host
    /// - Permission setting fails
    #[instrument(
        name = "create_tracker_storage",
        skip_all,
        fields(step_type = "system", component = "tracker", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "create_tracker_storage",
            action = "create_directories",
            "Creating tracker storage directory structure"
        );

        match self
            .ansible_client
            .run_playbook("create-tracker-storage", &[])
        {
            Ok(_) => {
                info!(
                    step = "create_tracker_storage",
                    status = "success",
                    "Tracker storage directories created successfully"
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

    #[test]
    fn test_create_tracker_storage_step_new() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("/fake/build/dir")));
        let step = CreateTrackerStorageStep::new(ansible_client);
        assert!(Arc::strong_count(&step.ansible_client) >= 1);
    }
}
