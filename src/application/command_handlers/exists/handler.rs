//! Exists command handler implementation
//!
//! **Purpose**: Check whether an environment exists
//!
//! This handler checks whether an environment with the given name exists
//! in the data directory. It is a read-only operation that does not modify
//! any state or make any network calls.
//!
//! ## Design Rationale
//!
//! - "Not found" is a valid result (`exists = false`), NOT an error
//! - Only repository access failures produce errors
//! - Returns a simple boolean result via `ExistsResult`

use std::sync::Arc;

use tracing::instrument;

use super::errors::ExistsCommandHandlerError;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::EnvironmentName;

/// Result of checking whether an environment exists
#[derive(Debug, Clone)]
pub struct ExistsResult {
    /// The environment name that was checked
    pub name: String,
    /// Whether the environment exists
    pub exists: bool,
}

/// `ExistsCommandHandler` checks whether an environment exists
///
/// **Purpose**: Read-only existence check against the repository
///
/// This handler queries the repository to determine if an environment
/// with the given name exists. It never modifies state or makes network calls.
pub struct ExistsCommandHandler {
    repository: Arc<dyn EnvironmentRepository>,
}

impl ExistsCommandHandler {
    /// Create a new `ExistsCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    /// Execute the exists command workflow
    ///
    /// Checks whether the named environment exists in the repository.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to check
    ///
    /// # Returns
    ///
    /// * `Ok(ExistsResult)` - Result indicating whether the environment exists
    /// * `Err(ExistsCommandHandlerError)` - If the repository check fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Repository access fails (file system error, permissions, etc.)
    #[instrument(
        name = "exists_command",
        skip_all,
        fields(
            command_type = "exists",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<ExistsResult, ExistsCommandHandlerError> {
        let exists = self.repository.exists(env_name)?;

        Ok(ExistsResult {
            name: env_name.to_string(),
            exists,
        })
    }
}
