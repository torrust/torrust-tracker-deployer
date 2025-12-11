//! Tracker database initialization step
//!
//! This module provides the `InitTrackerDatabaseStep` which handles creation
//! of the `SQLite` database file for the Torrust Tracker on remote hosts
//! via Ansible playbooks. This step ensures the tracker has an empty database
//! file ready for schema initialization and data storage.
//!
//! ## Key Features
//!
//! - Creates empty `SQLite` database file
//! - Sets appropriate ownership and permissions
//! - Idempotent operation (safe to run multiple times)
//! - Verifies database file creation
//!
//! ## Database Location
//!
//! The step creates:
//! ```text
//! /opt/torrust/storage/tracker/lib/database/tracker.db
//! ```
//!
//! ## Prerequisites
//!
//! - Tracker storage directories must exist (created by `CreateTrackerStorageStep`)
//! - The ansible user must have write access to the database directory

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that initializes the tracker database on a remote host via Ansible
///
/// This step creates an empty `SQLite` database file for the Torrust Tracker,
/// ensuring it has correct ownership and permissions.
pub struct InitTrackerDatabaseStep {
    ansible_client: Arc<AnsibleClient>,
}

impl InitTrackerDatabaseStep {
    /// Create a new tracker database initialization step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the database initialization
    ///
    /// Runs the Ansible playbook that creates the empty `SQLite` database file.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Database file creation fails on remote host
    /// - Permission setting fails
    /// - File verification fails
    #[instrument(
        name = "init_tracker_database",
        skip_all,
        fields(step_type = "application", component = "tracker", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "init_tracker_database",
            action = "create_database_file",
            "Initializing tracker SQLite database"
        );

        match self
            .ansible_client
            .run_playbook("init-tracker-database", &[])
        {
            Ok(_) => {
                info!(
                    step = "init_tracker_database",
                    status = "success",
                    "Tracker database initialized successfully"
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
    fn test_init_tracker_database_step_new() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("/fake/build/dir")));
        let step = InitTrackerDatabaseStep::new(ansible_client);
        assert!(Arc::strong_count(&step.ansible_client) >= 1);
    }
}
