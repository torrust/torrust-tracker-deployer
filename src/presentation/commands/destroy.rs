//! Destroy Command Handler
//!
//! This module handles the destroy command execution, including environment validation,
//! repository access, and infrastructure destruction. It provides user-friendly
//! progress updates and comprehensive error handling.

use std::time::Duration;

use thiserror::Error;

use crate::application::command_handlers::{
    destroy::DestroyCommandHandlerError, DestroyCommandHandler,
};
use crate::domain::environment::name::{EnvironmentName, EnvironmentNameError};
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

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
/// Returns `Ok(())` on success, or a `DestroyError` if:
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
/// if let Err(e) = destroy::handle("test-env", Path::new(".")) {
///     eprintln!("Destroy failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(environment_name: &str, working_dir: &std::path::Path) -> Result<(), DestroyError> {
    // Create user output with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!("Destroying environment '{environment_name}'..."));

    // Validate environment name
    let env_name = EnvironmentName::new(environment_name.to_string()).map_err(|source| {
        let error = DestroyError::InvalidEnvironmentName {
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
        let error = DestroyError::DestroyOperationFailed {
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

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Destroy command specific errors
///
/// This enum contains all error variants specific to the destroy command,
/// including environment validation, repository access, and destruction failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum DestroyError {
    /// Environment name validation failed
    ///
    /// The provided environment name doesn't meet the validation requirements.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Invalid environment name '{name}': {source}
Tip: Environment names must be 1-63 characters, start with letter/digit, contain only letters/digits/hyphens")]
    InvalidEnvironmentName {
        name: String,
        #[source]
        source: EnvironmentNameError,
    },

    /// Environment not found or inaccessible
    ///
    /// The environment couldn't be loaded from persistent storage.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' not found or inaccessible from data directory '{data_dir}'
Tip: Check if environment exists: ls -la {data_dir}/"
    )]
    EnvironmentNotAccessible { name: String, data_dir: String },

    /// Destroy operation failed
    ///
    /// The destruction process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to destroy environment '{name}': {source}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    DestroyOperationFailed {
        name: String,
        #[source]
        source: DestroyCommandHandlerError,
    },

    /// Repository operation failed
    ///
    /// Failed to create or access the environment repository.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to access environment repository at '{data_dir}': {reason}
Tip: Check directory permissions and disk space"
    )]
    RepositoryAccessFailed { data_dir: String, reason: String },
}

impl DestroyError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::commands::destroy;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// if let Err(e) = destroy::handle("test-env", Path::new(".")) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Check environment name format:
   - Length: Must be 1-63 characters
   - Start: Must begin with letter (a-z, A-Z) or digit (0-9)
   - Characters: Only letters, digits, and hyphens allowed
   - End: Must not end with a hyphen

2. Common valid examples:
   - 'production'
   - 'test-env'
   - 'e2e-provision'
   - 'dev123'

3. Common invalid examples:
   - 'test_env' (underscores not allowed)
   - '-test' (starts with hyphen)
   - 'test-' (ends with hyphen)
   - '' (empty string)

For more information, see the environment naming conventions in the documentation."
            }

            Self::EnvironmentNotAccessible { .. } => {
                "Environment Not Accessible - Detailed Troubleshooting:

1. Check if environment exists:
   - List environments: ls -la data/
   - Look for directory with your environment name

2. Verify file permissions:
   - Check directory permissions: ls -ld data/
   - Ensure read/write access: chmod 755 data/

3. Check if environment was provisioned:
   - Look for environment.json file: ls -la data/{env_name}/
   - Verify it's a valid deployment environment

4. Common causes:
   - Environment was never created (run provision first)
   - Wrong data directory path
   - Permission issues
   - Corrupted environment state

If the environment should exist, check the logs for more details."
            }

            Self::DestroyOperationFailed { .. } => {
                "Destroy Operation Failed - Detailed Troubleshooting:

1. Check system resources:
   - Ensure sufficient disk space
   - Check network connectivity
   - Verify system permissions

2. Review the operation logs:
   - Run with verbose logging: --log-output file-and-stderr
   - Check log files in data/logs/
   - Look for specific error details

3. Check infrastructure state:
   - Verify LXD/OpenTofu are accessible
   - Check if VMs/containers are running
   - Ensure cleanup tools are available

4. Manual intervention may be needed:
   - Some resources might need manual cleanup
   - Check provider-specific tools (lxc list, tofu state list)
   - Remove stale infrastructure manually if needed

5. Recovery options:
   - Retry the destroy operation
   - Force cleanup with provider tools
   - Contact administrator if permissions are needed

For persistent issues, check the infrastructure documentation."
            }

            Self::RepositoryAccessFailed { .. } => {
                "Repository Access Failed - Detailed Troubleshooting:

1. Check directory permissions:
   - Verify data directory exists and is accessible
   - Ensure write permissions: chmod 755 data/
   - Check parent directory permissions

2. Verify disk space:
   - Check available space: df -h .
   - Ensure sufficient space for operations
   - Clean up if disk is full

3. Check file system issues:
   - Test file creation: touch data/test.tmp && rm data/test.tmp
   - Look for file system errors in system logs
   - Check if directory is on a read-only mount

4. Common solutions:
   - Create data directory: mkdir -p data
   - Fix permissions: sudo chown -R $USER:$USER data/
   - Move to directory with sufficient space

If the problem persists, check system logs and contact administrator."
            }
        }
    }
}
