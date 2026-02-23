//! Test Command Handler
//!
//! This module handles the test command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::test::result::TestResult;
use crate::application::command_handlers::TestCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::presentation::input::cli::OutputFormat;
use crate::presentation::views::commands::test::{JsonView, TestResultData, TextView};
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::TestSubcommandError;

/// Steps in the test workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestStep {
    ValidateEnvironment,
    CreateCommandHandler,
    TestInfrastructure,
}

impl TestStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::CreateCommandHandler,
        Self::TestInfrastructure,
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
            Self::TestInfrastructure => "Testing infrastructure",
        }
    }
}

/// Presentation layer controller for test command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// while delegating business logic to the application layer's `TestCommandHandler`.
///
/// ## Responsibilities
///
/// - Validate environment name format
/// - Create and invoke the application layer `TestCommandHandler`
/// - Report progress to the user (4 steps)
/// - Format success/error messages with actionable guidance
///
/// ## Architecture
///
/// This controller **only orchestrates the workflow** - all validation logic
/// is implemented in the `TestCommandHandler` at the application layer.
///
/// The `TestCommandHandler.execute()` method performs all infrastructure validation:
/// - Cloud-init completion check
/// - Docker installation verification
/// - Docker Compose installation verification
pub struct TestCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    progress: ProgressReporter,
}

impl TestCommandController {
    /// Create a new `TestCommandController` with dependencies
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Working directory containing the data folder
    /// * `repository` - Environment repository with Send + Sync bounds
    /// * `user_output` - Shared output service for user feedback
    #[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, TestStep::count());

        Self {
            repository,
            progress,
        }
    }

    /// Execute the complete test workflow
    ///
    /// This method orchestrates the four-step workflow:
    /// 1. Validate environment name
    /// 2. Create command handler
    /// 3. Execute validation workflow via application layer
    /// 4. Complete workflow and display success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment to test
    ///
    /// # Errors
    ///
    /// Returns `TestSubcommandError` if any step fails
    pub async fn execute(
        &mut self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<(), TestSubcommandError> {
        // 1. Validate environment name
        let env_name = self.validate_environment_name(environment_name)?;

        // 2. Create command handler
        let handler = self.create_command_handler()?;

        // 3. Execute validation workflow via application layer
        let result = self.fixture_infrastructure(&handler, &env_name).await?;

        // 4. Complete workflow with rendered output
        self.complete_workflow(environment_name, &result, output_format)?;

        Ok(())
    }

    /// Step 1: Validate environment name format
    ///
    /// # Errors
    ///
    /// Returns `TestSubcommandError::InvalidEnvironmentName` if validation fails
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, TestSubcommandError> {
        self.progress
            .start_step(TestStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            TestSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Step 2: Create the application layer command handler
    ///
    /// # Errors
    ///
    /// Returns `TestSubcommandError::ProgressReportingFailed` if progress reporting fails
    fn create_command_handler(&mut self) -> Result<TestCommandHandler, TestSubcommandError> {
        self.progress
            .start_step(TestStep::CreateCommandHandler.description())?;

        let handler = TestCommandHandler::new(self.repository.clone());
        self.progress.complete_step(None)?;

        Ok(handler)
    }

    /// Step 3: Execute infrastructure validation tests
    ///
    /// Delegates all validation logic to the application layer `TestCommandHandler`.
    /// The handler returns a structured `TestResult` containing DNS warnings
    /// which are rendered here in the presentation layer.
    ///
    /// # Errors
    ///
    /// Returns `TestSubcommandError::ValidationFailed` if any validation check fails
    async fn fixture_infrastructure(
        &mut self,
        handler: &TestCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<TestResult, TestSubcommandError> {
        self.progress
            .start_step(TestStep::TestInfrastructure.description())?;

        let result = handler.execute(env_name).await.map_err(|source| {
            TestSubcommandError::ValidationFailed {
                name: env_name.to_string(),
                source: Box::new(source),
            }
        })?;

        // Render advisory DNS warnings from the test result
        for warning in &result.dns_warnings {
            self.progress
                .output()
                .lock()
                .borrow_mut()
                .warn(&format!("DNS check: {warning}"));
        }

        let step_message = if result.has_dns_warnings() {
            "Infrastructure tests passed (with DNS warnings)"
        } else {
            "Infrastructure tests passed"
        };

        self.progress.complete_step(Some(step_message))?;

        Ok(result)
    }

    /// Step 4: Complete workflow and display success message
    ///
    /// # Errors
    ///
    /// Returns `TestSubcommandError::ProgressReportingFailed` if progress reporting fails
    fn complete_workflow(
        &mut self,
        environment_name: &str,
        result: &TestResult,
        output_format: OutputFormat,
    ) -> Result<(), TestSubcommandError> {
        let data = TestResultData::new(environment_name, result);

        let output = match output_format {
            OutputFormat::Text => TextView::render(&data),
            OutputFormat::Json => JsonView::render(&data),
        };

        self.progress.result(&output)?;

        Ok(())
    }
}
