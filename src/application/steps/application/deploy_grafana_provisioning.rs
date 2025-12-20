//! Grafana provisioning deployment step
//!
//! This module provides the `DeployGrafanaProvisioningStep` which handles deployment
//! of Grafana provisioning configuration files (datasources and dashboards) to remote hosts
//! via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Deploys Grafana datasource configuration (prometheus.yml)
//! - Deploys Grafana dashboard provider configuration
//! - Deploys dashboard JSON files
//! - Sets appropriate ownership and permissions
//! - Only executes when Grafana is enabled in environment configuration
//!
//! ## Deployment Flow
//!
//! 1. Create provisioning directory structure on remote host
//! 2. Copy all provisioning files from build directory to remote host
//! 3. Set file permissions (0644) and directory permissions (0755)
//!
//! ## File Locations
//!
//! - **Source**: `{build_dir}/grafana/provisioning/**/*`
//! - **Destination**: `/opt/torrust/storage/grafana/provisioning/**/*`
//! - **Container Mount**: Mounted as `/etc/grafana/provisioning/` (read-only)

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that deploys Grafana provisioning configuration to a remote host via Ansible
///
/// This step copies all rendered Grafana provisioning files (datasources, dashboards,
/// dashboard JSONs) from the build directory to the remote host's Grafana provisioning
/// directory.
pub struct DeployGrafanaProvisioningStep {
    ansible_client: Arc<AnsibleClient>,
}

impl DeployGrafanaProvisioningStep {
    /// Create a new Grafana provisioning deployment step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the provisioning deployment
    ///
    /// Runs the Ansible playbook that deploys Grafana provisioning files.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Directory creation fails
    /// - File copying fails
    /// - Permission setting fails
    #[instrument(
        name = "deploy_grafana_provisioning",
        skip_all,
        fields(step_type = "deployment", component = "grafana", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "deploy_grafana_provisioning",
            action = "deploy_files",
            "Deploying Grafana provisioning configuration to remote host"
        );

        match self
            .ansible_client
            .run_playbook("deploy-grafana-provisioning", &[])
        {
            Ok(_) => {
                info!(
                    step = "deploy_grafana_provisioning",
                    status = "success",
                    "Grafana provisioning configuration deployed successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    step = "deploy_grafana_provisioning",
                    error = %e,
                    "Failed to deploy Grafana provisioning configuration"
                );
                Err(e)
            }
        }
    }
}
