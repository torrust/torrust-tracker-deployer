//! Destroy command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::DestroyCommandHandlerError;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::DestroyInfrastructureStep;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::{Destroyed, Environment};
use crate::shared::error::Traceable;

/// `DestroyCommandHandler` orchestrates the complete infrastructure destruction workflow
///
/// The `DestroyCommandHandler` orchestrates the complete infrastructure teardown workflow.
///
/// This command handler handles all steps required to destroy infrastructure:
/// 1. Destroy infrastructure via `OpenTofu`
/// 2. Clean up state files
/// 3. Transition environment to `Destroyed` state
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<S>` (any state) as input via environment name lookup
/// - Transitions to `Environment<Destroying>` at start
/// - Returns `Environment<Destroyed>` on success
/// - Transitions to `Environment<DestroyFailed>` on error
///
/// State is persisted after each transition using the injected repository.
///
/// # Idempotency
///
/// The destroy operation is idempotent. Running it multiple times on the same
/// environment will:
/// - Succeed if the infrastructure is already destroyed
/// - Report appropriate status to the user
/// - Not fail due to missing resources
pub struct DestroyCommandHandler {
    pub(crate) repository: TypedEnvironmentRepository,
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
}

impl DestroyCommandHandler {
    /// Create a new `DestroyCommandHandler`
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn crate::shared::Clock>,
    ) -> Self {
        Self {
            repository: TypedEnvironmentRepository::new(repository),
            clock,
        }
    }

    /// Execute the complete destruction workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to destroy
    ///
    /// # Returns
    ///
    /// Returns the destroyed environment
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the destruction workflow fails:
    /// * Environment not found or cannot be loaded
    /// * Environment is in an invalid state for destruction  
    /// * `OpenTofu` destroy fails
    /// * Unable to persist the destroyed state
    ///
    /// On error, the environment transitions to `DestroyFailed` state and is persisted.
    #[instrument(
        name = "destroy_command",
        skip_all,
        fields(
            command_type = "destroy",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &crate::domain::environment::name::EnvironmentName,
    ) -> Result<crate::domain::environment::Environment<Destroyed>, DestroyCommandHandlerError>
    {
        use crate::domain::environment::state::AnyEnvironmentState;

        info!(
            command = "destroy",
            environment = %env_name,
            "Starting complete infrastructure destruction workflow"
        );

        // 1. Load the environment from storage
        let environment = self
            .repository
            .inner()
            .load(env_name)
            .map_err(DestroyCommandHandlerError::StatePersistence)?;

        // 2. Check if environment exists
        let environment = environment.ok_or_else(|| {
            DestroyCommandHandlerError::StatePersistence(
                crate::domain::environment::repository::RepositoryError::NotFound,
            )
        })?;

        // 3. Check if environment is already destroyed
        if let AnyEnvironmentState::Destroyed(env) = environment {
            info!(
                command = "destroy",
                environment = %env_name,
                "Environment is already destroyed"
            );
            return Ok(env);
        }

        // 4. Capture start time before transitioning to Destroying state
        let started_at = self.clock.now();

        // 5. Get the build directory from the environment context (before consuming environment)
        let opentofu_build_dir = environment.tofu_build_dir();

        // 6. Transition to Destroying state
        // Since we have AnyEnvironmentState, we need to match on it and call start_destroying on the typed environment
        let destroying_env = match environment {
            AnyEnvironmentState::Created(env) => env.start_destroying(),
            AnyEnvironmentState::Provisioning(env) => env.start_destroying(),
            AnyEnvironmentState::Provisioned(env) => env.start_destroying(),
            AnyEnvironmentState::Configuring(env) => env.start_destroying(),
            AnyEnvironmentState::Configured(env) => env.start_destroying(),
            AnyEnvironmentState::Releasing(env) => env.start_destroying(),
            AnyEnvironmentState::Released(env) => env.start_destroying(),
            AnyEnvironmentState::Running(env) => env.start_destroying(),
            AnyEnvironmentState::Destroying(env) => env, // Already destroying
            AnyEnvironmentState::ProvisionFailed(env) => env.start_destroying(),
            AnyEnvironmentState::ConfigureFailed(env) => env.start_destroying(),
            AnyEnvironmentState::ReleaseFailed(env) => env.start_destroying(),
            AnyEnvironmentState::RunFailed(env) => env.start_destroying(),
            AnyEnvironmentState::DestroyFailed(env) => env.start_destroying(),
            AnyEnvironmentState::Destroyed(_) => {
                unreachable!("Already handled Destroyed state above")
            }
        };

        // 7. Persist intermediate state
        self.repository.save_destroying(&destroying_env)?;

        // 8. Create OpenTofu client with correct build directory
        let opentofu_client = Arc::new(crate::adapters::tofu::client::OpenTofuClient::new(
            opentofu_build_dir,
        ));

        // 9. Execute destruction steps with explicit step tracking
        match Self::execute_destruction_with_tracking(&destroying_env, &opentofu_client) {
            Ok(()) => {
                // Transition to Destroyed state
                let destroyed = destroying_env.destroyed();

                // Persist final state
                self.repository.save_destroyed(&destroyed)?;

                info!(
                    command = "destroy",
                    environment = %destroyed.name(),
                    "Infrastructure destruction completed successfully"
                );

                Ok(destroyed)
            }
            Err((e, current_step)) => {
                // Transition to error state with structured context
                let context =
                    self.build_failure_context(&destroying_env, &e, current_step, started_at);
                let failed = destroying_env.destroy_failed(context);

                // Persist error state
                self.repository.save_destroy_failed(&failed)?;

                Err(e)
            }
        }
    }

    // Private helper methods

    /// Execute the destruction steps with step tracking
    ///
    /// This method executes all destruction steps while tracking which step is currently
    /// being executed. If an error occurs, it returns both the error and the step that
    /// was being executed, enabling accurate failure context generation.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any destruction step fails
    fn execute_destruction_with_tracking(
        environment: &crate::domain::environment::Environment<
            crate::domain::environment::Destroying,
        >,
        opentofu_client: &Arc<crate::adapters::tofu::client::OpenTofuClient>,
    ) -> StepResult<(), DestroyCommandHandlerError, crate::domain::environment::state::DestroyStep>
    {
        use crate::domain::environment::state::DestroyStep;

        // Step 1: Conditionally destroy infrastructure via OpenTofu
        // Only attempt infrastructure destruction if infrastructure was provisioned
        if Self::should_destroy_infrastructure(environment) {
            info!(
                environment = %environment.name(),
                "Destroying provisioned infrastructure"
            );
            Self::destroy_infrastructure(opentofu_client)
                .map_err(|e| (e, DestroyStep::DestroyInfrastructure))?;
        } else {
            info!(
                environment = %environment.name(),
                "Skipping infrastructure destruction (environment was never provisioned)"
            );
        }

        // Step 2: Clean up state files
        Self::cleanup_state_files(environment).map_err(|e| (e, DestroyStep::CleanupStateFiles))?;

        Ok(())
    }

    /// Build structured failure context for destroy command errors
    ///
    /// Creates a comprehensive `DestroyFailureContext` containing all relevant
    /// metadata about the failure including step, timing, error classification,
    /// and trace file location.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being destroyed (for trace directory path)
    /// * `error` - The destroy error that occurred
    /// * `current_step` - The step that was executing when the error occurred
    /// * `started_at` - The timestamp when destruction execution started
    ///
    /// # Returns
    ///
    /// A `DestroyFailureContext` with all failure metadata and trace file path
    fn build_failure_context(
        &self,
        _environment: &crate::domain::environment::Environment<
            crate::domain::environment::Destroying,
        >,
        error: &DestroyCommandHandlerError,
        current_step: crate::domain::environment::state::DestroyStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> crate::domain::environment::state::DestroyFailureContext {
        use crate::application::command_handlers::common::failure_context::build_base_failure_context;
        use crate::domain::environment::state::DestroyFailureContext;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        // Build base failure context using common helper
        let base = build_base_failure_context(&self.clock, started_at, error.to_string());

        // Build handler-specific context
        // Note: Trace file generation not implemented for destroy yet
        DestroyFailureContext {
            failed_step,
            error_kind,
            base,
        }
    }

    /// Check if infrastructure should be destroyed
    ///
    /// Determines whether to attempt infrastructure destruction based on whether
    /// the `OpenTofu` build directory exists. If the directory doesn't exist, it means
    /// no infrastructure was ever provisioned (e.g., environment in Created state).
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being destroyed
    ///
    /// # Returns
    ///
    /// Returns `true` if infrastructure destruction should be attempted, `false` otherwise
    pub(crate) fn should_destroy_infrastructure(
        environment: &Environment<crate::domain::environment::Destroying>,
    ) -> bool {
        let tofu_build_dir = environment.tofu_build_dir();
        tofu_build_dir.exists()
    }

    /// Destroy the infrastructure using `OpenTofu`
    ///
    /// Executes the `OpenTofu` destroy workflow to remove all managed infrastructure.
    ///
    /// # Arguments
    ///
    /// * `opentofu_client` - The `OpenTofu` client configured with the correct build directory
    ///
    /// # Errors
    ///
    /// Returns an error if `OpenTofu` destroy fails
    fn destroy_infrastructure(
        opentofu_client: &Arc<crate::adapters::tofu::client::OpenTofuClient>,
    ) -> Result<(), DestroyCommandHandlerError> {
        DestroyInfrastructureStep::new(Arc::clone(opentofu_client)).execute()?;
        Ok(())
    }

    /// Clean up state files during environment destruction
    ///
    /// Removes the data and build directories for the environment.
    /// This is called as part of the destruction workflow.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment being destroyed
    ///
    /// # Errors
    ///
    /// Returns an error if state file cleanup fails
    pub(crate) fn cleanup_state_files<S>(
        env: &Environment<S>,
    ) -> Result<(), DestroyCommandHandlerError> {
        let data_dir = env.data_dir();
        let build_dir = env.build_dir();

        // Remove data directory if it exists
        if data_dir.exists() {
            std::fs::remove_dir_all(data_dir).map_err(|source| {
                DestroyCommandHandlerError::StateCleanupFailed {
                    path: data_dir.clone(),
                    source,
                }
            })?;
            info!(
                command = "destroy",
                path = %data_dir.display(),
                "Removed state directory"
            );
        }

        // Remove build directory if it exists
        if build_dir.exists() {
            std::fs::remove_dir_all(build_dir).map_err(|source| {
                DestroyCommandHandlerError::StateCleanupFailed {
                    path: build_dir.clone(),
                    source,
                }
            })?;
            info!(
                command = "destroy",
                path = %build_dir.display(),
                "Removed build directory"
            );
        }

        Ok(())
    }
}
