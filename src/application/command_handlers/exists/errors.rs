//! Error types for exists command handler

use crate::application::errors::PersistenceError;
use crate::shared::error::kind::ErrorKind;
use crate::shared::error::traceable::Traceable;

/// Comprehensive error type for the `ExistsCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ExistsCommandHandlerError {
    #[error("Failed to check environment existence: {0}")]
    RepositoryError(#[from] PersistenceError),
}

impl From<crate::domain::environment::repository::RepositoryError> for ExistsCommandHandlerError {
    fn from(e: crate::domain::environment::repository::RepositoryError) -> Self {
        Self::RepositoryError(e.into())
    }
}

impl Traceable for ExistsCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::RepositoryError(e) => {
                format!("ExistsCommandHandlerError: Repository error - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::RepositoryError(_) => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::RepositoryError(_) => ErrorKind::StatePersistence,
        }
    }
}

impl ExistsCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// # Example
    ///
    /// ```
    /// use torrust_tracker_deployer_lib::application::command_handlers::exists::errors::ExistsCommandHandlerError;
    /// use torrust_tracker_deployer_lib::application::errors::PersistenceError;
    ///
    /// let error = ExistsCommandHandlerError::RepositoryError(PersistenceError::NotFound);
    ///
    /// let help = error.help();
    /// assert!(help.contains("Repository Error"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::RepositoryError(_) => {
                "Repository Error - Troubleshooting:

1. Check if the data directory exists and is accessible:
   ls -la data/

2. Verify file system permissions:
   ls -la data/

3. Check for disk space issues:
   df -h .

Common causes:
- File system permissions issues
- Disk full or read-only filesystem
- Corrupted data directory

For more information, see docs/user-guide/commands.md"
            }
        }
    }
}
