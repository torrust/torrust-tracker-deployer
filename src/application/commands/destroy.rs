//! Infrastructure destruction command
//!
//! This module contains the `DestroyCommand` which orchestrates the complete infrastructure
//! teardown workflow including:
//!
//! - Infrastructure destruction via `OpenTofu`
//! - State cleanup and resource verification
//!
//! The command handles the complex interaction with deployment tools and ensures
//! proper sequencing of destruction steps.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::OpenTofuError;
use crate::application::steps::DestroyInfrastructureStep;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::{Destroyed, Environment};
use crate::shared::command::CommandError;

/// Comprehensive error type for the `DestroyCommand`
#[derive(Debug, thiserror::Error)]
pub enum DestroyCommandError {
    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for DestroyCommandError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofu(e) => {
                format!("DestroyCommandError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("DestroyCommandError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("DestroyCommandError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}

/// `DestroyCommand` orchestrates the complete infrastructure destruction workflow
///
/// The `DestroyCommand` orchestrates the complete infrastructure teardown workflow.
///
/// This command handles all steps required to destroy infrastructure:
/// 1. Destroy infrastructure via `OpenTofu`
/// 2. Transition environment to `Destroyed` state
///
/// # State Management
///
/// The command integrates with the type-state pattern for environment lifecycle:
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
pub struct DestroyCommand {
    opentofu_client: Arc<crate::adapters::tofu::client::OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl DestroyCommand {
    /// Create a new `DestroyCommand`
    #[must_use]
    pub fn new(
        opentofu_client: Arc<crate::adapters::tofu::client::OpenTofuClient>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            opentofu_client,
            repository,
        }
    }

    /// Execute the complete destruction workflow
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment to destroy (can be in any state)
    ///
    /// # Returns
    ///
    /// Returns the destroyed environment
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the destruction workflow fails:
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
            environment = %environment.name()
        )
    )]
    pub fn execute<S>(
        &self,
        environment: Environment<S>,
    ) -> Result<Environment<Destroyed>, DestroyCommandError> {
        info!(
            command = "destroy",
            environment = %environment.name(),
            "Starting complete infrastructure destruction workflow"
        );

        // Execute destruction steps
        self.destroy_infrastructure()?;

        // Transition to Destroyed state
        let destroyed = environment.destroy();

        // Persist final state
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
    /// # Errors
    ///
    /// Returns an error if `OpenTofu` destroy fails
    fn destroy_infrastructure(&self) -> Result<(), DestroyCommandError> {
        DestroyInfrastructureStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Test builder for `DestroyCommand` that manages dependencies and lifecycle
    ///
    /// This builder simplifies test setup by:
    /// - Managing `TempDir` lifecycle
    /// - Providing sensible defaults for all dependencies
    /// - Allowing selective customization of dependencies
    /// - Returning only the command and necessary test artifacts
    pub struct DestroyCommandTestBuilder {
        temp_dir: TempDir,
    }

    impl DestroyCommandTestBuilder {
        /// Create a new test builder with default configuration
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            Self { temp_dir }
        }

        /// Build the `DestroyCommand` with all dependencies
        ///
        /// Returns: (`command`, `temp_dir`)
        /// The `temp_dir` must be kept alive for the duration of the test.
        pub fn build(self) -> (DestroyCommand, TempDir) {
            let opentofu_client = Arc::new(crate::adapters::tofu::client::OpenTofuClient::new(
                self.temp_dir.path(),
            ));

            let repository_factory =
                crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                    std::time::Duration::from_secs(30),
                );
            let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

            let command = DestroyCommand::new(opentofu_client, repository);

            (command, self.temp_dir)
        }
    }

    #[test]
    fn it_should_create_destroy_command_with_all_dependencies() {
        let (command, _temp_dir) = DestroyCommandTestBuilder::new().build();

        // Verify the command was created (basic structure test)
        // This test just verifies that the command can be created with the dependencies
        assert_eq!(Arc::strong_count(&command.opentofu_client), 1);
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to DestroyCommandError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let opentofu_error = OpenTofuError::CommandError(command_error);
        let destroy_error: DestroyCommandError = opentofu_error.into();
        drop(destroy_error);

        let command_error_direct = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let destroy_error: DestroyCommandError = command_error_direct.into();
        drop(destroy_error);
    }
}
