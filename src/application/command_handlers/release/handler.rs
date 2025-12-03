//! Release command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::ReleaseCommandHandlerError;
use crate::domain::environment::repository::{EnvironmentRepository, RepositoryError};
use crate::domain::environment::{Configured, Released};
use crate::domain::Environment;
use crate::domain::EnvironmentName;

/// `ReleaseCommandHandler` orchestrates the software release workflow
///
/// The `ReleaseCommandHandler` orchestrates the software release workflow to
/// deploy software to a configured environment.
///
/// This command handler handles all steps required to release software:
/// 1. Load the environment from storage
/// 2. Validate the environment is in the correct state
/// 3. Execute the release steps (currently a placeholder)
/// 4. Transition environment to `Released` state
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts environment in `Configured` state
/// - Transitions to `Environment<Releasing>` at start
/// - Returns `Environment<Released>` on success
/// - Transitions to `Environment<ReleaseFailed>` on error
///
/// State is persisted after each transition using the injected repository.
pub struct ReleaseCommandHandler {
    pub(crate) repository: Arc<dyn EnvironmentRepository>,
    #[allow(dead_code)]
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
}

impl ReleaseCommandHandler {
    /// Create a new `ReleaseCommandHandler`
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn crate::shared::Clock>,
    ) -> Self {
        Self { repository, clock }
    }

    /// Execute the release workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to release to
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Released>)` on success
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment is not in `Configured` state
    /// * State persistence fails
    #[instrument(
        name = "release_command",
        skip_all,
        fields(
            command_type = "release",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Released>, ReleaseCommandHandlerError> {
        info!(
            command = "release",
            environment = %env_name,
            "Starting software release workflow"
        );

        // 1. Load the environment from storage (returns AnyEnvironmentState - type-erased)
        let any_env = self
            .repository
            .load(env_name)
            .map_err(ReleaseCommandHandlerError::StatePersistence)?;

        // 2. Check if environment exists
        let any_env = any_env.ok_or_else(|| {
            ReleaseCommandHandlerError::StatePersistence(RepositoryError::NotFound)
        })?;

        // 3. Validate environment is in Configured state and restore type safety
        let environment: Environment<Configured> = any_env.try_into_configured()?;

        info!(
            command = "release",
            environment = %env_name,
            current_state = "configured",
            target_state = "releasing",
            "Environment loaded and validated. Transitioning to Releasing state."
        );

        // 4. Transition to Releasing state
        let releasing_env = environment.start_releasing();

        // 5. Persist intermediate state
        self.repository
            .save(&releasing_env.clone().into_any())
            .map_err(ReleaseCommandHandlerError::StatePersistence)?;

        info!(
            command = "release",
            environment = %env_name,
            current_state = "releasing",
            "Releasing state persisted. Executing release steps (placeholder)."
        );

        // 6. Execute release steps (placeholder - actual implementation in Phase 6)
        // TODO: Phase 6 will add actual release steps here

        // 7. Transition to Released state
        let released_env = releasing_env.released();

        // 8. Persist final state
        self.repository
            .save(&released_env.clone().into_any())
            .map_err(ReleaseCommandHandlerError::StatePersistence)?;

        info!(
            command = "release",
            environment = %released_env.name(),
            final_state = "released",
            "Software release completed successfully"
        );

        Ok(released_env)
    }
}
