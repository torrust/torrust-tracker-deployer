//! Error types for the Configure command handler

use crate::shared::command::CommandError;

/// Comprehensive error type for the `ConfigureCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ConfigureCommandHandlerError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ConfigureCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::Command(e) => {
                format!("ConfigureCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ConfigureCommandHandlerError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::Command(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}
