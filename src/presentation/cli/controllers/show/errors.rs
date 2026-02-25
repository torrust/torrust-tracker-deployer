//! Error types for the Show Subcommand
//!
//! This module defines error types that can occur during CLI show command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::cli::views::progress::ProgressReporterError;

/// Show command specific errors
///
/// This enum contains all error variants specific to the show command,
/// including environment validation and loading errors.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum ShowSubcommandError {
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
        "Environment '{name}' not found
Tip: Use 'list' command to see available environments"
    )]
    EnvironmentNotFound { name: String },

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

    /// Failed to load environment from storage
    ///
    /// The environment exists but couldn't be loaded due to a storage error.
    #[error(
        "Failed to load environment '{name}': {message}
Tip: Check if the environment data is corrupted or permissions are correct"
    )]
    LoadError { name: String, message: String },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for ShowSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl ShowSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    #[must_use]
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

1. List available environments:
   - Run: torrust-tracker-deployer list

2. Verify environment exists:
   - Check: ls -la data/
   - Look for environment.json file in data/<environment-name>/

3. Create environment first:
   - Run: torrust-tracker-deployer create environment --env-file <config.json>

4. Check file permissions:
   - Read permission: chmod +r data/<environment-name>/environment.json
   - Directory access: chmod +rx data/<environment-name>/"
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Detailed Troubleshooting:

This is a critical internal error. Please:

1. Run with verbose logging:
   - Use: --log-output file-and-stderr

2. Check system resources:
   - Verify sufficient disk space
   - Check available memory

3. Report the issue:
   - Include full error output
   - Attach log files from data/logs/"
            }

            Self::LoadError { .. } => {
                "Environment Load Error - Detailed Troubleshooting:

1. Check file integrity:
   - Verify environment.json is valid JSON
   - Check for file corruption

2. Check file permissions:
   - Read permission: chmod +r data/<environment-name>/environment.json
   - Directory access: chmod +rx data/<environment-name>/

3. Try recreating the environment:
   - Remove corrupted data: rm -rf data/<environment-name>
   - Create new environment: torrust-tracker-deployer create environment --env-file <config.json>"
            }
        }
    }
}
