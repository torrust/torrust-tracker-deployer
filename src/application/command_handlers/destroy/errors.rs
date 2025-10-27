//! Error types for the Destroy command handler

use std::path::PathBuf;

use crate::adapters::tofu::client::OpenTofuError;
use crate::domain::environment::state::StateTypeError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `DestroyCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum DestroyCommandHandlerError {
    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),

    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    #[error("Failed to clean up state files at '{path}': {source}")]
    StateCleanupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl crate::shared::Traceable for DestroyCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofu(e) => {
                format!("DestroyCommandHandlerError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("DestroyCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("DestroyCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::StateTransition(e) => {
                format!("DestroyCommandHandlerError: Invalid state transition - {e}")
            }
            Self::StateCleanupFailed { path, source } => {
                format!(
                    "DestroyCommandHandlerError: Failed to clean up state files at '{}' - {source}",
                    path.display()
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::StatePersistence(_)
            | Self::StateTransition(_)
            | Self::StateCleanupFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StateTransition(_) => crate::shared::ErrorKind::Configuration,
            Self::StatePersistence(_) | Self::StateCleanupFailed { .. } => {
                crate::shared::ErrorKind::StatePersistence
            }
        }
    }
}
