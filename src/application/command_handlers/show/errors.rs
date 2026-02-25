//! Error types for show command handler

use crate::application::errors::PersistenceError;
use crate::shared::error::kind::ErrorKind;
use crate::shared::error::traceable::Traceable;

/// Comprehensive error type for the `ShowCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ShowCommandHandlerError {
    #[error("Environment not found: '{name}'")]
    EnvironmentNotFound { name: String },

    #[error("Failed to load environment: {0}")]
    LoadError(#[from] PersistenceError),
}

impl From<crate::domain::environment::repository::RepositoryError> for ShowCommandHandlerError {
    fn from(e: crate::domain::environment::repository::RepositoryError) -> Self {
        Self::LoadError(e.into())
    }
}

impl Traceable for ShowCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("ShowCommandHandlerError: Environment not found - '{name}'")
            }
            Self::LoadError(e) => {
                format!("ShowCommandHandlerError: Failed to load environment - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::EnvironmentNotFound { .. } | Self::LoadError(_) => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. } => ErrorKind::Configuration,
            Self::LoadError(_) => ErrorKind::StatePersistence,
        }
    }
}

impl ShowCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// # Example
    ///
    /// ```
    /// use torrust_tracker_deployer_lib::application::command_handlers::show::errors::ShowCommandHandlerError;
    ///
    /// let error = ShowCommandHandlerError::EnvironmentNotFound {
    ///     name: "my-env".to_string(),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("Verify the environment name"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EnvironmentNotFound { .. } => {
                "Environment Not Found - Troubleshooting:

1. Verify the environment name is correct
2. Check if the environment was created:
   ls data/

3. If the environment doesn't exist, create it first:
   cargo run -- create environment --env-file <config.json>

4. If the environment was previously destroyed, the state may have been removed

Common causes:
- Typo in environment name
- Environment was destroyed and state was removed
- Working in the wrong directory

For more information, see docs/user-guide/commands.md"
            }
            Self::LoadError(_) => {
                "Environment Load Error - Troubleshooting:

1. Check if the environment state file exists:
   ls data/<env-name>/environment.json

2. Verify the file is valid JSON:
   cat data/<env-name>/environment.json | jq .

3. Ensure the file is readable:
   ls -la data/<env-name>/environment.json

4. Check for disk space issues or file system errors

Common causes:
- Corrupted environment state file
- Interrupted write operation
- File system permissions issues

For more information, see docs/user-guide/commands.md"
            }
        }
    }
}
