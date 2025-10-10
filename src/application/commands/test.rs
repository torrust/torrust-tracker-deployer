//! Infrastructure testing and validation command
//!
//! This module contains the `TestCommand` which validates deployed infrastructure
//! by running various checks to ensure services are properly installed and configured.
//!
//! ## Validation Steps
//!
//! - Cloud-init completion verification
//! - Docker installation validation
//! - Docker Compose installation validation
//!
//! The command provides comprehensive error reporting and logging to help
//! diagnose deployment issues.

use tracing::{info, instrument};

use crate::application::steps::{
    ValidateCloudInitCompletionStep, ValidateDockerComposeInstallationStep,
    ValidateDockerInstallationStep,
};
use crate::domain::environment::Environment;
use crate::infrastructure::remote_actions::RemoteActionError;
use crate::shared::command::CommandError;
use crate::shared::ssh::SshConfig;

/// Comprehensive error type for the `TestCommand`
#[derive(Debug, thiserror::Error)]
pub enum TestCommandError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Remote action failed: {0}")]
    RemoteAction(#[from] RemoteActionError),

    #[error("Environment '{environment_name}' does not have an instance IP set. The environment must be provisioned before running tests.")]
    MissingInstanceIp { environment_name: String },
}

/// `TestCommand` orchestrates the complete infrastructure testing and validation workflow
///
/// The `TestCommand` validates that an environment is properly set up with all required
/// infrastructure components.
///
/// ## Validation Steps
///
/// 1. Validate cloud-init completion
/// 2. Validate Docker installation
/// 3. Validate Docker Compose installation
///
/// ## Design Rationale
///
/// This command accepts an `Environment<S>` (any state) in its `execute` method to provide
/// flexibility for testing environments at different stages. This design:
///
/// - Aligns with `ProvisionCommand`, `ConfigureCommand` patterns (accept environment in execute)
/// - Allows testing environments regardless of compile-time state (runtime validation)
/// - Requires the environment to have an instance IP set (checked at runtime)
/// - Enables use in E2E tests where state tracking may not be enforced
pub struct TestCommand;

impl Default for TestCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCommand {
    /// Create a new `TestCommand`
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Execute the complete testing and validation workflow
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment to test (must have instance IP set)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment does not have an instance IP set
    /// * Any step in the validation workflow fails:
    ///   - Cloud-init completion validation fails
    ///   - Docker installation validation fails
    ///   - Docker Compose installation validation fails
    #[instrument(
        name = "test_command",
        skip_all,
        fields(
            command_type = "test",
            environment = %environment.name()
        )
    )]
    pub async fn execute<S>(&self, environment: &Environment<S>) -> Result<(), TestCommandError> {
        info!(
            command = "test",
            environment = %environment.name(),
            instance_ip = ?environment.instance_ip(),
            "Starting complete infrastructure testing workflow"
        );

        let instance_ip =
            environment
                .instance_ip()
                .ok_or_else(|| TestCommandError::MissingInstanceIp {
                    environment_name: environment.name().to_string(),
                })?;
        let ssh_config =
            SshConfig::with_default_port(environment.ssh_credentials().clone(), instance_ip);

        ValidateCloudInitCompletionStep::new(ssh_config.clone())
            .execute()
            .await?;

        ValidateDockerInstallationStep::new(ssh_config.clone())
            .execute()
            .await?;

        ValidateDockerComposeInstallationStep::new(ssh_config)
            .execute()
            .await?;

        info!(
            command = "test",
            environment = %environment.name(),
            instance_ip = ?environment.instance_ip(),
            "Infrastructure testing workflow completed successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_test_command() {
        let _command = TestCommand::new();

        // Verify the command was created (basic structure test)
        // TestCommand is now a zero-sized type that takes environment in execute()
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to TestCommandError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let test_error: TestCommandError = command_error.into();
        drop(test_error);

        let remote_action_error = RemoteActionError::ValidationFailed {
            action_name: "test".to_string(),
            message: "test error".to_string(),
        };
        let test_error: TestCommandError = remote_action_error.into();
        drop(test_error);
    }
}
