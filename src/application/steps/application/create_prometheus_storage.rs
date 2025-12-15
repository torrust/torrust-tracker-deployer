//! Prometheus storage directory creation step
//!
//! This module provides the `CreatePrometheusStorageStep` which handles creation
//! of the required directory structure for Prometheus on remote hosts
//! via Ansible playbooks. This step ensures Prometheus has the necessary
//! directories for configuration files.
//!
//! ## Key Features
//!
//! - Creates standardized directory structure for Prometheus storage
//! - Sets appropriate ownership and permissions
//! - Idempotent operation (safe to run multiple times)
//! - Only executes when Prometheus is enabled in environment configuration
//!
//! ## Directory Structure
//!
//! The step creates the following directory hierarchy:
//! ```text
//! /opt/torrust/storage/prometheus/
//! └── etc/           # Configuration files (prometheus.yml)
//! ```

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that creates Prometheus storage directories on a remote host via Ansible
///
/// This step creates the necessary directory structure for Prometheus,
/// ensuring all directories have correct ownership and permissions.
pub struct CreatePrometheusStorageStep {
    ansible_client: Arc<AnsibleClient>,
}

impl CreatePrometheusStorageStep {
    /// Create a new Prometheus storage directory creation step
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
    /// Runs the Ansible playbook that creates the Prometheus storage directory structure.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Directory creation fails on remote host
    /// - Permission setting fails
    #[instrument(
        name = "create_prometheus_storage",
        skip_all,
        fields(step_type = "system", component = "prometheus", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "create_prometheus_storage",
            action = "create_directories",
            "Creating Prometheus storage directory structure"
        );

        match self
            .ansible_client
            .run_playbook("create-prometheus-storage", &[])
        {
            Ok(_) => {
                info!(
                    step = "create_prometheus_storage",
                    status = "success",
                    "Prometheus storage directories created successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "create_prometheus_storage",
                    error = %e,
                    "Failed to create Prometheus storage directories"
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
    fn it_should_create_prometheus_storage_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = CreatePrometheusStorageStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
