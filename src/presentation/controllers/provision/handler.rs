//! Provision Command Handler
//!
//! This module handles the provision command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::ProvisionCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ProvisionSubcommandError;

/// Number of main steps in the provision workflow
const PROVISION_WORKFLOW_STEPS: usize = 3;

/// Presentation layer controller for provision command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `ProvisionCommandHandler`.
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
/// `ProvisionCommandHandler`, maintaining clear separation of concerns.
pub struct ProvisionCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl ProvisionCommandController {
    /// Create a new provision command controller
    ///
    /// Creates a `ProvisionCommandController` with direct service injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, PROVISION_WORKFLOW_STEPS);

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete provision workflow
    ///
    /// Orchestrates all steps of the provision command:
    /// 1. Validate environment name
    /// 2. Load and validate environment state
    /// 3. Create command handler
    /// 4. Provision infrastructure
    /// 5. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to provision
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment cannot be loaded from repository
    /// - Environment is not in "Created" state
    /// - Infrastructure provisioning fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Provisioned>)` on success, or a `ProvisionSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    pub async fn execute(
        &mut self,
        environment_name: &str,
    ) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        let handler = self.create_command_handler()?;

        let provisioned = self.provision_infrastructure(&handler, &env_name).await?;

        self.complete_workflow(environment_name)?;

        Ok(provisioned)
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ProvisionSubcommandError> {
        self.progress.start_step("Validating environment")?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ProvisionSubcommandError::InvalidEnvironmentName {
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
    /// dependencies (repository, clock).
    #[allow(clippy::result_large_err)]
    fn create_command_handler(
        &mut self,
    ) -> Result<ProvisionCommandHandler, ProvisionSubcommandError> {
        self.progress.start_step("Creating command handler")?;
        let handler = ProvisionCommandHandler::new(self.clock.clone(), self.repository.clone());
        self.progress.complete_step(None)?;

        Ok(handler)
    }

    /// Execute infrastructure provisioning via application layer
    ///
    /// Delegates to the application layer `ProvisionCommandHandler` to
    /// orchestrate the actual infrastructure provisioning workflow.
    ///
    /// The application layer handles:
    /// - Loading the environment from repository
    /// - Validating the environment state (must be Created)
    /// - Complete provisioning workflow
    /// - State transitions and persistence
    #[allow(clippy::result_large_err)]
    async fn provision_infrastructure(
        &mut self,
        handler: &ProvisionCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
        self.progress.start_step("Provisioning infrastructure")?;

        let provisioned = handler.execute(env_name).await.map_err(|source| {
            ProvisionSubcommandError::ProvisionOperationFailed {
                name: env_name.to_string(),
                source: Box::new(source),
            }
        })?;

        self.progress
            .complete_step(Some("Infrastructure provisioned"))?;
        Ok(provisioned)
    }

    /// Complete the workflow with success message
    ///
    /// Shows final success message to the user with workflow summary.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(&mut self, name: &str) -> Result<(), ProvisionSubcommandError> {
        self.progress
            .complete(&format!("Environment '{name}' provisioned successfully"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use crate::shared::SystemClock;

    /// Create test dependencies for provision command handler tests
    ///
    /// Returns the common dependencies needed for testing `handle_provision_command`:
    /// - `user_output`: `ReentrantMutex`-wrapped `UserOutput` for thread-safe access
    /// - `repository`: Environment repository for persistence
    /// - `clock`: System clock for timing operations
    #[allow(clippy::type_complexity)] // Test helper with complex but clear types
    fn create_test_dependencies(
        temp_dir: &tempfile::TempDir,
    ) -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<dyn EnvironmentRepository + Send + Sync>,
        Arc<dyn Clock>,
    ) {
        use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
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
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test with invalid environment name (contains underscore)
        let result = ProvisionCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ProvisionSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_empty_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ProvisionCommandController::new(repository, clock, user_output.clone())
            .execute("")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ProvisionSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_nonexistent_environment() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test environment that doesn't exist yet
        let result = ProvisionCommandController::new(repository, clock, user_output.clone())
            .execute("non-existent-env")
            .await;

        assert!(result.is_err());
        // After refactoring, repository NotFound error is wrapped in ProvisionOperationFailed
        match result.unwrap_err() {
            ProvisionSubcommandError::ProvisionOperationFailed { name, .. } => {
                assert_eq!(name, "non-existent-env");
            }
            other => panic!("Expected ProvisionOperationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_accept_valid_environment_name() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at provision operation since we don't have a real environment setup
        let result = ProvisionCommandController::new(repository, clock, user_output.clone())
            .execute("test-env")
            .await;

        // Should fail at operation, not at name validation
        if let Err(ProvisionSubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
