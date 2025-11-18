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
use crate::domain::environment::state::Created;
use crate::domain::Environment;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::config_loader::ConfigLoader;
use super::errors::CreateEnvironmentCommandError;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of main steps in the environment creation workflow
const ENVIRONMENT_CREATION_WORKFLOW_STEPS: usize = 3;

// ============================================================================
// HIGH-LEVEL API (EXECUTION CONTEXT PATTERN)
// ============================================================================

/// Handle environment creation command using `ExecutionContext` pattern
///
/// This function provides a clean interface for creating deployment environments,
/// integrating with the `ExecutionContext` pattern for dependency injection.
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file (JSON format)
/// * `working_dir` - Working directory path for environment storage
/// * `context` - Execution context providing access to services
///
/// # Returns
///
/// * `Ok(Environment<Created>)` - Environment created successfully
/// * `Err(CreateEnvironmentCommandError)` - Environment creation failed
///
/// # Errors
///
/// Returns `CreateEnvironmentCommandError` when:
/// * Configuration file cannot be loaded or is malformed
/// * Environment name is invalid or already exists
/// * Working directory is not accessible or doesn't exist
/// * Infrastructure provisioning fails (OpenTofu/LXD errors)
/// * File system operations fail (permission errors, disk space)
/// * User output system fails (mutex poisoning)
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::environment;
/// use torrust_tracker_deployer_lib::presentation::dispatch::context::ExecutionContext;
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let container = Arc::new(Container::new(VerbosityLevel::Normal));
/// let context = ExecutionContext::new(container);
/// let env_file = Path::new("./environment.json");
/// let working_dir = Path::new("./");
///
/// environment::handle(env_file, working_dir, &context)?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(
    env_file: &Path,
    working_dir: &Path,
    context: &crate::presentation::dispatch::context::ExecutionContext,
) -> Result<Environment<Created>, CreateEnvironmentCommandError> {
    handle_environment_creation_command(
        env_file,
        working_dir,
        &context.repository_factory(),
        &context.clock(),
        &context.user_output(),
    )
}

// ============================================================================
// INTERMEDIATE API (DIRECT DEPENDENCY INJECTION)
// ============================================================================

/// Handle the environment creation command
///
/// This is a thin wrapper over `CreateEnvironmentCommandController` that serves as
/// the public entry point for the environment creation command.
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file  
/// * `working_dir` - Working directory path for environment storage
/// * `repository_factory` - Factory for creating environment repositories
/// * `clock` - System clock service for timestamps
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Errors
///
/// Returns an error if:
/// - Configuration loading or validation fails
/// - Environment creation fails
/// - Progress reporting encounters a poisoned mutex
///
/// All errors include detailed context and actionable troubleshooting guidance.
///
/// # Returns
///
/// Returns `Ok(Environment<Created>)` on success, or a `CreateEnvironmentCommandError` on failure.
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
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::environment;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let container = Container::new(VerbosityLevel::Normal);
/// let context = ExecutionContext::new(Arc::new(container));
///
/// if let Err(e) = environment::handle(
///     Path::new("config.json"),
///     Path::new("./"),
///     &context
/// ) {
///     eprintln!("Environment creation failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
///
/// Direct usage (for testing or specialized scenarios):
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::environment::handler::handle;
/// use torrust_tracker_deployer_lib::presentation::dispatch::context::ExecutionContext;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let container = Arc::new(Container::new(VerbosityLevel::Normal));
/// let context = ExecutionContext::new(container);
///
/// if let Err(e) = handle(
///     Path::new("config.json"),
///     Path::new("./"),
///     &context
/// ) {
///     eprintln!("Environment creation failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_environment_creation_command(
    env_file: &Path,
    working_dir: &Path,
    repository_factory: &Arc<RepositoryFactory>,
    clock: &Arc<dyn Clock>,
    user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<Environment<Created>, CreateEnvironmentCommandError> {
    CreateEnvironmentCommandController::new(repository_factory.clone(), clock.clone(), user_output)
        .execute(env_file, working_dir)
}

// ============================================================================
// PRESENTATION LAYER CONTROLLER (IMPLEMENTATION DETAILS)
// ============================================================================

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
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl CreateEnvironmentCommandController {
    /// Create a new environment creation command controller
    ///
    /// Creates a `CreateEnvironmentCommandController` with dependency injection.
    /// This follows the single container architecture pattern.
    pub fn new(
        repository_factory: Arc<RepositoryFactory>,
        clock: Arc<dyn Clock>,
        user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress =
            ProgressReporter::new(user_output.clone(), ENVIRONMENT_CREATION_WORKFLOW_STEPS);

        Self {
            repository_factory,
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
    pub fn execute(
        &mut self,
        env_file: &Path,
        working_dir: &Path,
    ) -> Result<Environment<Created>, CreateEnvironmentCommandError> {
        let config = self.load_configuration(env_file)?;

        let command_handler = self.create_command_handler(working_dir)?;

        let environment = self.execute_create_command(&command_handler, config, working_dir)?;

        self.display_creation_results(&environment)?;

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
        self.progress.start_step("Loading configuration")?;

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
    /// - Creating repository using factory
    /// - Setting up command handler with dependencies
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Working directory path for environment storage
    ///
    /// # Returns
    ///
    /// Returns the initialized `CreateCommandHandler`.
    fn create_command_handler(
        &mut self,
        working_dir: &Path,
    ) -> Result<CreateCommandHandler, CreateEnvironmentCommandError> {
        self.progress.start_step("Creating command handler")?;

        // Repository expects the BASE data directory, not the working directory
        // It will append the environment name to create environment-specific paths
        let data_dir = working_dir.join("data");
        let repository = self.repository_factory.create(data_dir);

        let command_handler = CreateCommandHandler::new(repository, self.clock.clone());

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
        self.progress.start_step("Creating environment")?;

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
    /// - Instance details (name, data directory, build directory)
    ///
    /// # Arguments
    ///
    /// * `environment` - The successfully created environment
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
    ) -> Result<(), CreateEnvironmentCommandError> {
        self.progress.complete(&format!(
            "Environment '{}' created successfully",
            environment.name().as_str()
        ))?;

        self.progress.blank_line()?;

        self.progress.steps(
            "Environment Details:",
            &[
                &format!("Environment name: {}", environment.name().as_str()),
                &format!("Instance name: {}", environment.instance_name().as_str()),
                &format!("Data directory: {}", environment.data_dir().display()),
                &format!("Build directory: {}", environment.build_dir().display()),
            ],
        )?;

        Ok(())
    }
}
