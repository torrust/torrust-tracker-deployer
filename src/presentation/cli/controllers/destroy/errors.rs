//! Error types for the Destroy Subcommand
//!
//! This module defines error types that can occur during CLI destroy command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::cli::views::progress::ProgressReporterError;
use crate::presentation::cli::views::ViewRenderError;

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
    /// Output formatting failed (JSON serialization error).
    /// This indicates an internal error in data serialization.
    #[error(
        "Failed to format output: {reason}\nTip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    OutputFormatting { reason: String },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for DestroySubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}
impl From<ViewRenderError> for DestroySubcommandError {
    fn from(e: ViewRenderError) -> Self {
        Self::OutputFormatting {
            reason: e.to_string(),
        }
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
    /// ```ignore
    /// use std::path::Path;
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::cli::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::cli::controllers::destroy;
    /// use torrust_tracker_deployer_lib::presentation::cli::views::VerbosityLevel;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// if let Err(e) = context
    ///     .container()
    ///     .create_destroy_controller()
    ///     .execute("test-env", output_format)
    ///     .await
    /// {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    ///
    /// Direct usage (for testing):
    ///
    /// ```ignore
    /// use std::path::{Path, PathBuf};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use parking_lot::ReentrantMutex;
    /// use std::cell::RefCell;
    /// use torrust_tracker_deployer_lib::presentation::cli::controllers::destroy::handler::DestroyCommandController;
    /// use torrust_tracker_deployer_lib::presentation::cli::views::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::clock::SystemClock;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let data_dir = PathBuf::from("./data");
    /// let file_repository_factory = FileRepositoryFactory::new(Duration::from_secs(30));
    /// let repository = file_repository_factory.create(data_dir);
    /// let clock = Arc::new(SystemClock);
    /// if let Err(e) = DestroyCommandController::new(repository, clock, output).execute("test-env", output_format).await {
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
   - Verify OpenTofu and provider tools are accessible
   - Check if VMs/servers are running using provider tools
   - Ensure cleanup tools are available

4. Manual intervention may be needed:
   - Some resources might need manual cleanup
   - Check provider-specific tools (tofu state list)
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
            Self::OutputFormatting { .. } => {
                "Output Formatting Failed - Critical Internal Error:\n\nThis error should not occur during normal operation. It indicates a bug in the output formatting system.\n\n1. Immediate actions:\n   - Save full error output\n   - Copy log files from data/logs/\n   - Note the exact command and output format being used\n\n2. Report the issue:\n   - Create GitHub issue with full details\n   - Include: command, output format (--output-format), error output, logs\n   - Describe steps to reproduce\n\n3. Temporary workarounds:\n   - Try using different output format (text vs json)\n   - Try running command again\n\nPlease report it so we can fix it."
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
