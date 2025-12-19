//! Grafana firewall configuration step
//!
//! This module provides the `ConfigureGrafanaFirewallStep` which handles configuration
//! of UFW firewall rules for Grafana UI access. This step opens port 3100 to allow
//! public access to the Grafana web interface for metrics visualization.
//!
//! ## Key Features
//!
//! - Opens firewall port 3100 for Grafana UI (container port 3000 → host port 3100)
//! - Reloads firewall rules without disrupting SSH access
//! - Conditional execution based on Grafana configuration presence
//!
//! ## Port Configuration
//!
//! The Grafana UI is exposed on a fixed port:
//! - **Host port 3100** → Container port 3000 (Grafana default)
//! - Unlike tracker ports, this is not configurable (fixed mapping)
//!
//! ## Execution Order
//!
//! This step must be run **AFTER** `ConfigureFirewallStep` (which sets up SSH access).
//! It should only be executed if Grafana configuration is present in the environment.
//!
//! ## Security Note
//!
//! This public port exposure is **temporary** until HTTPS support with reverse proxy
//! is implemented. Once a reverse proxy (like nginx) is added with HTTPS, this direct
//! port exposure will be removed, and Grafana will only be accessible through the proxy.
//!
//! ## Safety
//!
//! This step is designed to be safe for the following reasons:
//! 1. SSH firewall rules are already configured by `ConfigureFirewallStep`
//! 2. Only opens a single, fixed port (3100)
//! 3. Firewall reload preserves existing rules
//! 4. No risk of SSH lockout (SSH rules already applied)

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that configures UFW firewall rules for Grafana UI access
///
/// This step opens firewall port 3100 to allow public access to the Grafana
/// web interface. The playbook execution is unconditional - the decision to
/// execute this step is made at the command handler level based on whether
/// Grafana is configured in the environment.
pub struct ConfigureGrafanaFirewallStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureGrafanaFirewallStep {
    /// Create a new Grafana firewall configuration step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    ///
    /// # Note
    ///
    /// Unlike tracker ports which are variable, Grafana UI port is fixed at 3100.
    /// The playbook always opens this port when executed - conditional execution
    /// happens at the step level (don't run if Grafana is disabled).
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the Grafana firewall configuration
    ///
    /// This method opens firewall port 3100 for Grafana UI access and reloads
    /// the firewall. The port is fixed and not configurable.
    ///
    /// # Safety
    ///
    /// This method is designed to be safe because:
    /// - SSH firewall rules are already configured by `ConfigureFirewallStep`
    /// - Only opens a single, fixed port (3100)
    /// - Firewall reload preserves existing SSH rules
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - UFW commands fail
    /// - Firewall reload fails
    #[instrument(
        name = "configure_grafana_firewall",
        skip_all,
        fields(
            step_type = "system",
            component = "firewall",
            service = "grafana",
            method = "ansible"
        )
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "configure_grafana_firewall",
            action = "open_grafana_ui_port",
            port = 3100,
            "Configuring UFW firewall for Grafana UI"
        );

        // Run Ansible playbook
        // Unlike tracker firewall, no variables are needed (port is fixed at 3100)
        // The playbook unconditionally opens port 3100 when executed
        match self
            .ansible_client
            .run_playbook("configure-grafana-firewall", &["-e", "@variables.yml"])
        {
            Ok(_) => {
                info!(
                    step = "configure_grafana_firewall",
                    status = "success",
                    port = 3100,
                    "Grafana firewall rules configured successfully"
                );
                Ok(())
            }
            Err(e) => {
                // Propagate errors to the caller
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn it_should_create_configure_grafana_firewall_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = ConfigureGrafanaFirewallStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
