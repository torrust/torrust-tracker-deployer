//! Infrastructure destruction task for E2E testing
//!
//! This module provides the E2E testing task for destroying infrastructure using
//! the `DestroyCommandHandler`. It orchestrates the complete infrastructure teardown workflow
//! through the application layer command.
//!
//! ## Key Operations
//!
//! - Execute infrastructure destruction via `DestroyCommandHandler`
//! - Destroy infrastructure using `OpenTofu` operations
//! - Transition environment to `Destroyed` state
//! - Update test context with final state
//!
//! ## Integration
//!
//! This task is typically the final step in E2E testing workflows, cleaning up
//! all provisioned infrastructure after tests complete.

use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
use crate::application::command_handlers::DestroyCommandHandler;
use crate::testing::e2e::context::TestContext;

/// Errors that can occur during the destroy task
#[derive(Debug, Error)]
pub enum DestroyTaskError {
    /// Destruction command execution failed
    #[error(
        "Failed to destroy infrastructure: {source}
Tip: Check OpenTofu logs in the build directory for detailed error information"
    )]
    DestructionFailed {
        #[source]
        source: DestroyCommandHandlerError,
    },
}

impl DestroyTaskError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::virtual_machine::run_destroy_command::DestroyTaskError;
    /// # use torrust_tracker_deployer_lib::application::command_handlers::destroy::DestroyCommandHandlerError;
    /// # use torrust_tracker_deployer_lib::shared::command::CommandError;
    /// let error = DestroyTaskError::DestructionFailed {
    ///     source: DestroyCommandHandlerError::Command(CommandError::StartupFailed {
    ///         command: "tofu".to_string(),
    ///         source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    ///     }),
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DestructionFailed { .. } => {
                "Destruction Failed - Detailed Troubleshooting:

1. Check OpenTofu logs in the build directory:
   - Review terraform.log for detailed error messages
   - Look for resource deletion conflicts or permission errors

2. Verify infrastructure state:
   - Ensure infrastructure resources still exist
   - Check that OpenTofu state files are intact
   - Verify network connectivity to infrastructure providers

3. Check for resource locks:
   - Ensure no other processes are accessing the resources
   - Verify that no manual holds exist on resources
   - Check for dependency issues preventing deletion

4. Manual cleanup may be required if destroy fails:
   - Review OpenTofu state to identify remaining resources
   - Use provider-specific tools (e.g., lxc commands) for manual cleanup
   - Remove state files after manual cleanup is complete

For more information, see docs/e2e-testing.md and docs/vm-providers.md."
            }
        }
    }
}

/// Destroy infrastructure using `DestroyCommandHandler`
///
/// This function updates the `TestContext`'s internal environment to reflect the
/// destroyed state, ensuring consistency throughout the test lifecycle. Callers
/// can access the destroyed environment through the `TestContext`.
///
/// If the `keep_env` flag is set in the test context, this function will skip
/// destruction and preserve the environment for debugging purposes.
///
/// # Errors
///
/// Returns an error if:
/// - `DestroyCommandHandler` execution fails
/// - Infrastructure destruction fails
/// - `OpenTofu` destroy operations fail
pub fn run_destroy_command(test_context: &mut TestContext) -> Result<(), DestroyTaskError> {
    use crate::domain::environment::state::AnyEnvironmentState;

    // If keep_env is set, skip destruction and preserve the environment
    if test_context.keep_env {
        let instance_name = &test_context.environment.instance_name();
        info!(
            operation = "destroy",
            action = "keep_environment",
            instance = %instance_name,
            connect_command = format!("lxc exec {} -- /bin/bash", instance_name),
            "Keeping test environment as requested (destruction skipped)"
        );
        return Ok(());
    }

    info!("Destroying test infrastructure");

    // Create repository for this environment
    let repository = test_context.create_repository();

    // Use the new DestroyCommandHandler to handle all infrastructure destruction steps
    let destroy_command_handler = DestroyCommandHandler::new(
        Arc::clone(&test_context.services.opentofu_client),
        repository,
    );

    // Execute destruction with environment (can be in any state)
    // The DestroyCommandHandler accepts Environment<S> generically, so we need to extract
    // the environment from AnyEnvironmentState. Since destroy works on any state,
    // we handle the special case of already-destroyed environments.

    let destroyed_env = match test_context.environment.clone() {
        AnyEnvironmentState::Destroyed(env) => {
            // Already destroyed, just return it
            info!("Environment is already in Destroyed state");
            Ok(env)
        }
        AnyEnvironmentState::Created(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Provisioning(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Provisioned(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Configuring(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Configured(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Releasing(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Released(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::Running(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::ProvisionFailed(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::ConfigureFailed(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::ReleaseFailed(env) => destroy_command_handler.execute(env),
        AnyEnvironmentState::RunFailed(env) => destroy_command_handler.execute(env),
    }
    .map_err(|source| DestroyTaskError::DestructionFailed { source })?;

    info!(
        status = "complete",
        environment = %destroyed_env.name(),
        "Infrastructure destroyed successfully"
    );

    // Update the test context with the destroyed environment state
    test_context.update_from_destroyed(destroyed_env);

    Ok(())
}
