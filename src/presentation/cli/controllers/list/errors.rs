//! Error types for the List Subcommand
//!
//! This module defines error types that can occur during CLI list command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use std::path::PathBuf;

use thiserror::Error;

use crate::presentation::cli::views::progress::ProgressReporterError;
use crate::presentation::cli::views::ViewRenderError;

/// List command specific errors
///
/// This enum contains all error variants specific to the list command,
/// including directory access and scanning errors.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum ListSubcommandError {
    // ===== Data Directory Errors =====
    /// Data directory not found
    ///
    /// The data directory where environments are stored does not exist.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Data directory not found: '{path}'
Tip: Run from the deployer workspace directory or specify --working-dir"
    )]
    DataDirectoryNotFound { path: PathBuf },

    /// Permission denied accessing directory
    ///
    /// Access to the data directory was denied.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Permission denied accessing directory: '{path}'
Tip: Check file permissions for the data directory"
    )]
    PermissionDenied { path: PathBuf },

    /// Failed to scan environments directory
    ///
    /// An error occurred while scanning the data directory.
    #[error(
        "Failed to scan environments directory: {message}
Tip: Check filesystem health and permissions"
    )]
    ScanError { message: String },

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

impl From<ProgressReporterError> for ListSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}
impl From<ViewRenderError> for ListSubcommandError {
    fn from(e: ViewRenderError) -> Self {
        Self::OutputFormatting {
            reason: e.to_string(),
        }
    }
}

impl ListSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DataDirectoryNotFound { .. } => {
                "Data Directory Not Found - Detailed Troubleshooting:

1. Verify current directory:
   - Run: pwd
   - Expected: Your deployer workspace directory

2. Check if data directory exists:
   - Run: ls -la data/
   - Should contain environment subdirectories

3. Create environment first:
   - Run: torrust-tracker-deployer create environment --env-file <config.json>

Common causes:
- Running from the wrong directory
- No environments have been created yet
- Data directory was moved or deleted

For more information, see docs/user-guide/commands.md"
            }
            Self::PermissionDenied { .. } => {
                "Permission Denied - Detailed Troubleshooting:

1. Check directory permissions:
   - Run: ls -ld data/
   - Should have read permission (r--)

2. Check file permissions:
   - Run: ls -l data/*/environment.json
   - Should have read permission (r--)

3. Fix permissions if needed:
   - Run: chmod +rx data/
   - Run: chmod +r data/*/environment.json

Common causes:
- File created by different user
- Restrictive umask settings
- SELinux or AppArmor restrictions

For more information, see docs/user-guide/commands.md"
            }
            Self::ScanError { .. } => {
                "Scan Error - Detailed Troubleshooting:

1. Check directory permissions:
   - Run: ls -ld data/
   - Should have read permission (r--)

2. Verify filesystem health:
   - Check for disk errors or filesystem issues

3. Try running with elevated permissions if needed

Common causes:
- File system errors
- Corrupted directory entries
- Network filesystem issues

For more information, see docs/user-guide/commands.md"
            }
            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - This is an internal error:

1. This indicates a bug in the application
2. Please report this issue with:
   - Full command output
   - Log file contents (use --log-output file-and-stderr)
   - Steps to reproduce

Report issues at: https://github.com/torrust/torrust-tracker-deployer/issues"
            }
            Self::OutputFormatting { .. } => {
                "Output Formatting Failed - Critical Internal Error:\n\nThis error should not occur during normal operation. It indicates a bug in the output formatting system.\n\n1. Immediate actions:\n   - Save full error output\n   - Copy log files from data/logs/\n   - Note the exact command and output format being used\n\n2. Report the issue:\n   - Create GitHub issue with full details\n   - Include: command, output format (--output-format), error output, logs\n   - Describe steps to reproduce\n\n3. Temporary workarounds:\n   - Try using different output format (text vs json)\n   - Try running command again\n\nPlease report it so we can fix it."
            }
        }
    }
}
