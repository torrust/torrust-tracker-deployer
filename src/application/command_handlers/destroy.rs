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
/// 2. Transition environment to `Destroyed` state
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<S>` (any state) as input
/// - Returns `Environment<Destroyed>` on success
///
/// State is persisted after destruction using the injected repository.
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
}

impl DestroyCommandHandler {
    /// Create a new `DestroyCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
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
    /// On error, cleanup may be partial. The user should manually verify
    /// and complete the cleanup if necessary.
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

        // 4. Get the build directory from the environment context
        let opentofu_build_dir = environment.tofu_build_dir();

        // 5. Create OpenTofu client with correct build directory
        let opentofu_client = Arc::new(crate::adapters::tofu::client::OpenTofuClient::new(
            opentofu_build_dir,
        ));

        // 6. Execute infrastructure destruction
        // OpenTofu destroy is idempotent - it will succeed even if infrastructure doesn't exist
        Self::destroy_infrastructure(&opentofu_client)?;

        // 7. Transition to Destroyed state based on current state
        let destroyed = environment.destroy()?;

        // 8. Clean up state files only after successful infrastructure destruction
        Self::cleanup_state_files(&destroyed)?;

        // 9. Persist final state
        self.repository.save(&destroyed.clone().into_any())?;

        info!(
            command = "destroy",
            environment = %destroyed.name(),
            "Infrastructure destruction completed successfully"
        );

        Ok(destroyed)
    }

    // Private helper methods

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

    /// Clean up state files after successful infrastructure destruction
    ///
    /// Removes the data and build directories for the environment.
    /// This is only called after infrastructure destruction succeeds.
    ///
    /// # Arguments
    ///
    /// * `env` - The destroyed environment
    ///
    /// # Errors
    ///
    /// Returns an error if state file cleanup fails
    fn cleanup_state_files(env: &Environment<Destroyed>) -> Result<(), DestroyCommandHandlerError> {
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

            let command_handler = DestroyCommandHandler::new(repository);

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
}
