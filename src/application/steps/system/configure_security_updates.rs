//! Automatic security updates configuration step
//!
//! This module provides the `ConfigureSecurityUpdatesStep` which handles
//! configuration of automatic security updates on remote hosts via Ansible playbooks.
//! This step ensures that the system automatically receives and installs security
//! patches with scheduled reboots.
//!
//! ## Key Features
//!
//! - Installs and configures unattended-upgrades package
//! - Enables automatic security update installation
//! - Configures automatic reboots at 2:00 AM when updates require restart
//! - Verifies configuration is valid and service is running
//! - Integration with the step-based deployment architecture
//!
//! ## Configuration Process
//!
//! The step executes the "configure-security-updates" Ansible playbook which handles:
//! - Package installation (unattended-upgrades)
//! - Automatic update configuration
//! - Reboot scheduling for security updates
//! - Service enablement and startup
//! - Configuration verification

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that configures automatic security updates on a remote host via Ansible
pub struct ConfigureSecurityUpdatesStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureSecurityUpdatesStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the security updates configuration step
    ///
    /// This will run the "configure-security-updates" Ansible playbook to configure
    /// unattended-upgrades on the remote host. The playbook handles package installation,
    /// automatic update configuration, and scheduled reboot setup.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The Ansible client fails to execute the playbook
    /// * Package installation fails
    /// * Configuration file modification fails
    /// * Service startup fails
    /// * Configuration verification fails
    /// * The playbook execution fails for any other reason
    #[instrument(
        name = "configure_security_updates",
        skip_all,
        fields(
            step_type = "system",
            component = "security_updates",
            method = "ansible"
        )
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "configure_security_updates",
            action = "configure_automatic_updates",
            "Configuring automatic security updates via Ansible"
        );

        self.ansible_client
            .run_playbook("configure-security-updates")?;

        info!(
            step = "configure_security_updates",
            status = "success",
            "Automatic security updates configuration completed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn it_should_create_configure_security_updates_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = ConfigureSecurityUpdatesStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
