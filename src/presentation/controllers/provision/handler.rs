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
use crate::domain::TemplateManager;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::dispatch::context::ExecutionContext;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ProvisionSubcommandError;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of main steps in the provision workflow
const PROVISION_WORKFLOW_STEPS: usize = 9;

// ============================================================================
// HIGH-LEVEL API (EXECUTION CONTEXT PATTERN)
// ============================================================================

/// Handle provision command using `ExecutionContext` pattern
///
/// This function provides a clean interface for provisioning deployment environments,
/// integrating with the `ExecutionContext` pattern for dependency injection.
///
/// # Arguments
///
/// * `environment_name` - Name of the environment to provision
/// * `working_dir` - Working directory path for operations
/// * `context` - Execution context providing access to services
///
/// # Returns
///
/// * `Ok(Environment<Provisioned>)` - Environment provisioned successfully
/// * `Err(ProvisionSubcommandError)` - Provision operation failed
///
/// # Errors
///
/// Returns `ProvisionSubcommandError` when:
/// * Environment name is invalid or contains special characters
/// * Working directory is not accessible or doesn't exist
/// * Environment is not found or not in "Created" state
/// * Infrastructure provisioning fails (OpenTofu/LXD errors)
/// * SSH connectivity cannot be established
/// * Cloud-init does not complete successfully
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::presentation::controllers::provision;
/// use torrust_tracker_deployer_lib::presentation::dispatch::context::ExecutionContext;
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let container = Arc::new(Container::new(VerbosityLevel::Normal));
/// let context = ExecutionContext::new(container);
/// let working_dir = Path::new("./test");
///
/// provision::handle("my-env", working_dir, &context)?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(
    environment_name: &str,
    working_dir: &std::path::Path,
    context: &ExecutionContext,
) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
    handle_provision_command(
        environment_name,
        working_dir,
        context.repository_factory(),
        context.clock(),
        context.template_manager(),
        &context.user_output(),
    )
}

// ============================================================================
// INTERMEDIATE API (DIRECT DEPENDENCY INJECTION)
// ============================================================================

/// Handle the provision command
///
/// This is a thin wrapper over `ProvisionCommandController` that serves as
/// the public entry point for the provision command.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to provision
/// * `working_dir` - Root directory for environment data storage
/// * `repository_factory` - Factory for creating environment repositories
/// * `clock` - Clock service for timing operations
/// * `template_manager` - Template manager for rendering configuration files
/// * `user_output` - Shared user output service for consistent output formatting
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
/// All errors include detailed context and actionable troubleshooting guidance.
///
/// # Returns
///
/// Returns `Ok(Environment<Provisioned>)` on success, or a `ProvisionSubcommandError` on failure.
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
/// use torrust_tracker_deployer_lib::presentation::controllers::provision;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let container = Container::new(VerbosityLevel::Normal);
/// let context = ExecutionContext::new(Arc::new(container));
///
/// if let Err(e) = provision::handle("test-env", Path::new("."), &context) {
///     eprintln!("Provision failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
///
/// Direct usage (for testing or specialized scenarios):
///
/// ```rust
/// use std::path::{Path, PathBuf};
/// use std::sync::Arc;
/// use parking_lot::ReentrantMutex;
/// use std::cell::RefCell;
/// use torrust_tracker_deployer_lib::presentation::controllers::provision;
/// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
/// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
/// use torrust_tracker_deployer_lib::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
/// use torrust_tracker_deployer_lib::shared::SystemClock;
/// use torrust_tracker_deployer_lib::domain::TemplateManager;
///
/// let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
/// let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
/// let clock = Arc::new(SystemClock);
/// let template_manager = Arc::new(TemplateManager::new(PathBuf::from("templates")));
/// if let Err(e) = provision::handle_provision_command("test-env", Path::new("."), repository_factory, clock, template_manager, &user_output) {
///     eprintln!("Provision failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
#[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
pub fn handle_provision_command(
    environment_name: &str,
    working_dir: &std::path::Path,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    template_manager: Arc<TemplateManager>,
    user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
    ProvisionCommandController::new(
        working_dir.to_path_buf(),
        repository_factory,
        clock,
        template_manager,
        user_output.clone(),
    )
    .execute(environment_name)
}

// ============================================================================
// PRESENTATION LAYER CONTROLLER (IMPLEMENTATION DETAILS)
// ============================================================================

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
#[allow(unused)] // Temporary during refactoring
pub struct ProvisionCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    template_manager: Arc<TemplateManager>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

impl ProvisionCommandController {
    /// Create a new provision command controller from working directory
    ///
    /// Creates a `ProvisionCommandController` with direct service injection from working directory and user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        working_dir: std::path::PathBuf,
        repository_factory: Arc<RepositoryFactory>,
        clock: Arc<dyn Clock>,
        template_manager: Arc<TemplateManager>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let repository = repository_factory.create(working_dir);
        let progress = ProgressReporter::new(user_output.clone(), PROVISION_WORKFLOW_STEPS);

        Self {
            repository,
            repository_factory,
            clock,
            template_manager,
            user_output,
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
    pub fn execute(
        &mut self,
        environment_name: &str,
    ) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        let handler = self.create_command_handler()?;

        let provisioned = self.provision_infrastructure(&handler, &env_name)?;

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
    /// dependencies (repository, clock, template manager, etc.).
    #[allow(clippy::result_large_err)]
    fn create_command_handler(
        &mut self,
    ) -> Result<ProvisionCommandHandler, ProvisionSubcommandError> {
        self.progress.start_step("Creating command handler")?;
        let handler = ProvisionCommandHandler::new(
            self.clock.clone(),
            self.template_manager.clone(),
            self.repository.clone(),
        );
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
    fn provision_infrastructure(
        &mut self,
        handler: &ProvisionCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
        self.progress.start_step("Provisioning infrastructure")?;

        // Use tokio runtime to execute async handler
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            ProvisionSubcommandError::RepositoryAccessFailed {
                data_dir: "runtime".to_string(),
                reason: format!("Failed to create tokio runtime: {e}"),
            }
        })?;

        let provisioned = runtime
            .block_on(handler.execute(env_name))
            .map_err(
                |source| ProvisionSubcommandError::ProvisionOperationFailed {
                    name: env_name.to_string(),
                    source,
                },
            )?;

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
    /// - `repository_factory`: Factory for creating environment repositories
    /// - `clock`: System clock for timing operations
    /// - `template_manager`: Template manager for rendering configuration files
    #[allow(clippy::type_complexity)] // Test helper with complex but clear types
    fn create_test_dependencies() -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<RepositoryFactory>,
        Arc<dyn Clock>,
        Arc<TemplateManager>,
    ) {
        use tempfile::TempDir;

        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        // Create a temporary templates directory for testing
        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        (user_output, repository_factory, clock, template_manager)
    }

    #[test]
    fn it_should_return_error_for_invalid_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository_factory, clock, template_manager) = create_test_dependencies();

        // Test with invalid environment name (contains underscore)
        let result = handle_provision_command(
            "invalid_name",
            temp_dir.path(),
            repository_factory,
            clock,
            template_manager,
            &user_output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ProvisionSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_empty_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository_factory, clock, template_manager) = create_test_dependencies();

        let result = handle_provision_command(
            "",
            temp_dir.path(),
            repository_factory,
            clock,
            template_manager,
            &user_output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ProvisionSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_nonexistent_environment() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository_factory, clock, template_manager) = create_test_dependencies();

        // Try to provision an environment that doesn't exist
        let result = handle_provision_command(
            "nonexistent-env",
            temp_dir.path(),
            repository_factory,
            clock,
            template_manager,
            &user_output,
        );

        assert!(result.is_err());
        // After refactoring, repository NotFound error is wrapped in ProvisionOperationFailed
        match result.unwrap_err() {
            ProvisionSubcommandError::ProvisionOperationFailed { name, .. } => {
                assert_eq!(name, "nonexistent-env");
            }
            other => panic!("Expected ProvisionOperationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_accept_valid_environment_name() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        let (user_output, repository_factory, clock, template_manager) = create_test_dependencies();

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at provision operation since we don't have a real environment setup
        let result = handle_provision_command(
            "test-env",
            working_dir,
            repository_factory,
            clock,
            template_manager,
            &user_output,
        );

        // Should fail at operation, not at name validation
        if let Err(ProvisionSubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
