//! Environment Creation Handler
//!
//! This module contains the main handler function for environment creation
//! and its supporting helper functions.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::application::command_handlers::CreateCommandHandler;
use crate::domain::Environment;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::progress::ProgressReporter;
use crate::presentation::user_output::UserOutput;
use crate::shared::clock::SystemClock;

use super::config_loader::ConfigLoader;
use super::errors::CreateEnvironmentCommandError;

/// Handle environment creation from configuration file
///
/// This function orchestrates the environment creation workflow with progress reporting:
///
/// 1. Load configuration from file
/// 2. Initialize dependencies
/// 3. Validate environment
/// 4. Execute create command
/// 5. Display creation results
///
/// Each step is tracked and timed using `ProgressReporter` for clear user feedback.
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file (JSON format)
/// * `working_dir` - Root directory for environment data storage
/// * `context` - Execution context providing access to application services
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateEnvironmentCommandError` if any step fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Configuration file cannot be loaded or validated
/// - Command execution fails
///
/// All errors include detailed context and actionable troubleshooting guidance.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateEnvironmentCommandError> {
    // Create progress reporter for 3 main steps
    let mut progress = ProgressReporter::new(context.user_output().clone(), 3);

    // Step 1: Load configuration
    progress.start_step("Loading configuration")?;
    let config = load_configuration(&mut progress, env_file)?;
    progress.complete_step(Some(&format!(
        "Configuration loaded: {}",
        config.environment.name
    )))?;

    // Step 2: Initialize dependencies
    progress.start_step("Initializing dependencies")?;

    // Create repository and clock services
    // TODO: Once Container is expanded, get these from context.container()
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(working_dir.to_path_buf());
    let clock = Arc::new(SystemClock);

    let command_handler = CreateCommandHandler::new(repository, clock);
    progress.complete_step(None)?;

    // Step 3: Execute create command (provision infrastructure)
    progress.start_step("Creating environment")?;
    let environment = execute_create_command(&mut progress, &command_handler, config)?;
    progress.complete_step(Some(&format!(
        "Instance created: {}",
        environment.instance_name().as_str()
    )))?;

    // Complete with summary
    progress.complete(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ))?;

    // Display final results
    display_creation_results(&context.user_output(), &environment)?;

    Ok(())
}

/// Load and validate configuration from file
///
/// This step handles:
/// - Loading configuration file using Figment
/// - Parsing JSON content
/// - Validating configuration using domain rules
///
/// # Arguments
///
/// * `progress` - Progress reporter for user feedback
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
pub(crate) fn load_configuration(
    progress: &mut ProgressReporter,
    env_file: &Path,
) -> Result<EnvironmentCreationConfig, CreateEnvironmentCommandError> {
    let user_output = progress.output();

    user_output
        .lock()
        .map_err(|_| CreateEnvironmentCommandError::UserOutputLockFailed)?
        .progress(&format!(
            "Loading configuration from '{}'...",
            env_file.display()
        ));

    let loader = ConfigLoader;

    loader
        .load_from_file(env_file)
        .inspect_err(|err: &CreateEnvironmentCommandError| {
            // Attempt to log error, but don't fail if mutex is poisoned
            if let Ok(mut output) = user_output.lock() {
                output.error(&err.to_string());
            }
        })
}

/// Execute the create command with the given configuration
///
/// This step handles:
/// - Executing the create command with the given handler
/// - Handling command execution errors
///
/// # Arguments
///
/// * `progress` - Progress reporter for user feedback
/// * `command_handler` - Pre-created command handler
/// * `config` - Validated environment creation configuration
///
/// # Returns
///
/// Returns the created `Environment` on success.
///
/// # Errors
///
/// Returns an error if command execution fails (e.g., environment already exists).
pub(crate) fn execute_create_command(
    progress: &mut ProgressReporter,
    command_handler: &CreateCommandHandler,
    config: EnvironmentCreationConfig,
) -> Result<Environment, CreateEnvironmentCommandError> {
    let user_output = progress.output();

    user_output
        .lock()
        .map_err(|_| CreateEnvironmentCommandError::UserOutputLockFailed)?
        .progress(&format!(
            "Creating environment '{}'...",
            config.environment.name
        ));

    user_output
        .lock()
        .map_err(|_| CreateEnvironmentCommandError::UserOutputLockFailed)?
        .progress("Validating configuration and creating environment...");

    #[allow(clippy::manual_inspect)]
    command_handler.execute(config).map_err(|source| {
        let error = CreateEnvironmentCommandError::CommandFailed { source };
        // Attempt to log error, but don't fail if mutex is poisoned
        if let Ok(mut output) = user_output.lock() {
            output.error(&error.to_string());
        }
        error
    })
}

/// Display the results of successful environment creation
///
/// This step outputs:
/// - Success message with environment name
/// - Instance name
/// - Data directory location
/// - Build directory location
///
/// # Arguments
///
/// * `user_output` - Shared user output for displaying messages
/// * `environment` - The successfully created environment
///
/// # Returns
///
/// Returns `Ok(())` on success, or `CreateEnvironmentCommandError::UserOutputLockFailed`
/// if the `UserOutput` mutex is poisoned.
///
/// # Errors
///
/// This function will return an error if the `UserOutput` mutex is poisoned,
/// which indicates a panic occurred in another thread while holding the output lock.
pub(crate) fn display_creation_results(
    user_output: &Arc<Mutex<UserOutput>>,
    environment: &Environment,
) -> Result<(), CreateEnvironmentCommandError> {
    let mut output = user_output
        .lock()
        .map_err(|_| CreateEnvironmentCommandError::UserOutputLockFailed)?;

    output.success(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ));

    output.result(&format!(
        "Instance name: {}",
        environment.instance_name().as_str()
    ));

    output.result(&format!(
        "Data directory: {}",
        environment.data_dir().display()
    ));

    output.result(&format!(
        "Build directory: {}",
        environment.build_dir().display()
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::TempDir;

    use crate::application::command_handlers::CreateCommandHandler;
    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
    use crate::presentation::controllers::create::subcommands::environment::config_loader::ConfigLoader;
    use crate::presentation::user_output::{UserOutput, VerbosityLevel};
    use crate::shared::clock::SystemClock;

    mod display_creation_results_tests {
        use super::*;

        #[test]
        fn it_should_display_environment_details() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.json");

            let project_root = env!("CARGO_MANIFEST_DIR");
            let private_key_path = format!("{project_root}/fixtures/testing_rsa");
            let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

            let config_json = format!(
                r#"{{
                "environment": {{
                    "name": "test-display"
                }},
                "ssh_credentials": {{
                    "private_key_path": "{private_key_path}",
                    "public_key_path": "{public_key_path}"
                }}
            }}"#
            );
            std::fs::write(&config_path, config_json).unwrap();

            // Create environment
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            // Create command handler using manual dependency creation
            let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
            let repository = repository_factory.create(temp_dir.path().to_path_buf());
            let clock = Arc::new(SystemClock);
            let command_handler = CreateCommandHandler::new(repository, clock);

            let environment = command_handler.execute(config).unwrap();

            // Test display function with custom output
            let stderr_buf = Vec::new();
            let stderr_writer = Box::new(Cursor::new(stderr_buf));
            let stdout_buf = Vec::new();
            let stdout_writer = Box::new(Cursor::new(stdout_buf));

            let output =
                UserOutput::with_writers(VerbosityLevel::Normal, stdout_writer, stderr_writer);
            let display_output = Arc::new(Mutex::new(output));

            // Test display function
            let result = display_creation_results(&display_output, &environment);
            assert!(result.is_ok(), "display_creation_results should succeed");

            // Note: We can't easily verify the exact output without refactoring UserOutput
            // to expose the buffers, but the important thing is it succeeds
        }
    }
}
