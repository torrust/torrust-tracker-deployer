//! Environment Creation Command Handler
//!
//! This module handles the environment creation command execution at the presentation layer,
//! including configuration loading, validation, and user interaction.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::application::command_handlers::CreateCommandHandler;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Created;
use crate::domain::Environment;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::create::{
    EnvironmentDetailsData, JsonView, TextView,
};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;
use crate::shared::clock::Clock;

use super::config_loader::ConfigLoader;
use super::errors::CreateEnvironmentCommandError;

/// Steps in the environment creation workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreateEnvironmentStep {
    LoadConfiguration,
    CreateCommandHandler,
    CreateEnvironment,
}

impl CreateEnvironmentStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::LoadConfiguration,
        Self::CreateCommandHandler,
        Self::CreateEnvironment,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::LoadConfiguration => "Loading configuration",
            Self::CreateCommandHandler => "Creating command handler",
            Self::CreateEnvironment => "Creating environment",
        }
    }
}

/// Presentation layer controller for environment creation command workflow
///
/// Coordinates user interaction, progress reporting, and output formatting
/// before delegating to the application layer environment creation logic.
///
/// # Responsibilities
///
/// - Load and validate configuration from file
/// - Show progress updates to the user
/// - Initialize dependencies and command handler
/// - Execute environment creation through application layer
/// - Format success/error messages for display
/// - Display creation results with environment details
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual environment creation to the application layer's
/// `CreateCommandHandler`, maintaining clear separation of concerns.
pub struct CreateEnvironmentCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl CreateEnvironmentCommandController {
    /// Create a new environment creation command controller
    ///
    /// Creates a `CreateEnvironmentCommandController` with dependency injection.
    /// This follows the single container architecture pattern.
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output.clone(), CreateEnvironmentStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete environment creation workflow
    ///
    /// Orchestrates all steps of the environment creation command:
    /// 1. Load and validate configuration from file
    /// 2. Create application layer command handler  
    /// 3. Execute environment creation through application layer
    /// 4. Display creation results and environment details
    ///
    /// # Arguments
    ///
    /// * `env_file` - Path to the environment configuration file
    /// * `working_dir` - Working directory path for environment storage
    /// * `output_format` - Output format for results (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration loading or validation fails
    /// - Environment creation fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Created>)` on success, or a `CreateEnvironmentCommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(
        &mut self,
        env_file: &Path,
        working_dir: &Path,
        output_format: OutputFormat,
    ) -> Result<Environment<Created>, CreateEnvironmentCommandError> {
        let config = self.load_configuration(env_file)?;

        let command_handler = self.create_command_handler()?;

        let environment = self.execute_create_command(&command_handler, config, working_dir)?;

        self.display_creation_results(&environment, output_format)?;

        Ok(environment)
    }

    /// Load and validate configuration from file
    ///
    /// This step handles:
    /// - Loading configuration file using `ConfigLoader`
    /// - Parsing JSON content
    /// - Validating configuration using domain rules
    ///
    /// # Arguments
    ///
    /// * `env_file` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Returns the loaded and validated `EnvironmentCreationConfig`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file is not found
    /// - JSON parsing fails
    /// - Domain validation fails
    fn load_configuration(
        &mut self,
        env_file: &Path,
    ) -> Result<EnvironmentCreationConfig, CreateEnvironmentCommandError> {
        self.progress
            .start_step(CreateEnvironmentStep::LoadConfiguration.description())?;

        self.progress.sub_step(&format!(
            "Loading configuration from '{}'...",
            env_file.display()
        ))?;

        let loader = ConfigLoader;

        let config = loader.load_from_file(env_file).inspect_err(
            |err: &CreateEnvironmentCommandError| {
                // Log error details for debugging
                tracing::error!(
                    error = %err,
                    config_file = %env_file.display(),
                    "Configuration loading failed"
                );
            },
        )?;

        self.progress.complete_step(Some(&format!(
            "Configuration loaded: {}",
            config.environment.name
        )))?;

        Ok(config)
    }

    /// Create application layer command handler
    ///
    /// This step handles:
    /// - Setting up command handler with dependencies
    ///
    /// # Returns
    ///
    /// Returns the initialized `CreateCommandHandler`.
    fn create_command_handler(
        &mut self,
    ) -> Result<CreateCommandHandler, CreateEnvironmentCommandError> {
        self.progress
            .start_step(CreateEnvironmentStep::CreateCommandHandler.description())?;

        let command_handler =
            CreateCommandHandler::new(self.repository.clone(), self.clock.clone());

        self.progress.complete_step(None)?;

        Ok(command_handler)
    }

    /// Execute the create command with the given configuration
    ///
    /// This step handles:
    /// - Executing the create command with the given handler
    /// - Handling command execution errors
    ///
    /// # Arguments
    ///
    /// * `command_handler` - Pre-created command handler
    /// * `config` - Validated environment creation configuration
    /// * `working_dir` - Working directory path for environment storage
    ///
    /// # Returns
    ///
    /// Returns the created `Environment` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails (e.g., environment already exists).
    fn execute_create_command(
        &mut self,
        command_handler: &CreateCommandHandler,
        config: EnvironmentCreationConfig,
        working_dir: &Path,
    ) -> Result<Environment<Created>, CreateEnvironmentCommandError> {
        self.progress
            .start_step(CreateEnvironmentStep::CreateEnvironment.description())?;

        self.progress.sub_step(&format!(
            "Creating environment '{}'...",
            config.environment.name
        ))?;

        self.progress
            .sub_step("Validating configuration and creating environment...")?;

        let environment = command_handler
            .execute(config, working_dir)
            .map_err(|source| CreateEnvironmentCommandError::CommandFailed { source })?;

        self.progress.complete_step(Some(&format!(
            "Environment created: {}",
            environment.name().as_str()
        )))?;

        Ok(environment)
    }

    /// Display the results of successful environment creation
    ///
    /// This step outputs:
    /// - Final completion message with environment name
    /// - Environment details (name, instance name, data directory, build directory)
    ///
    /// The output formatting is delegated to the view layer (`TextView` or `JsonView`)
    /// following the MVC pattern and Strategy Pattern. This separates presentation
    /// concerns from controller logic and allows easy addition of new formats.
    ///
    /// # Arguments
    ///
    /// * `environment` - The successfully created environment
    /// * `output_format` - The format to use for rendering output (Text or Json)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `CreateEnvironmentCommandError` if progress reporting fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if progress reporting encounters issues,
    /// which indicates the environment was created but we couldn't display results.
    fn display_creation_results(
        &mut self,
        environment: &Environment<Created>,
        output_format: OutputFormat,
    ) -> Result<(), CreateEnvironmentCommandError> {
        self.progress.complete(&format!(
            "Environment '{}' created successfully",
            environment.name().as_str()
        ))?;

        self.progress.blank_line()?;

        // Convert domain model to presentation DTO
        let details = EnvironmentDetailsData::from(environment);

        // Render using appropriate view based on output format (Strategy Pattern)
        let output = match output_format {
            OutputFormat::Text => TextView::render(&details)?,
            OutputFormat::Json => JsonView::render(&details).map_err(|e| {
                CreateEnvironmentCommandError::OutputFormatting {
                    reason: format!("Failed to serialize environment details as JSON: {e}"),
                }
            })?,
        };

        // Output the rendered result
        self.progress.result(&output)?;

        Ok(())
    }
}
