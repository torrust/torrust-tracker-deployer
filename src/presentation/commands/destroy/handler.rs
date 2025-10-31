//! Destroy Command Handler
//!
//! This module handles the destroy command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::sync::{Arc, Mutex};

use crate::domain::environment::name::EnvironmentName;
use crate::presentation::commands::factory::CommandHandlerFactory;
use crate::presentation::progress::ProgressReporter;
use crate::presentation::user_output::UserOutput;

use super::errors::DestroySubcommandError;

/// Handle the destroy command
///
/// This function orchestrates the environment destruction workflow with progress reporting:
/// 1. Validating the environment name
/// 2. Tearing down infrastructure
/// 3. Cleaning up resources
///
/// Each step is tracked and timed using `ProgressReporter` for clear user feedback.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to destroy
/// * `working_dir` - Root directory for environment data storage
/// * `user_output` - Shared user output service for consistent output formatting
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
/// use std::path::Path;
/// use std::sync::{Arc, Mutex};
/// use torrust_tracker_deployer_lib::presentation::commands::destroy;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
/// if let Err(e) = destroy::handle_destroy_command("test-env", Path::new("."), &user_output) {
///     eprintln!("Destroy failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_destroy_command(
    environment_name: &str,
    working_dir: &std::path::Path,
    user_output: &Arc<Mutex<UserOutput>>,
) -> Result<(), DestroySubcommandError> {
    // Create factory and context with all shared dependencies
    let factory = CommandHandlerFactory::new();
    let ctx = factory.create_context(working_dir.to_path_buf(), user_output.clone());

    // Create progress reporter for 3 main steps
    let mut progress = ProgressReporter::new(user_output.clone(), 3);

    // Step 1: Validate environment name
    progress.start_step("Validating environment")?;
    let env_name = EnvironmentName::new(environment_name.to_string()).map_err(|source| {
        DestroySubcommandError::InvalidEnvironmentName {
            name: environment_name.to_string(),
            source,
        }
    })?;
    progress.complete_step(Some(&format!(
        "Environment name validated: {environment_name}"
    )))?;

    // Step 2: Initialize dependencies
    progress.start_step("Initializing dependencies")?;
    let command_handler = factory.create_destroy_handler(&ctx);
    progress.complete_step(None)?;

    // Step 3: Execute destroy command (tear down infrastructure)
    progress.start_step("Tearing down infrastructure")?;
    let _destroyed_env = command_handler.execute(&env_name).map_err(|source| {
        DestroySubcommandError::DestroyOperationFailed {
            name: environment_name.to_string(),
            source,
        }
    })?;
    progress.complete_step(Some("Infrastructure torn down"))?;

    // Complete with summary
    progress.complete(&format!(
        "Environment '{environment_name}' destroyed successfully"
    ))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::user_output::VerbosityLevel;
    use std::fs;
    use tempfile::TempDir;

    /// Test helper to create a test user output
    fn create_test_user_output() -> Arc<Mutex<UserOutput>> {
        Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)))
    }

    #[test]
    fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let user_output = create_test_user_output();

        // Test with invalid environment name (contains underscore)
        let result = handle_destroy_command("invalid_name", working_dir, &user_output);

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
        let user_output = create_test_user_output();

        let result = handle_destroy_command("", working_dir, &user_output);

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
        let user_output = create_test_user_output();

        // Try to destroy an environment that doesn't exist
        let result = handle_destroy_command("nonexistent-env", working_dir, &user_output);

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
        let user_output = create_test_user_output();

        // Create a mock environment directory to test validation
        let env_dir = working_dir.join("test-env");
        fs::create_dir_all(&env_dir).unwrap();

        // Valid environment name should pass validation, but will fail
        // at destroy operation since we don't have a real environment setup
        let result = handle_destroy_command("test-env", working_dir, &user_output);

        // Should fail at operation, not at name validation
        if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not fail at name validation for 'test-env'");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }
}
