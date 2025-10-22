//! Infrastructure destruction task for E2E testing
//!
//! This module provides the E2E testing task for destroying infrastructure using
//! the `DestroyCommand`. It orchestrates the complete infrastructure teardown workflow
//! through the application layer command.
//!
//! ## Key Operations
//!
//! - Execute infrastructure destruction via `DestroyCommand`
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

use crate::application::commands::destroy::DestroyCommandError;
use crate::application::commands::DestroyCommand;
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
        source: DestroyCommandError,
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
    /// # use torrust_tracker_deployer_lib::application::commands::destroy::DestroyCommandError;
    /// # use torrust_tracker_deployer_lib::shared::command::CommandError;
    /// let error = DestroyTaskError::DestructionFailed {
    ///     source: DestroyCommandError::Command(CommandError::StartupFailed {
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

/// Destroy infrastructure using `DestroyCommand`
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
/// - `DestroyCommand` execution fails
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

    // Use the new DestroyCommand to handle all infrastructure destruction steps
    let destroy_command = DestroyCommand::new(
        Arc::clone(&test_context.services.opentofu_client),
        repository,
    );

    // Execute destruction with environment (can be in any state)
    // The DestroyCommand accepts Environment<S> generically, so we need to extract
    // the environment from AnyEnvironmentState. Since destroy works on any state,
    // we handle the special case of already-destroyed environments.

    let destroyed_env = match test_context.environment.clone() {
        AnyEnvironmentState::Destroyed(env) => {
            // Already destroyed, just return it
            info!("Environment is already in Destroyed state");
            Ok(env)
        }
        AnyEnvironmentState::Created(env) => destroy_command.execute(env),
        AnyEnvironmentState::Provisioning(env) => destroy_command.execute(env),
        AnyEnvironmentState::Provisioned(env) => destroy_command.execute(env),
        AnyEnvironmentState::Configuring(env) => destroy_command.execute(env),
        AnyEnvironmentState::Configured(env) => destroy_command.execute(env),
        AnyEnvironmentState::Releasing(env) => destroy_command.execute(env),
        AnyEnvironmentState::Released(env) => destroy_command.execute(env),
        AnyEnvironmentState::Running(env) => destroy_command.execute(env),
        AnyEnvironmentState::ProvisionFailed(env) => destroy_command.execute(env),
        AnyEnvironmentState::ConfigureFailed(env) => destroy_command.execute(env),
        AnyEnvironmentState::ReleaseFailed(env) => destroy_command.execute(env),
        AnyEnvironmentState::RunFailed(env) => destroy_command.execute(env),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ssh::{SshCredentials, DEFAULT_SSH_PORT};
    use crate::domain::{Environment, EnvironmentName};
    use crate::shared::Username;
    use crate::testing::e2e::context::{TestContext, TestContextType};
    use tempfile::TempDir;

    /// Helper function to create a test context for testing
    fn create_test_context_with_keep_flag(keep_env: bool) -> TestContext {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create SSH key files
        let ssh_private_key = temp_path.join("test_key");
        let ssh_public_key = temp_path.join("test_key.pub");
        std::fs::write(&ssh_private_key, "test private key").expect("Failed to write private key");
        std::fs::write(&ssh_public_key, "test public key").expect("Failed to write public key");

        let env_name = EnvironmentName::new("test-env".to_string()).expect("Valid name");
        let ssh_user = Username::new("testuser".to_string()).expect("Valid username");
        let ssh_credentials = SshCredentials::new(ssh_private_key, ssh_public_key, ssh_user);
        let environment = Environment::new(env_name, ssh_credentials, DEFAULT_SSH_PORT);

        // Create test context with the keep_env flag
        let test_context = TestContext::from_environment(keep_env, environment, TestContextType::VirtualMachine)
            .expect("Failed to create test context")
            .init()
            .expect("Failed to initialize test context");

        // Keep temp_dir alive by leaking it (acceptable in tests)
        std::mem::forget(temp_dir);

        test_context
    }

    #[test]
    fn test_run_destroy_command_respects_keep_env_true() {
        // Create a test context with keep_env = true
        let mut test_context = create_test_context_with_keep_flag(true);

        // Call run_destroy_command
        let result = run_destroy_command(&mut test_context);

        // Should succeed without attempting actual destruction
        assert!(result.is_ok(), "run_destroy_command should succeed when keep_env is true");

        // Verify the environment state remains unchanged (not actually destroyed)
        // Since keep_env is true, the function should return early without modifying state
        assert_eq!(
            test_context.environment.state_name(),
            "created",
            "Environment state should remain created when keep_env is true"
        );
    }

    #[test]
    fn test_run_destroy_command_attempts_destruction_when_keep_env_false() {
        // Create a test context with keep_env = false
        let mut test_context = create_test_context_with_keep_flag(false);

        // Call run_destroy_command
        let result = run_destroy_command(&mut test_context);

        // This will fail because there's no actual infrastructure to destroy in the test
        // But we can verify it attempted to destroy (didn't return early)
        // The error confirms the destruction was attempted
        assert!(
            result.is_err(),
            "run_destroy_command should attempt destruction when keep_env is false"
        );

        // Verify the error is a DestructionFailed error (not an early return)
        match result {
            Err(DestroyTaskError::DestructionFailed { .. }) => {
                // Expected - destruction was attempted but failed due to missing infrastructure
            }
            _ => panic!("Expected DestructionFailed error when keep_env is false"),
        }
    }

    #[test]
    fn test_keep_env_flag_is_preserved_in_test_context() {
        // Test with keep_env = true
        let test_context_keep = create_test_context_with_keep_flag(true);
        assert!(
            test_context_keep.keep_env,
            "keep_env should be true when set to true"
        );

        // Test with keep_env = false
        let test_context_destroy = create_test_context_with_keep_flag(false);
        assert!(
            !test_context_destroy.keep_env,
            "keep_env should be false when set to false"
        );
    }
}
