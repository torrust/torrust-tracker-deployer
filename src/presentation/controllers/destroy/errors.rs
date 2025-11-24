//! Error types for the Destroy Subcommand
//!
//! This module defines error types that can occur during CLI destroy command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Destroy command specific errors
///
/// This enum contains all error variants specific to the destroy command,
/// including environment validation, repository access, and destruction failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum DestroySubcommandError {
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

    // ===== Destroy Operation Errors =====
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

impl From<ProgressReporterError> for DestroySubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl DestroySubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
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
    /// use torrust_tracker_deployer_lib::presentation::controllers::destroy;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container));
    ///
    /// if let Err(e) = destroy::handle("test-env", &context).await {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    ///
    /// Direct usage (for testing):
    ///
    /// ```rust
    /// use std::path::{Path, PathBuf};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use parking_lot::ReentrantMutex;
    /// use std::cell::RefCell;
    /// use torrust_tracker_deployer_lib::presentation::controllers::destroy;
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
    /// if let Err(e) = destroy::handle_destroy_command("test-env", repository, clock, &output).await {
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

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This is a critical bug that indicates progress reporting to the user failed.
This should never happen in normal operation.

1. Immediate actions:
   - Save all relevant logs and error messages
   - Note what operation was being performed
   - Record the environment state

2. Report the issue:
   - This is a bug that needs to be reported
   - Include full logs: --log-output file-and-stderr
   - Provide steps to reproduce if possible
   - Include system information (OS, versions)

3. Workaround:
   - Restart the application
   - Try the operation again
   - Check for resource exhaustion (memory, threads)

This error indicates a serious bug in the application's progress reporting system.
Please report it to the development team with full details."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_invalid_environment_name() {
        let error = DestroySubcommandError::InvalidEnvironmentName {
            name: "invalid_name".to_string(),
            source: EnvironmentNameError::InvalidFormat {
                attempted_name: "invalid_name".to_string(),
                reason: "contains underscore".to_string(),
                valid_examples: vec!["dev".to_string(), "staging".to_string()],
            },
        };

        let help = error.help();
        assert!(help.contains("Invalid Environment Name"));
        assert!(help.contains("Check environment name format"));
    }

    #[test]
    fn it_should_provide_help_for_environment_not_accessible() {
        let error = DestroySubcommandError::EnvironmentNotAccessible {
            name: "test-env".to_string(),
            data_dir: "/tmp/data".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Accessible"));
        assert!(help.contains("Check if environment exists"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        use std::path::PathBuf;

        let errors: Vec<DestroySubcommandError> = vec![
            DestroySubcommandError::InvalidEnvironmentName {
                name: "test".to_string(),
                source: EnvironmentNameError::InvalidFormat {
                    attempted_name: "test".to_string(),
                    reason: "invalid".to_string(),
                    valid_examples: vec!["dev".to_string()],
                },
            },
            DestroySubcommandError::EnvironmentNotAccessible {
                name: "test".to_string(),
                data_dir: "/tmp".to_string(),
            },
            DestroySubcommandError::DestroyOperationFailed {
                name: "test".to_string(),
                source: DestroyCommandHandlerError::StateCleanupFailed {
                    path: PathBuf::from("/tmp"),
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
                },
            },
            DestroySubcommandError::RepositoryAccessFailed {
                data_dir: "/tmp".to_string(),
                reason: "permission denied".to_string(),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn it_should_display_error_with_context() {
        let error = DestroySubcommandError::InvalidEnvironmentName {
            name: "invalid_env".to_string(),
            source: EnvironmentNameError::InvalidFormat {
                attempted_name: "invalid_env".to_string(),
                reason: "contains underscore".to_string(),
                valid_examples: vec!["dev".to_string()],
            },
        };

        let message = error.to_string();
        assert!(message.contains("invalid_env"));
        assert!(message.contains("Invalid environment name"));
    }
}
