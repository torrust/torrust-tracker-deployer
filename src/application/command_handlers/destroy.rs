//! Infrastructure destruction command handler
//!
//! This module contains the `DestroyCommandHandler` which orchestrates the complete infrastructure
//! teardown workflow including:
//!
//! - Infrastructure destruction via `OpenTofu`
//! - State cleanup and resource verification
//!
//! The command handler handles the complex interaction with deployment tools and ensures
//! proper sequencing of destruction steps.

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::OpenTofuError;
use crate::application::steps::DestroyInfrastructureStep;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::StateTypeError;
use crate::domain::environment::{Destroyed, Environment};
use crate::shared::command::CommandError;
use crate::shared::Traceable;

/// Comprehensive error type for the `DestroyCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum DestroyCommandHandlerError {
    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),

    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    #[error("Failed to clean up state files at '{path}': {source}")]
    StateCleanupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl crate::shared::Traceable for DestroyCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofu(e) => {
                format!("DestroyCommandHandlerError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("DestroyCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("DestroyCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::StateTransition(e) => {
                format!("DestroyCommandHandlerError: Invalid state transition - {e}")
            }
            Self::StateCleanupFailed { path, source } => {
                format!(
                    "DestroyCommandHandlerError: Failed to clean up state files at '{}' - {source}",
                    path.display()
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::StatePersistence(_)
            | Self::StateTransition(_)
            | Self::StateCleanupFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StateTransition(_) => crate::shared::ErrorKind::Configuration,
            Self::StatePersistence(_) | Self::StateCleanupFailed { .. } => {
                crate::shared::ErrorKind::StatePersistence
            }
        }
    }
}

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
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn crate::shared::Clock>,
}

impl DestroyCommandHandler {
    /// Create a new `DestroyCommandHandler`
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn crate::shared::Clock>,
    ) -> Self {
        Self { repository, clock }
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
        self.repository.save(&destroying_env.clone().into_any())?;

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
                self.repository.save(&destroyed.clone().into_any())?;

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
                self.repository.save(&failed.clone().into_any())?;

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
    ) -> Result<
        (),
        (
            DestroyCommandHandlerError,
            crate::domain::environment::state::DestroyStep,
        ),
    > {
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
        use crate::domain::environment::state::{BaseFailureContext, DestroyFailureContext};
        use crate::domain::environment::TraceId;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        let now = self.clock.now();
        let trace_id = TraceId::new();

        // Calculate actual execution duration
        let execution_duration = now
            .signed_duration_since(started_at)
            .to_std()
            .unwrap_or_default();

        // Build context with all failure information
        DestroyFailureContext {
            failed_step,
            error_kind,
            base: BaseFailureContext {
                error_summary: error.to_string(),
                failed_at: now,
                execution_started_at: started_at,
                execution_duration,
                trace_id,
                trace_file_path: None, // Trace file generation not implemented for destroy yet
            },
        }
    }

    // Private helper methods

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
    fn should_destroy_infrastructure(
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
    fn cleanup_state_files<S>(env: &Environment<S>) -> Result<(), DestroyCommandHandlerError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Test builder for `DestroyCommandHandler` that manages dependencies and lifecycle
    ///
    /// This builder simplifies test setup by:
    /// - Managing `TempDir` lifecycle
    /// - Providing sensible defaults for all dependencies
    /// - Allowing selective customization of dependencies
    /// - Returning only the command handler and necessary test artifacts
    pub struct DestroyCommandHandlerTestBuilder {
        temp_dir: TempDir,
    }

    impl DestroyCommandHandlerTestBuilder {
        /// Create a new test builder with default configuration
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            Self { temp_dir }
        }

        /// Build the `DestroyCommandHandler` with all dependencies
        ///
        /// Returns: (`command_handler`, `temp_dir`)
        /// The `temp_dir` must be kept alive for the duration of the test.
        pub fn build(self) -> (DestroyCommandHandler, TempDir) {
            let repository_factory =
                crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                    std::time::Duration::from_secs(30),
                );
            let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

            // Create a system clock for testing
            let clock = Arc::new(crate::shared::SystemClock);

            let command_handler = DestroyCommandHandler::new(repository, clock);

            (command_handler, self.temp_dir)
        }
    }

    #[test]
    fn it_should_create_destroy_command_handler_with_all_dependencies() {
        let (command_handler, _temp_dir) = DestroyCommandHandlerTestBuilder::new().build();

        // Verify the command handler was created (basic structure test)
        // This test just verifies that the command handler can be created with the dependencies
        assert_eq!(Arc::strong_count(&command_handler.repository), 1);
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to DestroyCommandHandlerError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let opentofu_error = OpenTofuError::CommandError(command_error);
        let destroy_error: DestroyCommandHandlerError = opentofu_error.into();
        drop(destroy_error);

        let command_error_direct = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let destroy_error: DestroyCommandHandlerError = command_error_direct.into();
        drop(destroy_error);
    }

    #[test]
    fn it_should_skip_infrastructure_destruction_when_tofu_build_dir_does_not_exist() {
        use crate::domain::environment::testing::EnvironmentTestBuilder;

        // Arrange: Create environment in Created state with no OpenTofu build directory
        let (created_env, _data_dir, _build_dir, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();

        // Transition to Destroying state
        let destroying_env = created_env.start_destroying();

        // Verify tofu_build_dir does not exist
        assert!(
            !destroying_env.tofu_build_dir().exists(),
            "OpenTofu build directory should not exist for Created state"
        );

        // Act: Check if infrastructure should be destroyed
        let should_destroy = DestroyCommandHandler::should_destroy_infrastructure(&destroying_env);

        // Assert: Infrastructure destruction should be skipped
        assert!(
            !should_destroy,
            "Infrastructure destruction should be skipped when tofu_build_dir does not exist"
        );
    }

    #[test]
    fn it_should_attempt_infrastructure_destruction_when_tofu_build_dir_exists() {
        use crate::domain::environment::testing::EnvironmentTestBuilder;

        // Arrange: Create environment with OpenTofu build directory
        let (created_env, _data_dir, _build_dir, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();

        // Create the OpenTofu build directory to simulate provisioned state
        let tofu_build_dir = created_env.tofu_build_dir();
        std::fs::create_dir_all(&tofu_build_dir).expect("Failed to create tofu build dir");

        // Transition to Destroying state
        let destroying_env = created_env.start_destroying();

        // Verify tofu_build_dir exists
        assert!(
            destroying_env.tofu_build_dir().exists(),
            "OpenTofu build directory should exist for provisioned environment"
        );

        // Act: Check if infrastructure should be destroyed
        let should_destroy = DestroyCommandHandler::should_destroy_infrastructure(&destroying_env);

        // Assert: Infrastructure destruction should be attempted
        assert!(
            should_destroy,
            "Infrastructure destruction should be attempted when tofu_build_dir exists"
        );
    }

    #[test]
    fn it_should_clean_up_state_files_regardless_of_infrastructure_state() {
        use crate::domain::environment::testing::EnvironmentTestBuilder;

        // Arrange: Create environment with data and build directories
        let (created_env, data_dir, build_dir, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();

        // Create the directories
        std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
        std::fs::create_dir_all(&build_dir).expect("Failed to create build dir");

        // Create some files in the directories
        std::fs::write(data_dir.join("environment.json"), "{}").expect("Failed to write file");
        std::fs::write(build_dir.join("test.txt"), "test").expect("Failed to write file");

        // Verify directories exist before cleanup
        assert!(data_dir.exists(), "Data directory should exist");
        assert!(build_dir.exists(), "Build directory should exist");

        // Act: Clean up state files
        let result = DestroyCommandHandler::cleanup_state_files(&created_env);

        // Assert: Cleanup succeeded
        assert!(
            result.is_ok(),
            "State file cleanup should succeed: {:?}",
            result.err()
        );

        // Assert: Directories were removed
        assert!(
            !data_dir.exists(),
            "Data directory should be removed after cleanup"
        );
        assert!(
            !build_dir.exists(),
            "Build directory should be removed after cleanup"
        );
    }
}
