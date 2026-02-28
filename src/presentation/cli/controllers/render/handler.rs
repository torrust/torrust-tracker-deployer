//! Render Command Controller
//!
//! This module handles the render command execution at the presentation layer,
//! including input validation, mode selection, and user feedback.

use std::cell::RefCell;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::render::{
    RenderCommandHandler, RenderInputMode, RenderResult,
};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::EnvironmentName;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::render::{JsonView, RenderDetailsData, TextView};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;

use super::errors::RenderCommandError;

/// Steps in the render workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderStep {
    ValidateInput,
    LoadConfiguration,
    GenerateArtifacts,
}

impl RenderStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateInput,
        Self::LoadConfiguration,
        Self::GenerateArtifacts,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateInput => "Validating input parameters",
            Self::LoadConfiguration => "Loading configuration",
            Self::GenerateArtifacts => "Generating deployment artifacts",
        }
    }
}

/// Presentation layer controller for render command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// for generating deployment artifacts without executing deployment.
///
/// # Responsibilities
///
/// - Validate input mode (env-name OR env-file)
/// - Parse and validate IP address
/// - Show progress updates to the user
/// - Format output for display
/// - Delegate artifact generation to application layer handler
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. Business logic is delegated to the application layer's
/// `RenderCommandHandler`.
pub struct RenderCommandController {
    handler: RenderCommandHandler,
    progress: ProgressReporter,
}

impl RenderCommandController {
    /// Create a new render command controller
    ///
    /// Creates a `RenderCommandController` with repository and user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        Self {
            handler: RenderCommandHandler::new(repository),
            progress: ProgressReporter::new(user_output, RenderStep::count()),
        }
    }

    /// Execute the render command workflow
    ///
    /// This performs input validation and delegates to the application handler.
    ///
    /// # Arguments
    ///
    /// * `env_name` - Optional environment name (mutually exclusive with `env_file`)
    /// * `env_file` - Optional config file path (mutually exclusive with `env_name`)
    /// * `ip` - Target instance IP address (required)
    /// * `output_dir` - Output directory for generated artifacts (required)
    /// * `force` - Whether to overwrite existing output directory
    /// * `working_dir` - Working directory for environment data (from --working-dir global arg)
    /// * `output_format` - Output format (text or JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Artifact generation succeeded
    /// * `Err(RenderCommandError)` - Validation or generation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither `env_name` nor `env_file` is provided
    /// - IP address is invalid
    /// - Output directory exists and force is false
    /// - Config file doesn't exist
    /// - Environment not found
    /// - Template rendering fails
    #[allow(clippy::too_many_arguments)] // Required parameters for render workflow - all are necessary
    pub async fn execute(
        &mut self,
        env_name: Option<&str>,
        env_file: Option<&Path>,
        ip: &str,
        output_dir: &Path,
        force: bool,
        working_dir: &Path,
        output_format: OutputFormat,
    ) -> Result<(), RenderCommandError> {
        // Step 1: Validate input
        self.progress
            .start_step(RenderStep::ValidateInput.description())?;

        // Validate IP address first (fail fast)
        let _target_ip: Ipv4Addr = ip
            .parse()
            .map_err(|_| RenderCommandError::InvalidIpAddress { ip: ip.to_string() })?;

        // Determine input mode and prepare handler parameters
        let input_mode = match (env_name, env_file) {
            (Some(name), None) => {
                let env_name = EnvironmentName::new(name).map_err(|e| {
                    RenderCommandError::InvalidEnvironmentName {
                        value: name.to_string(),
                        reason: e.to_string(),
                    }
                })?;
                RenderInputMode::EnvironmentName(env_name)
            }
            (None, Some(path)) => {
                // Validate file exists
                if !path.exists() {
                    return Err(RenderCommandError::ConfigFileNotFound {
                        path: path.to_path_buf(),
                    });
                }
                RenderInputMode::ConfigFile(path.to_path_buf())
            }
            (None, None) => return Err(RenderCommandError::NoInputMode),
            (Some(_), Some(_)) => unreachable!("Clap ensures mutual exclusivity"),
        };

        self.progress.complete_step(None)?;

        // Step 2: Load configuration and validate
        self.progress
            .start_step(RenderStep::LoadConfiguration.description())?;

        // working_dir is now passed as a parameter from the router
        // which gets it from context.working_dir() (the --working-dir global argument)

        self.progress.complete_step(None)?;

        // Step 3: Generate artifacts
        self.progress
            .start_step(RenderStep::GenerateArtifacts.description())?;

        // Call application handler
        let result = self
            .handler
            .execute(input_mode, ip, output_dir, force, working_dir)
            .await
            .map_err(RenderCommandError::from)?;

        self.progress.complete_step(None)?;

        // Render and display results
        self.complete_workflow(&result, output_format)?;

        Ok(())
    }

    /// Complete the workflow with render details output
    ///
    /// Renders the artifact generation summary using the chosen output format
    /// (text or JSON) and displays it to the user.
    fn complete_workflow(
        &mut self,
        result: &RenderResult,
        output_format: OutputFormat,
    ) -> Result<(), RenderCommandError> {
        let data = RenderDetailsData::from_result(result);

        match output_format {
            OutputFormat::Text => {
                self.progress.blank_line()?;
                self.progress.complete(&TextView::render(&data))?;
            }
            OutputFormat::Json => {
                self.progress.result(&JsonView::render(&data)?)?;
            }
        }

        Ok(())
    }
}
