//! Test command handler implementation
//!
//! **Purpose**: Smoke test for running Torrust Tracker services
//!
//! This handler validates that a deployed Tracker application is running and accessible.
//! The command is designed for post-deployment verification - checking that services
//! respond correctly to requests, not validating infrastructure components.
//!
//! **Current Implementation Status**: Work in Progress / Temporary Scaffolding
//!
//! The current validation steps (cloud-init, Docker, Docker Compose) are **temporary
//! scaffolding** that exist only because the complete deployment workflow is not yet
//! implemented. These steps will be **removed** when the full deployment is implemented
//! and replaced with actual smoke tests.
//!
//! **Target Implementation** (when `Running` state is implemented):
//!
//! - Make HTTP requests to publicly exposed Tracker services
//! - Verify services respond correctly (health checks, basic API calls)
//! - Confirm deployment is production-ready from end-user perspective
//!
//! For rationale and alternatives, see:
//! - `docs/decisions/test-command-as-smoke-test.md` - Architectural decision record

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

/// `TestCommandHandler` orchestrates smoke testing for running Torrust Tracker services
///
/// **Purpose**: Post-deployment smoke test to verify the application is running and accessible
///
/// **Current Status**: Work in Progress - Current implementation is temporary scaffolding
///
/// The current validation steps are **placeholders** until the complete deployment workflow
/// is implemented with the `Running` state. See module documentation for details.
///
/// ## Current Validation Steps (Temporary)
///
/// 1. Validate cloud-init completion
/// 2. Validate Docker installation
/// 3. Validate Docker Compose installation
///
/// ## Target Validation Steps (Future)
///
/// 1. HTTP health check to Tracker service
/// 2. Basic API request verification
/// 3. Metrics endpoint validation
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
