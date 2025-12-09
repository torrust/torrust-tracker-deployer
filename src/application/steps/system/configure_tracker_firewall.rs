//! Tracker firewall configuration step
//!
//! This module provides the `ConfigureTrackerFirewallStep` which handles configuration
//! of UFW firewall rules for Torrust Tracker services (UDP trackers, HTTP trackers, HTTP API).
//! This step opens the necessary ports for tracker operations while maintaining system security.
//!
//! ## Key Features
//!
//! - Opens firewall ports for configured tracker services
//! - Supports multiple UDP tracker instances
//! - Supports multiple HTTP tracker instances
//! - Opens HTTP API port for tracker management
//! - Uses centralized variables.yml for port configuration
//! - Reloads firewall rules without disrupting SSH access
//!
//! ## Port Configuration
//!
//! The step reads port numbers from the tracker configuration in variables.yml:
//! - `tracker_udp_ports`: Array of UDP tracker ports (e.g., [6868, 6969])
//! - `tracker_http_ports`: Array of HTTP tracker ports (e.g., [7070])
//! - `tracker_api_port`: HTTP API port for tracker management (e.g., 1212)
//!
//! ## Execution Order
//!
//! This step must be run **AFTER** `ConfigureFirewallStep` (which sets up SSH access).
//! It should only be executed if tracker configuration is present in the environment.
//!
//! ## Safety
//!
//! This step is designed to be safe for the following reasons:
//! 1. SSH firewall rules are already configured by ConfigureFirewallStep
//! 2. Only opens explicitly configured tracker ports
//! 3. Firewall reload preserves existing rules
//! 4. No risk of SSH lockout (SSH rules already applied)

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that configures UFW firewall rules for Tracker services
///
/// This step opens firewall ports for UDP trackers, HTTP trackers, and HTTP API.
/// Port numbers are read from the tracker configuration in variables.yml.
///
/// This step is conditional - it should only run if tracker configuration exists.
pub struct ConfigureTrackerFirewallStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureTrackerFirewallStep {
    /// Create a new tracker firewall configuration step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    ///
    /// # Note
    ///
    /// Tracker port configuration is resolved during template rendering phase
    /// and stored in variables.yml. The playbook reads these variables at runtime.
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the tracker firewall configuration
    ///
    /// This method opens firewall ports for all configured tracker services
    /// (UDP trackers, HTTP trackers, HTTP API) and reloads the firewall.
    ///
    /// # Safety
    ///
    /// This method is designed to be safe because:
    /// - SSH firewall rules are already configured by ConfigureFirewallStep
    /// - Only opens explicitly configured tracker ports
    /// - Firewall reload preserves existing SSH rules
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - UFW commands fail
    /// - Firewall reload fails
    #[instrument(
        name = "configure_tracker_firewall",
        skip_all,
        fields(step_type = "system", component = "firewall", service = "tracker", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "configure_tracker_firewall",
            action = "open_tracker_ports",
            "Configuring UFW firewall for Tracker services"
        );

        // Run Ansible playbook with variables file
        // Variables are loaded from variables.yml which contains tracker port configuration
        match self
            .ansible_client
            .run_playbook("configure-tracker-firewall", &["-e", "@variables.yml"])
        {
            Ok(_) => {
                info!(
                    step = "configure_tracker_firewall",
                    status = "success",
                    "Tracker firewall rules configured successfully"
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
    fn it_should_create_configure_tracker_firewall_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = ConfigureTrackerFirewallStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
