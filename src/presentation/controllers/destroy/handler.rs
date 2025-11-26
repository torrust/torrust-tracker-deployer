//! Destroy Command Handler
//!
//! This module handles the destroy command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::DestroyCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Destroyed;
use crate::domain::environment::Environment;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::DestroySubcommandError;

/// Steps in the destroy workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DestroyStep {
    ValidateEnvironment,
    CreateCommandHandler,
    TearDownInfrastructure,
}

impl DestroyStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::CreateCommandHandler,
        Self::TearDownInfrastructure,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::CreateCommandHandler => "Creating command handler",
            Self::TearDownInfrastructure => "Tearing down infrastructure",
        }
    }
}

/// Presentation layer controller for destroy command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `DestroyCommandHandler`.
///
/// # Responsibilities
///
/// - Validate user input (environment name format)
/// - Show progress updates to the user
/// - Format success/error messages for display
/// - Delegate business logic to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual business logic to the application layer's
/// `DestroyCommandHandler`, maintaining clear separation of concerns.
pub struct DestroyCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl DestroyCommandController {
    /// Create a new destroy command controller
    ///
    /// Creates a `DestroyCommandController` with direct repository injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, DestroyStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete destroy workflow
    ///
    /// Orchestrates all steps of the destroy command:
    /// 1. Validate environment name
    /// 2. Create command handler
    /// 3. Tear down infrastructure
    /// 4. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to destroy
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment cannot be loaded from repository
    /// - Infrastructure teardown fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Destroyed>)` on success, or a `DestroySubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(
        &mut self,
        environment_name: &str,
    ) -> Result<Environment<Destroyed>, DestroySubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        let handler = self.create_command_handler()?;

        let destroyed = self.tear_down_infrastructure(&handler, &env_name)?;

        self.complete_workflow(environment_name)?;

        Ok(destroyed)
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, DestroySubcommandError> {
        self.progress
            .start_step(DestroyStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            DestroySubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Create application layer command handler
    ///
    /// Creates the application layer command handler with all required
    /// dependencies (repository, clock, etc.).
    #[allow(clippy::result_large_err)]
    fn create_command_handler(&mut self) -> Result<DestroyCommandHandler, DestroySubcommandError> {
        self.progress
            .start_step(DestroyStep::CreateCommandHandler.description())?;
        let handler = DestroyCommandHandler::new(self.repository.clone(), self.clock.clone());
        self.progress.complete_step(None)?;

        Ok(handler)
    }

    /// Execute infrastructure teardown via application layer
    ///
    /// Delegates to the application layer `DestroyCommandHandler` to
    /// orchestrate the actual infrastructure destruction workflow.
    #[allow(clippy::result_large_err)]
    fn tear_down_infrastructure(
        &mut self,
        handler: &DestroyCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Destroyed>, DestroySubcommandError> {
        self.progress
            .start_step(DestroyStep::TearDownInfrastructure.description())?;

        let destroyed = handler.execute(env_name).map_err(|source| {
            DestroySubcommandError::DestroyOperationFailed {
                name: env_name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some("Infrastructure torn down"))?;
        Ok(destroyed)
    }

    /// Complete the workflow with success message
    ///
    /// Shows final success message to the user with workflow summary.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(&mut self, name: &str) -> Result<(), DestroySubcommandError> {
        self.progress
            .complete(&format!("Environment '{name}' destroyed successfully"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use crate::shared::SystemClock;
    use std::fs;
    use tempfile::TempDir;

    /// Create test dependencies for destroy command handler tests
    ///
    /// Returns the common dependencies needed for testing `handle_destroy_command`:
    /// - `user_output`: `ReentrantMutex`-wrapped `UserOutput` for thread-safe access
    /// - `repository`: Environment repository with Send + Sync bounds
    /// - `clock`: System clock for timing operations
    #[allow(clippy::type_complexity)] // Test helper with complex but clear types
    fn create_test_dependencies(
        temp_dir: &TempDir,
    ) -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<dyn EnvironmentRepository + Send + Sync>,
        Arc<dyn Clock>,
    ) {
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let data_dir = temp_dir.path().join("data");
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(data_dir);
        let clock = Arc::new(SystemClock);

        (user_output, repository, clock)
    }

    #[tokio::test]
    async fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test with invalid environment name (contains underscore)
        let result = DestroyCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = DestroyCommandController::new(repository, clock, user_output.clone())
            .execute("")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Try to destroy an environment that doesn't exist
        let result = DestroyCommandController::new(repository, clock, user_output.clone())
            .execute("nonexistent-env")
            .await;

        assert!(result.is_err());
        // Should get DestroyOperationFailed because environment doesn't exist
        match result.unwrap_err() {
            DestroySubcommandError::DestroyOperationFailed { name, .. } => {
                assert_eq!(name, "nonexistent-env");
            }
            other => panic!("Expected DestroyOperationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_accept_valid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at destroy operation since we don't have a real environment setup
        let result = DestroyCommandController::new(repository, clock, user_output.clone())
            .execute("test-env")
            .await;

        // Should fail at operation, not at name validation
        if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
