//! Configure Command Handler
//!
//! This module handles the configure command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::ConfigureCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Configured;
use crate::domain::environment::Environment;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::configure::{
    ConfigureDetailsData, JsonView, TextView,
};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::progress::VerboseProgressListener;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ConfigureSubcommandError;

/// Steps in the configure workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigureStep {
    ValidateEnvironment,
    CreateCommandHandler,
    ConfigureInfrastructure,
}

impl ConfigureStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::CreateCommandHandler,
        Self::ConfigureInfrastructure,
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
            Self::ConfigureInfrastructure => "Configuring infrastructure",
        }
    }
}

/// Presentation layer controller for configure command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `ConfigureCommandHandler`.
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
/// `ConfigureCommandHandler`, maintaining clear separation of concerns.
pub struct ConfigureCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl ConfigureCommandController {
    /// Create a new configure command controller
    ///
    /// Creates a `ConfigureCommandController` with direct service injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, ConfigureStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete configure workflow
    ///
    /// Orchestrates all steps of the configure command:
    /// 1. Validate environment name
    /// 2. Load and validate environment state
    /// 3. Create command handler
    /// 4. Configure infrastructure
    /// 5. Display results (in specified format)
    /// 6. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to configure
    /// * `output_format` - The output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment cannot be loaded from repository
    /// - Environment is not in "Provisioned" state
    /// - Infrastructure configuration fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Configured>)` on success, or a `ConfigureSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    pub fn execute(
        &mut self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<Environment<Configured>, ConfigureSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        let handler = self.create_command_handler()?;

        let configured = self.configure_infrastructure(&handler, &env_name)?;

        self.complete_workflow(environment_name)?;

        self.display_configure_results(&configured, output_format)?;

        Ok(configured)
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ConfigureSubcommandError> {
        self.progress
            .start_step(ConfigureStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ConfigureSubcommandError::InvalidEnvironmentName {
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
    ) -> Result<ConfigureCommandHandler, ConfigureSubcommandError> {
        self.progress
            .start_step(ConfigureStep::CreateCommandHandler.description())?;

        let handler = ConfigureCommandHandler::new(self.clock.clone(), self.repository.clone());
        self.progress.complete_step(None)?;

        Ok(handler)
    }

    /// Execute infrastructure configuration via application layer
    ///
    /// Delegates to the application layer `ConfigureCommandHandler` to
    /// orchestrate the actual infrastructure configuration workflow.
    ///
    /// The application layer handles:
    /// - Loading the environment from repository
    /// - Validating the environment state (must be Provisioned)
    /// - Complete configuration workflow
    /// - State transitions and persistence
    #[allow(clippy::result_large_err)]
    fn configure_infrastructure(
        &mut self,
        handler: &ConfigureCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Configured>, ConfigureSubcommandError> {
        self.progress
            .start_step(ConfigureStep::ConfigureInfrastructure.description())?;

        // Create the listener for verbose progress reporting.
        // The VerboseProgressListener translates step events into
        // user-facing detail messages via UserOutput's verbosity filter.
        let listener = VerboseProgressListener::new(self.progress.output().clone());

        let configured = handler
            .execute(env_name, Some(&listener))
            .map_err(
                |source| ConfigureSubcommandError::ConfigureOperationFailed {
                    name: env_name.to_string(),
                    source: Box::new(source),
                },
            )?;

        self.progress
            .complete_step(Some("Infrastructure configured"))?;
        Ok(configured)
    }

    /// Complete the workflow with success message
    ///
    /// Shows final success message to the user with workflow summary.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(&mut self, name: &str) -> Result<(), ConfigureSubcommandError> {
        self.progress
            .complete(&format!("Environment '{name}' configured successfully"))?;
        Ok(())
    }

    /// Display configure results in the specified format
    ///
    /// Uses the Strategy Pattern to render configure details in either
    /// human-readable text or machine-readable JSON format.
    ///
    /// # Arguments
    ///
    /// * `configured` - The configured environment to display
    /// * `output_format` - The output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Note
    ///
    /// JSON serialization errors are propagated as `ConfigureSubcommandError::OutputFormatting`.
    #[allow(clippy::result_large_err)]
    fn display_configure_results(
        &mut self,
        configured: &Environment<Configured>,
        output_format: OutputFormat,
    ) -> Result<(), ConfigureSubcommandError> {
        self.progress.blank_line()?;
        let details = ConfigureDetailsData::from(configured);
        let output = match output_format {
            OutputFormat::Text => TextView::render(&details)?,
            OutputFormat::Json => JsonView::render(&details)?,
        };
        self.progress.result(&output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::cli::views::testing::TestUserOutput;
    use crate::presentation::cli::views::VerbosityLevel;
    use crate::shared::SystemClock;

    /// Create test dependencies for configure command handler tests
    ///
    /// Returns the common dependencies needed for testing `handle_configure_command`:
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
        use crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let data_dir = temp_dir.path().join("data");
        let file_repository_factory = FileRepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = file_repository_factory.create(data_dir);
        let clock = Arc::new(SystemClock);

        (user_output, repository, clock)
    }

    #[tokio::test]
    async fn it_should_return_error_for_invalid_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test with invalid environment name (contains underscore)
        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name", OutputFormat::Text);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigureSubcommandError::InvalidEnvironmentName { name, .. } => {
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

        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("", OutputFormat::Text);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigureSubcommandError::InvalidEnvironmentName { name, .. } => {
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

        // Try to configure an environment that doesn't exist
        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("nonexistent-env", OutputFormat::Text);

        assert!(result.is_err());
        // After refactoring, repository NotFound error is wrapped in ConfigureOperationFailed
        match result.unwrap_err() {
            ConfigureSubcommandError::ConfigureOperationFailed { name, .. } => {
                assert_eq!(name, "nonexistent-env");
            }
            other => panic!("Expected ConfigureOperationFailed, got: {other:?}"),
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
        // at configure operation since we don't have a real environment setup
        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("test-env", OutputFormat::Text);

        // Should fail at operation, not at name validation
        if let Err(ConfigureSubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
