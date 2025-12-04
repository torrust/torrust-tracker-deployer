//! Run command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::RunCommandHandlerError;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::{Released, Running};
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
    /// Returns `Ok(Environment<Running>)` on success
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
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Running>, RunCommandHandlerError> {
        let environment = self.load_released_environment(env_name)?;

        info!(
            command = "run",
            environment = %env_name,
            current_state = "released",
            target_state = "running",
            "Environment loaded and validated. Executing run steps (placeholder)."
        );

        // TODO: Phase 6 will add actual run steps here

        let running_env = environment.start_running();

        self.repository
            .save(&running_env.clone().into_any())
            .map_err(RunCommandHandlerError::StatePersistence)?;

        info!(
            command = "run",
            environment = %running_env.name(),
            final_state = "running",
            "Stack execution completed successfully"
        );

        Ok(running_env)
    }

    /// Load environment from storage and validate it is in `Released` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Released` state
    fn load_released_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Released>, RunCommandHandlerError> {
        let any_env = self
            .repository
            .load(env_name)
            .map_err(RunCommandHandlerError::StatePersistence)?;

        let any_env = any_env.ok_or_else(|| RunCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })?;

        Ok(any_env.try_into_released()?)
    }
}
