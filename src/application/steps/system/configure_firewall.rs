//! UFW firewall configuration step
//!
//! This module provides the `ConfigureFirewallStep` which handles configuration
//! of UFW (Uncomplicated Firewall) on remote hosts via Ansible playbooks.
//! This step ensures that the firewall is configured with restrictive default
//! policies while maintaining SSH access to prevent lockout.
//!
//! ## Key Features
//!
//! - Configures UFW with restrictive default policies (deny incoming, allow outgoing)
//! - Preserves SSH access on the configured port
//! - Uses Tera template for dynamic SSH port resolution
//! - Comprehensive SSH lockout prevention measures
//! - Verification steps to ensure firewall is active and SSH is accessible
//!
//! ## Configuration Process
//!
//! The step executes the "configure-firewall" Ansible playbook which handles:
//! - UFW installation and setup
//! - Reset UFW to clean state
//! - Set restrictive default policies
//! - Allow SSH access BEFORE enabling firewall (critical for preventing lockout)
//! - Enable UFW firewall
//! - Verify firewall status and SSH access
//!
//! ## SSH Lockout Prevention
//!
//! This is a **high-risk operation** that could result in SSH lockout if not
//! handled correctly. Safety measures include:
//!
//! 1. **Correct Sequencing**: SSH rules are added BEFORE enabling firewall
//! 2. **Dual SSH Protection**: Both port-specific and service-name rules
//! 3. **Port Configuration**: Uses actual SSH port from user configuration
//! 4. **Verification Steps**: Ansible tasks verify SSH access is preserved
//! 5. **Comprehensive Logging**: Detailed logging of each firewall step

use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::adapters::ansible::AnsibleClient;
use crate::application::traits::CommandProgressListener;
use crate::shared::command::CommandError;

/// Step that configures UFW firewall on a remote host via Ansible
///
/// This step configures a restrictive UFW firewall policy while ensuring
/// SSH access is maintained. The SSH port is resolved during template rendering
/// and embedded in the final Ansible playbook. The configuration follows the
/// principle of "allow SSH BEFORE enabling firewall" to prevent lockout.
pub struct ConfigureFirewallStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureFirewallStep {
    /// Create a new firewall configuration step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    ///
    /// # Note
    ///
    /// SSH port configuration is resolved during template rendering phase,
    /// not at step execution time. The rendered playbook contains the
    /// resolved SSH port value.
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the firewall configuration
    ///
    /// # Arguments
    ///
    /// * `listener` - Optional progress listener for reporting step-level details.
    ///   When provided, reports debug information (Ansible commands, working directory)
    ///   and detail information (firewall policies, SSH access preservation, status).
    ///
    /// # Safety
    ///
    /// This method is designed to prevent SSH lockout by:
    /// 1. Resetting UFW to clean state
    /// 2. Allowing SSH access BEFORE enabling firewall
    /// 3. Using the correct SSH port from user configuration
    ///
    /// The SSH port is resolved during template rendering and embedded in the
    /// playbook, so this method executes a playbook with pre-configured values.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - UFW commands fail
    /// - SSH rules cannot be applied
    /// - Firewall verification fails
    #[instrument(
        name = "configure_firewall",
        skip_all,
        fields(step_type = "system", component = "firewall", method = "ansible")
    )]
    pub fn execute(
        &self,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), CommandError> {
        warn!(
            step = "configure_firewall",
            action = "configure_ufw",
            "Configuring UFW firewall with variables from variables.yml"
        );

        // Report debug information about Ansible execution
        if let Some(l) = listener {
            l.on_debug(&format!(
                "Ansible working directory: {}",
                self.ansible_client.working_dir().display()
            ));
            l.on_debug("Executing playbook: ansible-playbook configure-firewall.yml -e @variables.yml -i inventory.ini");
        }

        // Run Ansible playbook with variables file
        // Note: The @ symbol in Ansible means "load variables from this file"
        // Equivalent to: ansible-playbook -e @variables.yml configure-firewall.yml
        match self
            .ansible_client
            .run_playbook("configure-firewall", &["-e", "@variables.yml"])
        {
            Ok(_) => {
                // Report configuration success with details
                if let Some(l) = listener {
                    l.on_detail("Configuring UFW with restrictive default policies");
                    l.on_detail("Allowing SSH access before enabling firewall");
                    l.on_detail("Firewall status: active");
                }

                info!(
                    step = "configure_firewall",
                    status = "success",
                    "UFW firewall configured successfully with SSH access preserved"
                );
                Ok(())
            }
            Err(e) => {
                // Propagate errors to the caller. Tests that run in container environments
                // should explicitly opt-out of running this step (for example via an
                // environment variable) instead of relying on runtime error detection.
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
    fn it_should_create_configure_firewall_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = ConfigureFirewallStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
