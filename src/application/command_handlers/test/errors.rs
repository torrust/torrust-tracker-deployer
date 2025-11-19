//! Error types for test command handler

use crate::domain::environment::repository::RepositoryError;
use crate::domain::environment::state::StateTypeError;
use crate::infrastructure::remote_actions::RemoteActionError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `TestCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum TestCommandHandlerError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Remote action failed: {0}")]
    RemoteAction(#[from] RemoteActionError),

    #[error("Environment '{environment_name}' does not have an instance IP set. The environment must be provisioned before running tests.")]
    MissingInstanceIp { environment_name: String },

    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    #[error("State persistence error: {0}")]
    StatePersistence(#[from] RepositoryError),
}
