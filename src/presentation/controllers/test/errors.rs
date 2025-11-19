//! Error types for the Test Subcommand
//!
//! This module defines error types that can occur during CLI test command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::test::errors::TestCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Test command specific errors
///
/// This enum contains all error variants specific to the test command,
/// including environment validation, repository access, and validation failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum TestSubcommandError {
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
        "Environment '{name}' not found in data directory '{data_dir}'
Tip: Check if environment exists: ls -la {data_dir}/"
    )]
    EnvironmentNotFound { name: String, data_dir: String },

    /// Environment does not have instance IP set
    ///
    /// The environment is missing the instance IP, which means it hasn't been provisioned yet.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' does not have instance IP set
Tip: Environment must be provisioned before testing"
    )]
    MissingInstanceIp { name: String },

    // ===== Validation Operation Errors =====
    /// Validation operation failed
    ///
    /// The validation process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Validation failed for environment '{name}': {source}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    ValidationFailed {
        name: String,
        #[source]
        source: Box<TestCommandHandlerError>,
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

impl From<ProgressReporterError> for TestSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl TestSubcommandError {
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
    /// use torrust_tracker_deployer_lib::presentation::controllers::test;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let container = Container::new(VerbosityLevel::Normal);
    /// let context = ExecutionContext::new(Arc::new(container));
    ///
    /// if let Err(e) = test::handle("test-env", Path::new("."), &context).await {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    ///
    /// Direct usage (for testing):
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::sync::Arc;
    /// use parking_lot::ReentrantMutex;
    /// use std::cell::RefCell;
    /// use torrust_tracker_deployer_lib::presentation::controllers::test;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
    /// if let Err(e) = test::handle_test_command("test-env", Path::new("."), repository_factory, &output).await {
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
   - Start: Must begin with a letter or digit
   - Characters: Only letters, digits, and hyphens allowed
   - No special characters: Avoid spaces, underscores, dots

2. Valid examples:
   - 'production'
   - 'staging-01'
   - 'dev-environment'

3. Invalid examples:
   - 'prod_01' (underscore not allowed)
   - '-production' (cannot start with hyphen)
   - 'prod.env' (dots not allowed)

For more information, see environment naming documentation."
            }

            Self::EnvironmentNotFound { .. } => {
                "Environment Not Found - Detailed Troubleshooting:

1. Verify environment exists:
   - List environments: ls -la data/
   - Check for environment.json file in data/<environment-name>/

2. Check file permissions:
   - Read permission: chmod +r data/<environment-name>/environment.json
   - Directory access: chmod +rx data/<environment-name>/

3. Create environment first:
   - Run: torrust-tracker-deployer create <environment-name>

4. Verify data directory:
   - Ensure data/ directory exists
   - Check disk space: df -h"
            }

            Self::MissingInstanceIp { .. } => {
                "Missing Instance IP - Detailed Troubleshooting:

1. Environment must be provisioned before testing:
   - Run: torrust-tracker-deployer provision <environment-name>

2. Verify provisioning status:
   - Check environment state in data/<environment-name>/environment.json
   - Look for 'instance_ip' field

3. Common causes:
   - Environment was created but never provisioned
   - Previous provision operation failed
   - Manual modification of environment.json

4. Next steps:
   - Provision the environment: torrust-tracker-deployer provision <environment-name>
   - Or destroy and recreate: torrust-tracker-deployer destroy <environment-name>"
            }

            Self::ValidationFailed { .. } => {
                "Validation Failed - Detailed Troubleshooting:

1. Check validation logs for specific failure:
   - Re-run with verbose logging:
     torrust-tracker-deployer test <environment-name> --log-output file-and-stderr

2. Common validation failures:
   - Cloud-init not completed: Wait for instance initialization
   - Docker not installed: Run configure command
   - Docker Compose not installed: Run configure command

3. Remediation steps:
   - If cloud-init failed: Destroy and re-provision
   - If Docker/Compose missing: Run configure command
     torrust-tracker-deployer configure <environment-name>

4. Check instance status:
   - Verify instance is running
   - Check SSH connectivity
   - Review system logs on the instance"
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This is a critical bug that should be reported to the development team.

1. Gather diagnostic information:
   - Re-run command with full logging:
     torrust-tracker-deployer test <environment-name> --log-output file-and-stderr
   - Capture all error output

2. Report the issue:
   - Include full error message
   - Include command that triggered the error
   - Include environment information (OS, version)
   - Attach log files from data/logs/

3. Temporary workaround:
   - None available - this indicates a serious internal error
   - Try restarting the application

For bug reports, visit:
https://github.com/torrust/torrust-tracker-deployer/issues"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_environment_name_help_message() {
        let error = TestSubcommandError::InvalidEnvironmentName {
            name: "invalid_name".to_string(),
            source: EnvironmentNameError::InvalidFormat {
                attempted_name: "invalid_name".to_string(),
                reason: "contains underscore".to_string(),
                valid_examples: vec!["dev".to_string(), "staging".to_string()],
            },
        };

        let help = error.help();
        assert!(help.contains("Invalid Environment Name"));
        assert!(help.contains("1-63 characters"));
        assert!(help.contains("Valid examples"));
    }

    #[test]
    fn test_environment_not_found_help_message() {
        let error = TestSubcommandError::EnvironmentNotFound {
            name: "test-env".to_string(),
            data_dir: "/path/to/data".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Found"));
        assert!(help.contains("ls -la data/"));
        assert!(help.contains("torrust-tracker-deployer create"));
    }

    #[test]
    fn test_missing_instance_ip_help_message() {
        let error = TestSubcommandError::MissingInstanceIp {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Missing Instance IP"));
        assert!(help.contains("torrust-tracker-deployer provision"));
    }

    #[test]
    fn test_validation_failed_help_message() {
        let error = TestSubcommandError::ValidationFailed {
            name: "test-env".to_string(),
            source: Box::new(TestCommandHandlerError::MissingInstanceIp {
                environment_name: "test-env".to_string(),
            }),
        };

        let help = error.help();
        assert!(help.contains("Validation Failed"));
        assert!(help.contains("--log-output file-and-stderr"));
        assert!(help.contains("Cloud-init"));
        assert!(help.contains("Docker"));
    }
}
