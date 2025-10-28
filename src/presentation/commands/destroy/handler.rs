//! Destroy Command Handler
//!
//! This module handles the destroy command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::time::Duration;

use crate::application::command_handlers::DestroyCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

use super::errors::DestroySubcommandError;

/// Handle the destroy command
///
/// This function orchestrates the environment destruction workflow by:
/// 1. Validating the environment name
/// 2. Loading the environment from persistent storage
/// 3. Executing the destroy command handler
/// 4. Providing user-friendly progress updates
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to destroy
/// * `working_dir` - Root directory for environment data storage
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `DestroySubcommandError` if:
/// - Environment name is invalid
/// - Environment cannot be loaded
/// - Destruction fails
///
/// # Errors
///
/// This function will return an error if the environment name is invalid,
/// the environment cannot be loaded, or the destruction process fails.
/// All errors include detailed context and actionable troubleshooting guidance.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::commands::destroy;
/// use std::path::Path;
///
/// if let Err(e) = destroy::handle_destroy_command("test-env", Path::new(".")) {
///     eprintln!("Destroy failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_destroy_command(
    environment_name: &str,
    working_dir: &std::path::Path,
) -> Result<(), DestroySubcommandError> {
    // Create user output with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!("Destroying environment '{environment_name}'..."));

    // Validate environment name
    let env_name = EnvironmentName::new(environment_name.to_string()).map_err(|source| {
        let error = DestroySubcommandError::InvalidEnvironmentName {
            name: environment_name.to_string(),
            source,
        };
        output.error(&error.to_string());
        error
    })?;

    // Create repository for loading environment state
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(working_dir.to_path_buf());

    // Create clock for timing information
    let clock = std::sync::Arc::new(crate::shared::SystemClock);

    // Create and execute destroy command handler
    output.progress("Tearing down infrastructure...");

    let command_handler = DestroyCommandHandler::new(repository, clock);

    // Execute destroy - the handler will load the environment and handle all states internally
    let _destroyed_env = command_handler.execute(&env_name).map_err(|source| {
        let error = DestroySubcommandError::DestroyOperationFailed {
            name: environment_name.to_string(),
            source,
        };
        output.error(&error.to_string());
        error
    })?;

    output.progress("Cleaning up resources...");
    output.success(&format!(
        "Environment '{environment_name}' destroyed successfully"
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        // Test with invalid environment name (contains underscore)
        let result = handle_destroy_command("invalid_name", working_dir);

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        let result = handle_destroy_command("", working_dir);

        assert!(result.is_err());
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        // Try to destroy an environment that doesn't exist
        let result = handle_destroy_command("nonexistent-env", working_dir);

        assert!(result.is_err());
        // Should get DestroyOperationFailed because environment doesn't exist
        match result.unwrap_err() {
            DestroySubcommandError::DestroyOperationFailed { name, .. } => {
                assert_eq!(name, "nonexistent-env");
            }
            other => panic!("Expected DestroyOperationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_accept_valid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at destroy operation since we don't have a real environment setup
        let result = handle_destroy_command("test-env", working_dir);

        // Should fail at operation, not at name validation
        if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
