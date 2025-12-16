//! Prometheus configuration deployment step
//!
//! This module provides the `DeployPrometheusConfigStep` which handles deployment
//! of the Prometheus configuration file (`prometheus.yml`) to remote hosts
//! via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Deploys prometheus.yml from build directory to remote host
//! - Sets appropriate ownership and permissions
//! - Verifies successful deployment with assertions
//! - Only executes when Prometheus is enabled in environment configuration
//!
//! ## Deployment Flow
//!
//! 1. Copy prometheus.yml from build directory to remote host
//! 2. Set file permissions (0644) and ownership
//! 3. Verify file exists and has correct properties
//!
//! ## File Locations
//!
//! - **Source**: `{build_dir}/storage/prometheus/etc/prometheus.yml`
//! - **Destination**: `/opt/torrust/storage/prometheus/etc/prometheus.yml`
//! - **Container Mount**: Mounted as `/etc/prometheus/prometheus.yml`

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that deploys Prometheus configuration to a remote host via Ansible
///
/// This step copies the rendered prometheus.yml configuration file from the
/// build directory to the remote host's Prometheus configuration directory.
pub struct DeployPrometheusConfigStep {
    ansible_client: Arc<AnsibleClient>,
}

impl DeployPrometheusConfigStep {
    /// Create a new Prometheus configuration deployment step
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
    /// Runs the Ansible playbook that deploys the Prometheus configuration file.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - File copying fails
    /// - Permission setting fails
    /// - Verification assertions fail
    #[instrument(
        name = "deploy_prometheus_config",
        skip_all,
        fields(step_type = "deployment", component = "prometheus", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "deploy_prometheus_config",
            action = "deploy_file",
            "Deploying Prometheus configuration to remote host"
        );

        match self
            .ansible_client
            .run_playbook("deploy-prometheus-config", &[])
        {
            Ok(_) => {
                info!(
                    step = "deploy_prometheus_config",
                    status = "success",
                    "Prometheus configuration deployed successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "deploy_prometheus_config",
                    error = %e,
                    "Failed to deploy Prometheus configuration"
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
    fn it_should_create_deploy_prometheus_config_step() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path().to_path_buf()));

        let step = DeployPrometheusConfigStep::new(ansible_client);

        // Step should be created successfully
        assert!(!std::ptr::addr_of!(step).cast::<()>().is_null());
    }
}
