//! List Command Handler
//!
//! This module handles the list command execution at the presentation layer,
//! displaying a summary of all environments in the workspace.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::list::info::EnvironmentList;
use crate::application::command_handlers::list::{ListCommandHandler, ListCommandHandlerError};
use crate::application::traits::RepositoryProvider;
use crate::presentation::cli::input::cli::output_format::OutputFormat;
use crate::presentation::cli::views::commands::list::{JsonView, TextView};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;

use super::errors::ListSubcommandError;

/// Steps in the list workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListStep {
    ScanEnvironments,
    DisplayResults,
}

impl ListStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[Self::ScanEnvironments, Self::DisplayResults];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ScanEnvironments => "Scanning for environments",
            Self::DisplayResults => "Displaying results",
        }
    }
}

/// Presentation layer controller for list command workflow
///
/// Lists all environments in the workspace with summary information.
/// This is a read-only command that scans local storage without network calls.
///
/// ## Responsibilities
///
/// - Scan data directory for environments
/// - Delegate to application layer for data extraction
/// - Display environment list to the user
/// - Handle partial failures gracefully
///
/// ## Architecture
///
/// This controller implements the Presentation Layer pattern, handling
/// user interaction while delegating business logic to the application layer.
pub struct ListCommandController {
    handler: ListCommandHandler,
    progress: ProgressReporter,
}

impl ListCommandController {
    /// Create a new `ListCommandController` with dependencies
    ///
    /// # Arguments
    ///
    /// * `file_repository_factory` - Factory for creating environment repositories
    /// * `data_directory` - Path to the data directory
    /// * `user_output` - Shared output service for user feedback
    #[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
    pub fn new(
        file_repository_factory: Arc<dyn RepositoryProvider>,
        data_directory: Arc<Path>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let handler = ListCommandHandler::new(file_repository_factory, data_directory);
        let progress = ProgressReporter::new(user_output, ListStep::count());

        Self { handler, progress }
    }

    /// Execute the list command workflow
    ///
    /// This method orchestrates the two-step workflow:
    /// 1. Scan for environments via application layer
    /// 2. Display results to user
    ///
    /// # Arguments
    ///
    /// * `output_format` - Output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns `ListSubcommandError` if any step fails
    pub fn execute(&mut self, output_format: OutputFormat) -> Result<(), ListSubcommandError> {
        // Step 1: Scan for environments via application layer
        let env_list = self.scan_environments()?;

        // Step 2: Display results
        self.display_results(&env_list, output_format)?;

        Ok(())
    }

    /// Step 1: Scan for environments via application layer
    fn scan_environments(&mut self) -> Result<EnvironmentList, ListSubcommandError> {
        self.progress
            .start_step(ListStep::ScanEnvironments.description())?;

        let env_list = self.handler.execute().map_err(Self::map_handler_error)?;

        let count = env_list.total_count;
        self.progress
            .complete_step(Some(&format!("Found {count} environment(s)")))?;

        Ok(env_list)
    }

    /// Map application layer errors to presentation errors
    fn map_handler_error(error: ListCommandHandlerError) -> ListSubcommandError {
        match error {
            ListCommandHandlerError::DataDirectoryNotFound { path } => {
                ListSubcommandError::DataDirectoryNotFound { path }
            }
            ListCommandHandlerError::PermissionDenied { path } => {
                ListSubcommandError::PermissionDenied { path }
            }
            ListCommandHandlerError::ScanError { message } => {
                ListSubcommandError::ScanError { message }
            }
        }
    }

    /// Step 2: Display environment list
    ///
    /// Orchestrates a functional pipeline to display the environment list:
    /// `EnvironmentList` → `String` → stdout
    ///
    /// The output is written to stdout (not stderr) as it represents the final
    /// command result rather than progress information.
    ///
    /// # Arguments
    ///
    /// * `env_list` - Environment list to display
    /// * `output_format` - Output format (Text or Json)
    fn display_results(
        &mut self,
        env_list: &EnvironmentList,
        output_format: OutputFormat,
    ) -> Result<(), ListSubcommandError> {
        self.progress
            .start_step(ListStep::DisplayResults.description())?;

        // Pipeline: EnvironmentList → render → output to stdout
        // Use Strategy Pattern to select view based on output format
        let output = match output_format {
            OutputFormat::Text => TextView::render(env_list)?,
            OutputFormat::Json => JsonView::render(env_list)?,
        };

        self.progress.result(&output)?;

        self.progress.complete_step(Some("Results displayed"))?;

        Ok(())
    }
}
