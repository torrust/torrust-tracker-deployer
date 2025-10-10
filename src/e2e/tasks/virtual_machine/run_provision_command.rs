//! Infrastructure provisioning task for E2E testing
//!
//! This module provides the E2E testing task for provisioning infrastructure using
//! `OpenTofu`. It orchestrates the complete provisioning workflow through the
//! `ProvisionCommand` and returns the IP address of the provisioned instance.
//!
//! ## Key Operations
//!
//! - Execute infrastructure provisioning via `ProvisionCommand`
//! - Initialize and apply `OpenTofu` configurations
//! - Render dynamic templates with runtime variables
//! - Retrieve instance IP addresses from `OpenTofu` outputs
//! - Prepare infrastructure for configuration phase
//!
//! ## Return Value
//!
//! Returns the IP address of the provisioned instance, which is essential
//! for subsequent E2E testing phases (configuration, validation, etc.).
//!
//! ## Integration
//!
//! This task is a critical early step in the E2E testing workflow, providing
//! the foundation infrastructure for all subsequent testing operations.

use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::application::commands::provision::ProvisionCommandError;
use crate::application::commands::ProvisionCommand;
use crate::domain::environment::state::StateTypeError;
use crate::e2e::context::TestContext;

/// Errors that can occur during the provision task
#[derive(Debug, Error)]
pub enum ProvisionTaskError {
    /// Environment is not in the correct state for provisioning
    #[error(
        "Environment must be in Created state to provision, but got: {state_type}
Tip: Ensure the environment is properly initialized before provisioning"
    )]
    InvalidState {
        state_type: String,
        #[source]
        source: StateTypeError,
    },

    /// Provisioning command execution failed
    #[error(
        "Failed to provision infrastructure: {source}
Tip: Check OpenTofu logs in the build directory for detailed error information"
    )]
    ProvisioningFailed {
        #[source]
        source: ProvisionCommandError,
    },
}

impl ProvisionTaskError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer::e2e::tasks::virtual_machine::run_provision_command::ProvisionTaskError;
    /// # use torrust_tracker_deployer::domain::environment::state::StateTypeError;
    /// let error = ProvisionTaskError::InvalidState {
    ///     state_type: "Provisioned".to_string(),
    ///     source: StateTypeError::UnexpectedState {
    ///         expected: "created",
    ///         actual: "provisioned".to_string(),
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidState { .. } => {
                "Invalid State for Provisioning - Detailed Troubleshooting:

1. Check the current environment state:
   - Examine the environment.json file in the data directory
   - Verify the environment state field shows 'Created'

2. If the environment is in a different state:
   - Destroy any existing infrastructure first
   - Re-initialize the environment from scratch

3. If the environment state is corrupted:
   - Remove the environment.json file
   - Recreate the environment using the initialization command

For more information, see the E2E testing documentation."
            }

            Self::ProvisioningFailed { .. } => {
                "Provisioning Failed - Detailed Troubleshooting:

1. Check OpenTofu logs in the build directory:
   - Review terraform.log for detailed error messages
   - Look for resource conflicts or permission errors

2. Verify infrastructure prerequisites:
   - Ensure LXD is properly installed and running
   - Check that your user has permissions to create VMs
   - Verify network connectivity to required resources

3. Check for resource conflicts:
   - Ensure no instances with the same name exist
   - Verify IP address ranges are available
   - Check for port conflicts

4. If using cloud-init:
   - Verify cloud-init configuration syntax
   - Check SSH key permissions and format

For more information, see docs/e2e-testing.md and docs/vm-providers.md."
            }
        }
    }
}

/// Provision infrastructure using `OpenTofu` and prepare for configuration
///
/// This function updates the `TestContext`'s internal environment to reflect the
/// provisioned state, ensuring consistency throughout the test lifecycle. Callers
/// can access the provisioned environment and its instance IP through the `TestContext`.
///
/// # Errors
///
/// Returns an error if:
/// - Environment is not in `Created` state
/// - `ProvisionCommand` execution fails
/// - Infrastructure provisioning fails
/// - IP address cannot be obtained from `OpenTofu` outputs
pub async fn run_provision_command(
    test_context: &mut TestContext,
) -> Result<(), ProvisionTaskError> {
    info!("Provisioning test infrastructure");

    // Create repository for this environment
    let repository = test_context.create_repository();

    // Use the new ProvisionCommand to handle all infrastructure provisioning steps
    let provision_command = ProvisionCommand::new(
        Arc::clone(&test_context.services.tofu_template_renderer),
        Arc::clone(&test_context.services.ansible_template_renderer),
        Arc::clone(&test_context.services.ansible_client),
        Arc::clone(&test_context.services.opentofu_client),
        Arc::clone(&test_context.services.clock),
        repository,
    );

    // Execute provisioning with environment in Created state
    // Extract the Created environment from AnyEnvironmentState
    let created_env = test_context
        .environment
        .clone()
        .try_into_created()
        .map_err(|source| ProvisionTaskError::InvalidState {
            state_type: test_context.environment.state_name().to_string(),
            source,
        })?;

    let provisioned_env = provision_command
        .execute(created_env)
        .await
        .map_err(|source| ProvisionTaskError::ProvisioningFailed { source })?;

    info!(
        status = "complete",
        environment = %provisioned_env.name(),
        instance_ip = ?provisioned_env.instance_ip(),
        "Instance provisioned successfully"
    );

    // Update the test context with the provisioned environment state
    test_context.update_from_provisioned(provisioned_env);

    Ok(())
}
