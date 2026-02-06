//! Purge command handler implementation

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument, warn};

use super::errors::PurgeCommandHandlerError;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::EnvironmentName;

/// `PurgeCommandHandler` orchestrates the removal of all local environment data
///
/// This command handler removes all local files associated with an environment:
/// 1. Removes the `data/{env-name}/` directory (environment state, configs, etc.)
/// 2. Removes the `build/{env-name}/` directory (generated templates, artifacts)
/// 3. Removes the environment entry from the repository
///
/// # State Management
///
/// Unlike other commands, purge **does not transition environment state**:
/// - Works on environments in any state
/// - Removes all local data regardless of current state
/// - Does not persist state after purge (the environment data is removed)
///
/// # Idempotency
///
/// The purge operation is idempotent. Running it multiple times on the same
/// environment will:
/// - Succeed if the directories are already removed
/// - Not fail due to missing resources
/// - Report appropriate status to the user
///
/// # Important Notes
///
/// - **Does NOT destroy infrastructure**: Only removes local files
/// - **Irreversible operation**: All local environment data is permanently deleted
/// - **Works in any state**: Can purge environments that are Created, Provisioned, Running, etc.
pub struct PurgeCommandHandler {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    working_dir: PathBuf,
}

impl PurgeCommandHandler {
    /// Create a new `PurgeCommandHandler`
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository for accessing environment data
    /// * `working_dir` - Root directory containing `data/` and `build/` subdirectories
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        working_dir: PathBuf,
    ) -> Self {
        Self {
            repository,
            working_dir,
        }
    }

    /// Execute the complete purge workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to purge
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found in repository
    /// * Unable to remove data directory due to permissions or I/O errors
    /// * Unable to remove build directory due to permissions or I/O errors
    /// * Unable to remove environment from repository
    ///
    /// If directories are already removed, the operation succeeds (idempotent).
    #[instrument(
        name = "purge_command",
        skip_all,
        fields(
            command_type = "purge",
            environment = %env_name
        )
    )]
    pub fn execute(&self, env_name: &EnvironmentName) -> Result<(), PurgeCommandHandlerError> {
        // Verify environment exists
        self.verify_environment_exists(env_name)?;

        // Remove data directory
        self.remove_data_directory(env_name)?;

        // Remove build directory
        self.remove_build_directory(env_name)?;

        // Remove from repository (this also removes the environment.json file)
        self.remove_from_repository(env_name)?;

        info!(
            command = "purge",
            environment = %env_name,
            "Environment purged successfully"
        );

        Ok(())
    }

    /// Verify environment exists in repository
    fn verify_environment_exists(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), PurgeCommandHandlerError> {
        match self.repository.exists(env_name) {
            Ok(true) => Ok(()),
            Ok(false) => Err(PurgeCommandHandlerError::EnvironmentNotFound {
                name: env_name.to_string(),
            }),
            Err(e) => {
                warn!(
                    command = "purge",
                    environment = %env_name,
                    error = %e,
                    "Failed to check if environment exists, proceeding anyway"
                );
                // Don't fail the purge if we can't check existence
                // The user may be trying to clean up a corrupted environment
                Ok(())
            }
        }
    }

    /// Remove the data directory for the environment
    fn remove_data_directory(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), PurgeCommandHandlerError> {
        let data_dir = self.working_dir.join("data").join(env_name.as_str());

        if !data_dir.exists() {
            info!(
                command = "purge",
                environment = %env_name,
                path = %data_dir.display(),
                "Data directory does not exist, skipping removal"
            );
            return Ok(());
        }

        info!(
            command = "purge",
            environment = %env_name,
            path = %data_dir.display(),
            "Removing data directory"
        );

        std::fs::remove_dir_all(&data_dir).map_err(|source| {
            PurgeCommandHandlerError::DataDirectoryRemovalFailed {
                path: data_dir,
                source,
            }
        })?;

        Ok(())
    }

    /// Remove the build directory for the environment
    fn remove_build_directory(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), PurgeCommandHandlerError> {
        let build_dir = self.working_dir.join("build").join(env_name.as_str());

        if !build_dir.exists() {
            info!(
                command = "purge",
                environment = %env_name,
                path = %build_dir.display(),
                "Build directory does not exist, skipping removal"
            );
            return Ok(());
        }

        info!(
            command = "purge",
            environment = %env_name,
            path = %build_dir.display(),
            "Removing build directory"
        );

        std::fs::remove_dir_all(&build_dir).map_err(|source| {
            PurgeCommandHandlerError::BuildDirectoryRemovalFailed {
                path: build_dir,
                source,
            }
        })?;

        Ok(())
    }

    /// Remove environment from repository
    fn remove_from_repository(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), PurgeCommandHandlerError> {
        info!(
            command = "purge",
            environment = %env_name,
            "Removing environment from repository"
        );

        self.repository.delete(env_name)?;

        Ok(())
    }
}
