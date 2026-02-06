//! Error types for the Purge command handler

use std::path::PathBuf;

use crate::shared::ErrorKind;

/// Comprehensive error type for the `PurgeCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum PurgeCommandHandlerError {
    /// Environment was not found in the repository
    #[error("Environment not found: {name}")]
    EnvironmentNotFound {
        /// The name of the environment that was not found
        name: String,
    },

    /// Failed to remove the data directory for the environment
    #[error("Failed to remove data directory at '{path}': {source}")]
    DataDirectoryRemovalFailed {
        /// Path to the data directory that couldn't be removed
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to remove the build directory for the environment
    #[error("Failed to remove build directory at '{path}': {source}")]
    BuildDirectoryRemovalFailed {
        /// Path to the build directory that couldn't be removed
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to remove environment from repository after purge
    #[error("Failed to remove environment from repository: {0}")]
    RepositoryRemovalFailed(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for PurgeCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("PurgeCommandHandlerError: Environment not found - {name}")
            }
            Self::DataDirectoryRemovalFailed { path, source } => {
                format!(
                    "PurgeCommandHandlerError: Failed to remove data directory at '{}' - {source}",
                    path.display()
                )
            }
            Self::BuildDirectoryRemovalFailed { path, source } => {
                format!(
                    "PurgeCommandHandlerError: Failed to remove build directory at '{}' - {source}",
                    path.display()
                )
            }
            Self::RepositoryRemovalFailed(e) => {
                format!(
                    "PurgeCommandHandlerError: Failed to remove environment from repository - {e}"
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        // None of these errors wrap other Traceable errors
        // RepositoryError doesn't implement Traceable
        None
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. } => ErrorKind::Configuration,
            Self::DataDirectoryRemovalFailed { .. } | Self::BuildDirectoryRemovalFailed { .. } => {
                ErrorKind::FileSystem
            }
            Self::RepositoryRemovalFailed(_) => ErrorKind::StatePersistence,
        }
    }
}

impl PurgeCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::application::command_handlers::purge::errors::PurgeCommandHandlerError;
    ///
    /// let error = PurgeCommandHandlerError::EnvironmentNotFound {
    ///     name: "test-env".to_string(),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("Environment Not Found"));
    /// assert!(help.contains("Troubleshooting"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EnvironmentNotFound { .. } => {
                "Environment Not Found - Troubleshooting:

1. Verify the environment name is correct
2. Check if the environment exists:
   cargo run -- list

3. If the environment doesn't exist, there's nothing to purge

4. If the environment was previously purged, it has already been removed

Common causes:
- Typo in environment name
- Environment was already purged
- Working in the wrong directory (check --working-dir)

For more information, see docs/user-guide/commands.md"
            }
            Self::DataDirectoryRemovalFailed { .. } => {
                "Data Directory Removal Failed - Troubleshooting:

1. Check filesystem permissions:
   ls -la data/

2. Verify you have write access to the data directory:
   test -w data/ && echo 'writable' || echo 'not writable'

3. Check if the directory is in use:
   lsof +D data/<env-name>/

4. Common issues:
   - Permission denied: Run with appropriate permissions
   - Directory in use: Close any programs accessing the files
   - Disk full: Free up space and retry
   - Read-only filesystem: Check mount options

5. If the directory is already gone, the purge succeeded

For more information, see docs/user-guide/commands.md"
            }
            Self::BuildDirectoryRemovalFailed { .. } => {
                "Build Directory Removal Failed - Troubleshooting:

1. Check filesystem permissions:
   ls -la build/

2. Verify you have write access to the build directory:
   test -w build/ && echo 'writable' || echo 'not writable'

3. Check if the directory is in use:
   lsof +D build/<env-name>/

4. Common issues:
   - Permission denied: Run with appropriate permissions
   - Directory in use: Close any programs accessing the files
   - Disk full: Free up space and retry
   - Read-only filesystem: Check mount options

5. If the directory is already gone, the purge succeeded

For more information, see docs/user-guide/commands.md"
            }
            Self::RepositoryRemovalFailed(_) => {
                "Repository Removal Failed - Troubleshooting:

1. Check if the environment file is locked:
   lsof data/<env-name>/environment.json

2. Verify filesystem permissions on the data directory

3. Check if another process is accessing the environment:
   ps aux | grep torrust-tracker-deployer

4. This error can occur if:
   - Another deployment operation is running
   - Environment file is corrupted
   - Filesystem is read-only

5. The local directories may have been removed even if repository
   cleanup failed - verify with 'ls data/' and 'ls build/'

For more information, see docs/user-guide/commands.md"
            }
        }
    }
}
