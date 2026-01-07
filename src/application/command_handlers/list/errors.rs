//! Error types for list command handler

use std::path::PathBuf;

use crate::shared::error::kind::ErrorKind;
use crate::shared::error::traceable::Traceable;

/// Comprehensive error type for the `ListCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ListCommandHandlerError {
    /// Data directory not found
    #[error("Data directory not found: '{path}'")]
    DataDirectoryNotFound { path: PathBuf },

    /// Permission denied accessing directory
    #[error("Permission denied accessing directory: '{path}'")]
    PermissionDenied { path: PathBuf },

    /// Failed to scan environments directory
    #[error("Failed to scan environments directory: {message}")]
    ScanError { message: String },
}

impl Traceable for ListCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::DataDirectoryNotFound { path } => {
                format!(
                    "ListCommandHandlerError: Data directory not found - '{}'",
                    path.display()
                )
            }
            Self::ScanError { message } => {
                format!("ListCommandHandlerError: Scan error - {message}")
            }
            Self::PermissionDenied { path } => {
                format!(
                    "ListCommandHandlerError: Permission denied - '{}'",
                    path.display()
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        None
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::DataDirectoryNotFound { .. }
            | Self::ScanError { .. }
            | Self::PermissionDenied { .. } => ErrorKind::FileSystem,
        }
    }
}

impl ListCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::application::command_handlers::list::errors::ListCommandHandlerError;
    ///
    /// let error = ListCommandHandlerError::DataDirectoryNotFound {
    ///     path: PathBuf::from("./data"),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("Verify current directory"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DataDirectoryNotFound { .. } => {
                "Data Directory Not Found - Troubleshooting:

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
            Self::ScanError { .. } => {
                "Scan Error - Troubleshooting:

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
            Self::PermissionDenied { .. } => {
                "Permission Denied - Troubleshooting:

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
        }
    }
}
