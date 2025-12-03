//! Run command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::RunCommandHandlerError;
use crate::domain::environment::repository::{EnvironmentRepository, RepositoryError};
use crate::domain::environment::Released;
use crate::domain::Environment;
use crate::domain::EnvironmentName;

/// `RunCommandHandler` orchestrates the stack execution workflow
///
/// The `RunCommandHandler` orchestrates the execution of the deployed software
/// stack on the target environment.
///
/// This command handler handles all steps required to run the stack:
/// 1. Load the environment from storage
/// 2. Validate the environment is in the correct state
/// 3. Start the services on the target instance (currently a placeholder)
/// 4. Transition environment to `Running` state
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts environment in `Released` state
/// - Transitions to `Environment<Running>` on success
/// - Transitions to `Environment<RunFailed>` on error
///
/// State is persisted after each transition using the injected repository.
pub struct RunCommandHandler {
    pub(crate) repository: Arc<dyn EnvironmentRepository>,
    #[allow(dead_code)]
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
}

impl RunCommandHandler {
    /// Create a new `RunCommandHandler`
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn crate::shared::Clock>,
    ) -> Self {
        Self { repository, clock }
    }

    /// Execute the run workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to run
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success (placeholder implementation)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment is not in `Released` state
    /// * State persistence fails
    #[instrument(
        name = "run_command",
        skip_all,
        fields(
            command_type = "run",
            environment = %env_name
        )
    )]
    pub fn execute(&self, env_name: &EnvironmentName) -> Result<(), RunCommandHandlerError> {
        info!(
            command = "run",
            environment = %env_name,
            "Starting stack execution workflow"
        );

        // 1. Load the environment from storage (returns AnyEnvironmentState - type-erased)
        let any_env = self
            .repository
            .load(env_name)
            .map_err(RunCommandHandlerError::StatePersistence)?;

        // 2. Check if environment exists
        let any_env = any_env
            .ok_or_else(|| RunCommandHandlerError::StatePersistence(RepositoryError::NotFound))?;

        // 3. Validate environment is in Released state and restore type safety
        let environment: Environment<Released> = any_env.try_into_released()?;

        info!(
            command = "run",
            environment = %env_name,
            current_state = "released",
            target_state = "running",
            "Environment loaded and validated. Would transition to Running state."
        );

        // Log intent about state transition (skeleton behavior)
        info!(
            command = "run",
            environment = %environment.name(),
            "Run command handler validated state successfully (skeleton - no actual run performed)"
        );

        Ok(())
    }
}
