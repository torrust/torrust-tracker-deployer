//! Error types for the Exists Subcommand
//!
//! This module defines error types that can occur during CLI exists command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::cli::views::ViewRenderError;

/// Exists command specific errors
///
/// This enum contains all error variants specific to the exists command,
/// including environment name validation and repository errors.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum ExistsSubcommandError {
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

    /// Failed to check environment existence
    ///
    /// The existence check failed due to a repository error.
    #[error(
        "Failed to check if environment '{name}' exists: {message}
Tip: Check file permissions and disk space"
    )]
    ExistenceCheckFailed { name: String, message: String },

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

impl From<ViewRenderError> for ExistsSubcommandError {
    fn from(e: ViewRenderError) -> Self {
        Self::OutputFormatting {
            reason: e.to_string(),
        }
    }
}

impl ExistsSubcommandError {
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

            Self::ExistenceCheckFailed { .. } => {
                "Existence Check Failed - Detailed Troubleshooting:

1. Check if the data directory exists and is accessible:
   ls -la data/

2. Verify file system permissions:
   ls -la data/

3. Check for disk space issues:
   df -h .

4. Ensure no file locks are held:
   Check if another process is accessing the data directory

Common causes:
- File system permissions issues
- Disk full or read-only filesystem
- Corrupted data directory"
            }

            Self::OutputFormatting { .. } => {
                "Output Formatting Failed - Critical Internal Error:

This error should not occur during normal operation. It indicates a bug in the output formatting system.

1. Immediate actions:
   - Save full error output
   - Copy log files from data/logs/
   - Note the exact command and output format being used

2. Report the issue:
   - Create GitHub issue with full details
   - Include: command, output format (--output-format), error output, logs
   - Describe steps to reproduce

3. Temporary workarounds:
   - Try using different output format (text vs json)
   - Try running command again

Please report it so we can fix it."
            }
        }
    }
}
