//! Ansible configuration task for E2E testing
//!
//! This module provides the E2E testing task for running Ansible configuration
//! on target instances. It executes Ansible playbooks to configure services
//! and applications on the deployed infrastructure.
//!
//! ## Key Operations
//!
//! - Executes Ansible playbooks using the `ConfigureCommandHandler`
//! - Handles configuration workflow for both containers and VMs
//! - Provides structured error handling and reporting
//!
//! ## Integration
//!
//! This is a generic task that works with infrastructure-agnostic configuration:
//! - Uses rendered Ansible inventories from provision simulation
//! - Works with both container and VM-based infrastructure
//! - Integrates with the existing `ConfigureCommandHandler` workflow
//!
//! ## E2E Config Tests Integration
//!
//! In E2E config tests, this module works seamlessly with provision simulation.
//! The provision simulation ensures that Ansible config files are generated with
//! the correct configuration even without executing the actual provision phase,
//! allowing the configuration command to run successfully on simulated infrastructure.

use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::application::command_handlers::configure::ConfigureCommandHandlerError;
use crate::application::command_handlers::ConfigureCommandHandler;
use crate::domain::environment::state::StateTypeError;
use crate::testing::e2e::context::TestContext;

/// Configure infrastructure using Ansible playbooks
///
/// This function executes Ansible configuration using the `ConfigureCommandHandler` for E2E tests.
/// It extracts the provisioned environment from the `TestContext` and applies configuration,
/// ensuring type-safe state transitions.
///
/// This function updates the `TestContext`'s internal environment to reflect the
/// configured state, ensuring consistency throughout the test lifecycle. Callers
/// can access the configured environment through the `TestContext`.
///
/// # Errors
///
/// Returns an error if:
/// - Environment is not in `Provisioned` state
/// - `ConfigureCommandHandler` execution fails
/// - Infrastructure configuration fails
pub fn run_configure_command(test_context: &mut TestContext) -> Result<(), ConfigureTaskError> {
    info!("Configuring test infrastructure");

    // Extract provisioned environment from TestContext for configuration
    let provisioned_env = test_context
        .environment
        .clone()
        .try_into_provisioned()
        .map_err(|source| ConfigureTaskError::InvalidState {
            state_type: test_context.environment.state_name().to_string(),
            source,
        })?;

    // Create repository for this environment
    let repository = test_context.create_repository();

    // Use the new ConfigureCommandHandler to handle all infrastructure configuration steps
    let configure_command_handler = ConfigureCommandHandler::new(
        Arc::clone(&test_context.services.ansible_client),
        Arc::clone(&test_context.services.clock),
        repository,
    );

    let configured_env = configure_command_handler
        .execute(provisioned_env)
        .map_err(|source| ConfigureTaskError::ConfigurationFailed { source })?;

    info!(
        status = "complete",
        environment = %configured_env.name(),
        "Infrastructure configuration completed successfully"
    );

    // Update the test context with the configured environment state
    test_context.update_from_configured(configured_env);

    Ok(())
}

/// Errors that can occur during the configure task
#[derive(Debug, Error)]
pub enum ConfigureTaskError {
    /// Environment is not in the correct state for configuration
    #[error(
        "Environment must be in Provisioned state to configure, but got: {state_type}
Tip: Run the provision command successfully before attempting configuration"
    )]
    InvalidState {
        state_type: String,
        #[source]
        source: StateTypeError,
    },

    /// Configuration command execution failed
    #[error(
        "Failed to configure infrastructure: {source}
Tip: Check Ansible logs for detailed playbook execution errors"
    )]
    ConfigurationFailed {
        #[source]
        source: ConfigureCommandHandlerError,
    },
}

impl ConfigureTaskError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_configure_command::ConfigureTaskError;
    /// # use torrust_tracker_deployer_lib::domain::environment::state::StateTypeError;
    /// let error = ConfigureTaskError::InvalidState {
    ///     state_type: "Created".to_string(),
    ///     source: StateTypeError::UnexpectedState {
    ///         expected: "provisioned",
    ///         actual: "created".to_string(),
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidState { .. } => {
                "Invalid State for Configuration - Detailed Troubleshooting:

1. Verify provisioning completed successfully:
   - Check that the provision command finished without errors
   - Ensure the environment.json shows 'Provisioned' state

2. Check instance connectivity:
   - Verify the instance IP address is accessible
   - Test SSH connectivity to the instance
   - Ensure cloud-init has completed

3. If provisioning failed previously:
   - Re-run the provision command to complete provisioning
   - Or destroy and recreate the environment from scratch

For more information, see the E2E testing documentation."
            }

            Self::ConfigurationFailed { .. } => {
                "Configuration Failed - Detailed Troubleshooting:

1. Check Ansible execution logs:
   - Review Ansible output for specific task failures
   - Look for permission or connectivity errors

2. Verify instance prerequisites:
   - Ensure SSH connectivity is working
   - Check that the instance OS is supported
   - Verify Python is installed (required by Ansible)

3. Check Ansible inventory and playbooks:
   - Verify inventory file has correct instance IP
   - Ensure playbook files exist in the build directory
   - Check variable substitutions in rendered templates

4. Common issues:
   - SSH key permissions (should be 600)
   - Firewall blocking Ansible connections
   - Instance not fully initialized (cloud-init still running)
   - Package repository connectivity issues

For more information, see docs/e2e-testing.md."
            }
        }
    }
}
