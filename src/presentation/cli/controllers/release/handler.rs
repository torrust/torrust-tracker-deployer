//! Release Command Handler
//!
//! This module handles the release command execution at the presentation layer,
//! including environment validation, state validation, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;
use tracing::info;

use crate::application::command_handlers::release::ReleaseCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Released;
use crate::domain::environment::Environment;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::release::{JsonView, ReleaseDetailsData, TextView};
use crate::presentation::cli::views::progress::{ProgressReporter, VerboseProgressListener};
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ReleaseSubcommandError;

/// Steps in the release workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReleaseStep {
    ValidateEnvironment,
    ReleaseApplication,
}

impl ReleaseStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[Self::ValidateEnvironment, Self::ReleaseApplication];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::ReleaseApplication => "Releasing application",
        }
    }
}

/// Presentation layer controller for release command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `ReleaseCommandHandler`.
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
/// `ReleaseCommandHandler`.
pub struct ReleaseCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl ReleaseCommandController {
    /// Create a new release command controller
    ///
    /// Creates a `ReleaseCommandController` with direct repository injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, ReleaseStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete release workflow
    ///
    /// Orchestrates all steps of the release command:
    /// 1. Validate environment name
    /// 2. Execute release via application handler
    /// 3. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to release to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment is not in the Configured state
    /// - Docker Compose file preparation fails
    /// - State persistence fails
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ReleaseSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    pub async fn execute(
        &mut self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<(), ReleaseSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        let released_env = self.release_application(&env_name).await?;

        self.complete_workflow(&released_env, output_format)?;

        Ok(())
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ReleaseSubcommandError> {
        self.progress
            .start_step(ReleaseStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ReleaseSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Release application to the environment
    ///
    /// Calls the application layer handler to execute the release workflow.
    #[allow(clippy::result_large_err)]
    async fn release_application(
        &mut self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Released>, ReleaseSubcommandError> {
        self.progress
            .start_step(ReleaseStep::ReleaseApplication.description())?;

        let handler = ReleaseCommandHandler::new(self.repository.clone(), self.clock.clone());

        // Create the listener for verbose progress reporting.
        // The VerboseProgressListener translates step events into
        // user-facing detail messages via UserOutput's verbosity filter.
        let listener = VerboseProgressListener::new(self.progress.output().clone());

        let released_env = handler
            .execute(env_name, Some(&listener))
            .await
            .map_err(|source| ReleaseSubcommandError::ApplicationLayerError { source })?;

        info!(
            environment = %env_name,
            final_state = "Released",
            "Application released successfully"
        );

        self.progress
            .complete_step(Some("Application released successfully"))?;

        Ok(released_env)
    }

    /// Complete the workflow with environment details output
    ///
    /// Renders the released environment details using the chosen output format
    /// (text or JSON) and displays them to the user.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(
        &mut self,
        released_env: &Environment<Released>,
        output_format: OutputFormat,
    ) -> Result<(), ReleaseSubcommandError> {
        let details = ReleaseDetailsData::from(released_env);

        let output = match output_format {
            OutputFormat::Text => TextView::render(&details),
            OutputFormat::Json => JsonView::render(&details)?,
        };

        self.progress.result(&output)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;
    use crate::presentation::cli::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::cli::views::testing::TestUserOutput;
    use crate::presentation::cli::views::VerbosityLevel;
    use crate::shared::SystemClock;
    use tempfile::TempDir;

    /// Create test dependencies for release command handler tests
    #[allow(clippy::type_complexity)]
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
        let file_repository_factory = FileRepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = file_repository_factory.create(data_dir);
        let clock = Arc::new(SystemClock);

        (user_output, repository, clock)
    }

    #[tokio::test]
    async fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test with invalid environment name (contains underscore)
        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name", OutputFormat::Text)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ReleaseSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("", OutputFormat::Text)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ReleaseSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Valid environment name but environment doesn't exist
        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("test-env", OutputFormat::Text)
            .await;

        // Should fail because environment doesn't exist
        assert!(result.is_err());
        match result.unwrap_err() {
            ReleaseSubcommandError::ApplicationLayerError { .. } => (),
            other => panic!("Expected ApplicationLayerError, got: {other:?}"),
        }
    }
}
