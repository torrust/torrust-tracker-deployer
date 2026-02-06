//! Error types for the Purge Subcommand
//!
//! This module defines error types that can occur during CLI purge command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Purge command specific errors
///
/// This enum contains all error variants specific to the purge command,
/// including environment validation, repository access, and purge failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum PurgeSubcommandError {
    // ===== Environment Validation Errors =====
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

    // ===== User Interaction Errors =====
    /// User cancelled the purge operation
    ///
    /// The user declined the confirmation prompt.
    #[error("Purge cancelled by user")]
    UserCancelled,

    // ===== Repository Access Errors =====
    /// Repository operation failed
    ///
    /// Failed to create or access the environment repository.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to access environment repository at '{data_dir}': {reason}
Tip: Check directory permissions and disk space"
    )]
    RepositoryAccessFailed { data_dir: String, reason: String },

    // ===== Purge Operation Errors =====
    /// Purge operation failed
    ///
    /// The purge process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to purge environment '{name}': {reason}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    PurgeOperationFailed { name: String, reason: String },

    // ===== Internal Errors =====
    /// Progress reporting failed
    ///
    /// Failed to report progress to the user due to an internal error.
    /// This indicates a critical internal error.
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for PurgeSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl PurgeSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::controllers::purge::handler::PurgeCommandController;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::clock::SystemClock;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let data_dir = PathBuf::from("./data");
    /// let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    /// let repository = repository_factory.create(data_dir);
    /// let clock = Arc::new(SystemClock);
    /// if let Err(e) = PurgeCommandController::new(repository, clock, output).execute("test-env", false).await {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)] // Help text is comprehensive for user guidance
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                r"Environment name validation failed.

Valid environment names must:
- Be 1-63 characters long
- Start with a letter or digit
- Contain only letters, digits, and hyphens
- Not end with a hyphen

Examples of valid names:
- my-environment
- prod-server-1
- test123

Examples of invalid names:
- -my-env (starts with hyphen)
- my_env (contains underscore)
- my-environment-with-a-very-long-name-that-exceeds-the-maximum-length (too long)

Troubleshooting steps:
1. Check the environment name format
2. Rename the environment directory if needed
3. Try using a shorter, simpler name"
            }
            Self::EnvironmentNotAccessible { .. } => {
                r"Environment not found in data directory.

Possible causes:
1. Environment doesn't exist (never created)
2. Wrong environment name
3. Data directory permissions issue
4. Environment data was manually deleted

Troubleshooting steps:
1. List existing environments:
   torrust-tracker-deployer list

2. Check data directory:
   ls -la ./data/

3. Verify environment directory exists:
   ls -la ./data/<environment-name>/

4. Check file permissions:
   ls -la ./data/<environment-name>/environment.json

5. If environment was destroyed independently:
   - The environment data still exists locally
   - Use 'purge --force' if you're sure you want to remove it"
            }
            Self::UserCancelled => {
                r"Purge operation cancelled at user request.

No changes were made to the environment data.

To proceed with purge:
1. Run the command again and confirm when prompted
2. Or use --force flag to skip confirmation:
   torrust-tracker-deployer purge <environment-name> --force

Warning: Purge is irreversible - all local environment data will be permanently deleted."
            }
            Self::RepositoryAccessFailed { .. } => {
                r"Failed to access environment repository.

Possible causes:
1. Data directory doesn't exist
2. Insufficient permissions
3. Disk full or read-only filesystem
4. File lock conflict

Troubleshooting steps:
1. Check if data directory exists:
   ls -la ./data/

2. Verify permissions:
   stat ./data/

3. Check disk space:
   df -h ./data/

4. Check for file locks:
   lsof ./data/

5. Try running with elevated permissions (if appropriate):
   sudo torrust-tracker-deployer purge <environment-name>"
            }
            Self::PurgeOperationFailed { .. } => {
                r"Purge operation failed during execution.

This could be due to:
1. File system errors (permissions, disk full)
2. Locked files or directories
3. Corrupted environment data

Troubleshooting steps:
1. Check logs for detailed error information:
   torrust-tracker-deployer purge <environment-name> --log-output file-and-stderr

2. Verify no processes are using the environment data:
   lsof +D ./data/<environment-name>/
   lsof +D ./build/<environment-name>/

3. Check disk space:
   df -h

4. Try manual cleanup (last resort):
   rm -rf ./data/<environment-name>/
   rm -rf ./build/<environment-name>/

5. If environment is in an invalid state:
   - Check environment.json for corruption
   - Restore from backup if available"
            }
            Self::ProgressReportingFailed { .. } => {
                r"Progress reporting system encountered a critical error.

This is an internal bug that should be reported.

Immediate steps:
1. Capture full logs:
   torrust-tracker-deployer purge <environment-name> --log-output file-and-stderr

2. Report the issue at:
   https://github.com/torrust/torrust-tracker-deployer/issues

3. Include in report:
   - Full error message
   - Log files
   - Environment: OS, Rust version
   - Steps to reproduce

Workaround:
Try using --force flag and check if operation completes despite the error:
   torrust-tracker-deployer purge <environment-name> --force"
            }
        }
    }
}
