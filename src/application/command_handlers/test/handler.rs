//! Test command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::TestCommandHandlerError;
use crate::adapters::ssh::SshConfig;
use crate::application::steps::{
    ValidateCloudInitCompletionStep, ValidateDockerComposeInstallationStep,
    ValidateDockerInstallationStep,
};
use crate::domain::environment::repository::{
    EnvironmentRepository, RepositoryError, TypedEnvironmentRepository,
};
use crate::domain::EnvironmentName;

/// `TestCommandHandler` orchestrates the complete infrastructure testing and validation workflow
///
/// The `TestCommandHandler` validates that an environment is properly set up with all required
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
/// This command accepts an `EnvironmentName` in its `execute` method to align with other
/// command handlers (`ProvisionCommandHandler`, `ConfigureCommandHandler`). This design:
///
/// - Loads environment from repository (consistent pattern across all handlers)
/// - Allows testing environments regardless of compile-time state (runtime validation)
/// - Requires the environment to have an instance IP set (checked at runtime)
/// - Enables repository integration for future enhancements (e.g., tracking test history)
pub struct TestCommandHandler {
    repository: TypedEnvironmentRepository,
}

impl TestCommandHandler {
    /// Create a new `TestCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self {
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the complete testing and validation workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to test
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment does not have an instance IP set
    /// * Any validation step fails:
    ///   - Cloud-init completion validation fails
    ///   - Docker installation validation fails
    ///   - Docker Compose installation validation fails
    #[instrument(
        name = "test_command",
        skip_all,
        fields(
            command_type = "test",
            environment = %env_name
        )
    )]
    pub async fn execute(&self, env_name: &EnvironmentName) -> Result<(), TestCommandHandlerError> {
        info!(
            command = "test",
            environment = %env_name,
            "Starting complete infrastructure testing workflow"
        );

        // 1. Load the environment from storage (returns AnyEnvironmentState - type-erased)
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(TestCommandHandlerError::StatePersistence)?;

        // 2. Check if environment exists
        let any_env = any_env.ok_or(TestCommandHandlerError::StatePersistence(
            RepositoryError::NotFound,
        ))?;

        // 3. Extract instance IP (runtime check - works with any state)
        let instance_ip =
            any_env
                .instance_ip()
                .ok_or_else(|| TestCommandHandlerError::MissingInstanceIp {
                    environment_name: env_name.to_string(),
                })?;

        let ssh_config =
            SshConfig::with_default_port(any_env.ssh_credentials().clone(), instance_ip);

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
            environment = %env_name,
            instance_ip = ?instance_ip,
            "Infrastructure testing workflow completed successfully"
        );

        Ok(())
    }
}
