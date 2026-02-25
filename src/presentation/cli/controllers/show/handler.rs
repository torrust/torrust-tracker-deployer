//! Show Command Handler
//!
//! This module handles the show command execution at the presentation layer,
//! displaying environment information with state-aware details.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::show::info::EnvironmentInfo;
use crate::application::command_handlers::show::{ShowCommandHandler, ShowCommandHandlerError};
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::show::{JsonView, TextView};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::UserOutput;

use super::errors::ShowSubcommandError;

/// Steps in the show workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowStep {
    ValidateEnvironment,
    LoadEnvironment,
    DisplayInformation,
}

impl ShowStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::LoadEnvironment,
        Self::DisplayInformation,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment name",
            Self::LoadEnvironment => "Loading environment",
            Self::DisplayInformation => "Displaying information",
        }
    }
}

/// Presentation layer controller for show command workflow
///
/// Displays environment information with state-aware details.
/// This is a read-only command that shows stored data without remote verification.
///
/// ## Responsibilities
///
/// - Validate environment name format
/// - Delegate to application layer for data extraction
/// - Display state-aware information to the user
/// - Provide next-step guidance based on current state
///
/// ## Architecture
///
/// This controller implements the Presentation Layer pattern, handling
/// user interaction while delegating business logic to the application layer.
pub struct ShowCommandController {
    handler: ShowCommandHandler,
    progress: ProgressReporter,
}

impl ShowCommandController {
    /// Create a new `ShowCommandController` with dependencies
    ///
    /// # Arguments
    ///
    /// * `repository` - Environment repository for loading environment data
    /// * `user_output` - Shared output service for user feedback
    #[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let handler = ShowCommandHandler::new(repository);
        let progress = ProgressReporter::new(user_output, ShowStep::count());

        Self { handler, progress }
    }

    /// Execute the show command workflow
    ///
    /// This method orchestrates the three-step workflow:
    /// 1. Validate environment name
    /// 2. Load environment and extract info via application layer
    /// 3. Display information to user
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment to show
    /// * `output_format` - Output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns `ShowSubcommandError` if any step fails
    pub fn execute(
        &mut self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<(), ShowSubcommandError> {
        // Step 1: Validate environment name
        let env_name = self.validate_environment_name(environment_name)?;

        // Step 2: Load environment via application layer
        let env_info = self.load_environment(&env_name)?;

        // Step 3: Display information
        self.display_information(&env_info, output_format)?;

        Ok(())
    }

    /// Step 1: Validate environment name format
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ShowSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Step 2: Load environment via application layer
    fn load_environment(
        &mut self,
        env_name: &EnvironmentName,
    ) -> Result<EnvironmentInfo, ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::LoadEnvironment.description())?;

        let env_info = self
            .handler
            .execute(env_name)
            .map_err(|e| Self::map_handler_error(e, env_name))?;

        self.progress
            .complete_step(Some(&format!("Environment loaded: {env_name}")))?;

        Ok(env_info)
    }

    /// Map application layer errors to presentation errors
    fn map_handler_error(
        error: ShowCommandHandlerError,
        env_name: &EnvironmentName,
    ) -> ShowSubcommandError {
        match error {
            ShowCommandHandlerError::EnvironmentNotFound { .. } => {
                ShowSubcommandError::EnvironmentNotFound {
                    name: env_name.to_string(),
                }
            }
            ShowCommandHandlerError::LoadError(e) => ShowSubcommandError::LoadError {
                name: env_name.to_string(),
                message: e.to_string(),
            },
        }
    }

    /// Step 3: Display environment information
    ///
    /// Orchestrates a functional pipeline to display environment information:
    /// `EnvironmentInfo` → `String` → stdout
    ///
    /// The output is written to stdout (not stderr) as it represents the final
    /// command result rather than progress information.
    ///
    /// # MVC Architecture
    ///
    /// Following the MVC pattern with functional composition:
    /// - Model: `EnvironmentInfo` (application layer DTO)
    /// - View: `TextView::render()` or `JsonView::render()` (formatting)
    /// - Controller (this method): Orchestrates the pipeline
    /// - Output: `ProgressReporter::result()` (routing to stdout)
    fn display_information(
        &mut self,
        env_info: &EnvironmentInfo,
        output_format: OutputFormat,
    ) -> Result<(), ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::DisplayInformation.description())?;

        // Render using appropriate view based on output format (Strategy Pattern)
        let output = match output_format {
            OutputFormat::Text => TextView::render(env_info),
            OutputFormat::Json => JsonView::render(env_info),
        };

        // Pipeline: EnvironmentInfo → render → output to stdout
        self.progress.result(&output)?;

        self.progress.complete_step(Some("Information displayed"))?;

        Ok(())
    }
}
