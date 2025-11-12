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
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::progress::ProgressReporter;
use crate::presentation::user_output::UserOutput;
use crate::shared::clock::Clock;

use super::errors::DestroySubcommandError;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of main steps in the destroy workflow
const DESTROY_WORKFLOW_STEPS: usize = 3;

// ============================================================================
// HIGH-LEVEL API (EXECUTION CONTEXT PATTERN)
// ============================================================================

/// Handle destroy command using `ExecutionContext` pattern
///
/// This function provides a clean interface for destroying deployment environments,
/// integrating with the `ExecutionContext` pattern for dependency injection.
///
/// # Arguments
///
/// * `environment_name` - Name of the environment to destroy
/// * `working_dir` - Working directory path for operations
/// * `context` - Execution context providing access to services
///
/// # Returns
///
/// * `Ok(Environment<Destroyed>)` - Environment destroyed successfully
/// * `Err(DestroySubcommandError)` - Destroy operation failed
///
/// # Errors
///
/// Returns `DestroySubcommandError` when:
/// * Environment name is invalid or contains special characters
/// * Working directory is not accessible or doesn't exist
/// * Environment is not found in the working directory
/// * Infrastructure destruction fails (OpenTofu/LXD errors)
/// * File system operations fail (permission errors, disk space)
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::presentation::controllers::destroy;
/// use torrust_tracker_deployer_lib::presentation::dispatch::context::ExecutionContext;
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let container = Arc::new(Container::new(VerbosityLevel::Normal));
/// let context = ExecutionContext::new(container);
/// let working_dir = Path::new("./test");
///
/// destroy::handle("my-env", working_dir, &context)?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(
    environment_name: &str,
    working_dir: &std::path::Path,
    context: &crate::presentation::dispatch::context::ExecutionContext,
) -> Result<Environment<Destroyed>, DestroySubcommandError> {
    handle_destroy_command(
        environment_name,
        working_dir,
        context.repository_factory(),
        context.clock(),
        &context.user_output(),
    )
}

// ============================================================================
// INTERMEDIATE API (DIRECT DEPENDENCY INJECTION)
// ============================================================================

/// Handle the destroy command
///
/// This is a thin wrapper over `DestroyCommandController` that serves as
/// the public entry point for the destroy command.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to destroy
/// * `working_dir` - Root directory for environment data storage
/// * `repository_factory` - Factory for creating environment repositories
/// * `clock` - Clock service for timing operations
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Errors
///
/// Returns an error if:
/// - Environment name is invalid (format validation fails)
/// - Environment cannot be loaded from repository
/// - Infrastructure teardown fails
/// - Progress reporting encounters a poisoned mutex
///
/// All errors include detailed context and actionable troubleshooting guidance.
///
/// # Returns
///
/// Returns `Ok(Environment<Destroyed>)` on success, or a `DestroySubcommandError` on failure.
///
/// # Example
///
/// Using with Container and `ExecutionContext` (recommended):
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
/// use torrust_tracker_deployer_lib::presentation::controllers::destroy;
/// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
///
/// let container = Container::new(VerbosityLevel::Normal);
/// let context = ExecutionContext::new(Arc::new(container));
///
/// if let Err(e) = destroy::handle("test-env", Path::new("."), &context) {
///     eprintln!("Destroy failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
///
/// Direct usage (for testing or specialized scenarios):
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use parking_lot::ReentrantMutex;
/// use std::cell::RefCell;
/// use torrust_tracker_deployer_lib::presentation::controllers::destroy;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
/// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
/// use torrust_tracker_deployer_lib::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
/// use torrust_tracker_deployer_lib::shared::SystemClock;
///
/// let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
/// let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
/// let clock = Arc::new(SystemClock);
/// if let Err(e) = destroy::handle_destroy_command("test-env", Path::new("."), repository_factory, clock, &user_output) {
///     eprintln!("Destroy failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
#[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
pub fn handle_destroy_command(
    environment_name: &str,
    working_dir: &std::path::Path,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<Environment<Destroyed>, DestroySubcommandError> {
    DestroyCommandController::new(
        working_dir.to_path_buf(),
        repository_factory,
        clock,
        user_output.clone(),
    )
    .execute(environment_name)
}

// ============================================================================
// PRESENTATION LAYER CONTROLLER (IMPLEMENTATION DETAILS)
// ============================================================================

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
#[allow(unused)] // Temporary during refactoring
pub struct DestroyCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

impl DestroyCommandController {
    /// Create a new destroy command controller from working directory
    ///
    /// Creates a `DestroyCommandController` with direct service injection from working directory and user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        working_dir: std::path::PathBuf,
        repository_factory: Arc<RepositoryFactory>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let repository = repository_factory.create(working_dir);
        let progress = ProgressReporter::new(user_output.clone(), DESTROY_WORKFLOW_STEPS);

        Self {
            repository,
            clock,
            user_output,
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
    pub fn execute(
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
        self.progress.start_step("Validating environment")?;

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
        self.progress.start_step("Creating command handler")?;
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
        self.progress.start_step("Tearing down infrastructure")?;

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
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::user_output::test_support::TestUserOutput;
    use crate::presentation::user_output::VerbosityLevel;
    use crate::shared::SystemClock;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        // Test with invalid environment name (contains underscore)
        let result = handle_destroy_command(
            "invalid_name",
            working_dir,
            repository_factory,
            clock,
            &user_output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        let result =
            handle_destroy_command("", working_dir, repository_factory, clock, &user_output);

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        // Try to destroy an environment that doesn't exist
        let result = handle_destroy_command(
            "nonexistent-env",
            working_dir,
            repository_factory,
            clock,
            &user_output,
        );

        assert!(result.is_err());
        // Should get DestroyOperationFailed because environment doesn't exist
        match result.unwrap_err() {
            DestroySubcommandError::DestroyOperationFailed { name, .. } => {
                assert_eq!(name, "nonexistent-env");
            }
            other => panic!("Expected DestroyOperationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_accept_valid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at destroy operation since we don't have a real environment setup
        let result = handle_destroy_command(
            "test-env",
            working_dir,
            repository_factory,
            clock,
            &user_output,
        );

        // Should fail at operation, not at name validation
        if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
