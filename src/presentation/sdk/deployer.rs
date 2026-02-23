//! Deployer facade — the main SDK entry point.
//!
//! [`Deployer`] provides typed access to all deployer operations without
//! requiring manual dependency wiring. Each method delegates to an
//! application-layer command handler.
//!
//! # Example
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
//!
//! let deployer = Deployer::builder()
//!     .working_dir("/path/to/workspace")
//!     .build()
//!     .expect("Failed to initialize deployer");
//!
//! let environments = deployer.list().expect("Failed to list environments");
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::application::command_handlers::create::CreateCommandHandlerError;
use crate::application::command_handlers::destroy::{
    DestroyCommandHandler, DestroyCommandHandlerError,
};
use crate::application::command_handlers::list::{
    EnvironmentList, ListCommandHandler, ListCommandHandlerError,
};
use crate::application::command_handlers::purge::errors::PurgeCommandHandlerError;
use crate::application::command_handlers::purge::handler::PurgeCommandHandler;
use crate::application::command_handlers::show::{
    EnvironmentInfo, ShowCommandHandler, ShowCommandHandlerError,
};
use crate::application::command_handlers::validate::{
    ValidateCommandHandler, ValidateCommandHandlerError, ValidationResult,
};
use crate::application::CreateCommandHandler;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::{Created, Destroyed};
use crate::domain::{Environment, EnvironmentName};
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::Clock;

use super::builder::DeployerBuilder;

/// The main entry point for SDK consumers.
///
/// Provides typed access to all deployer operations without requiring
/// manual dependency wiring. Construct via [`Deployer::builder`].
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::presentation::sdk::{
///     Deployer, EnvironmentCreationConfig,
/// };
///
/// let deployer = Deployer::builder()
///     .working_dir("/path/to/workspace")
///     .build()
///     .expect("Failed to initialize deployer");
///
/// // Validate a config file
/// let result = deployer
///     .validate(std::path::Path::new("envs/my-env.json"))
///     .expect("Validation failed");
/// println!("Environment: {}", result.environment_name);
/// ```
pub struct Deployer {
    working_dir: PathBuf,
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    data_directory: Arc<Path>,
}

impl Deployer {
    /// Create a new [`DeployerBuilder`].
    #[must_use]
    pub fn builder() -> DeployerBuilder {
        DeployerBuilder::new()
    }

    /// Internal constructor used by [`DeployerBuilder`].
    pub(crate) fn new(
        working_dir: PathBuf,
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        repository_factory: Arc<RepositoryFactory>,
        clock: Arc<dyn Clock>,
        data_directory: Arc<Path>,
    ) -> Self {
        Self {
            working_dir,
            repository,
            repository_factory,
            clock,
            data_directory,
        }
    }

    /// Create a new deployment environment from a configuration.
    ///
    /// Equivalent to `torrust-tracker-deployer create environment --env-file <path>`.
    ///
    /// # Errors
    ///
    /// Returns [`CreateCommandHandlerError`] if the configuration is invalid,
    /// the environment already exists, or a repository error occurs.
    pub fn create_environment(
        &self,
        config: EnvironmentCreationConfig,
    ) -> Result<Environment<Created>, CreateCommandHandlerError> {
        let handler = CreateCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        handler.execute(config, &self.working_dir)
    }

    /// Show information about an existing environment.
    ///
    /// Equivalent to `torrust-tracker-deployer show <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`ShowCommandHandlerError`] if the environment is not found
    /// or a repository error occurs.
    pub fn show(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<EnvironmentInfo, ShowCommandHandlerError> {
        let handler =
            ShowCommandHandler::new(self.repository.clone() as Arc<dyn EnvironmentRepository>);
        handler.execute(env_name)
    }

    /// List all environments in the workspace.
    ///
    /// Equivalent to `torrust-tracker-deployer list`.
    ///
    /// # Errors
    ///
    /// Returns [`ListCommandHandlerError`] if a repository error occurs.
    pub fn list(&self) -> Result<EnvironmentList, ListCommandHandlerError> {
        let handler = ListCommandHandler::new(
            Arc::clone(&self.repository_factory),
            Arc::clone(&self.data_directory),
        );
        handler.execute()
    }

    /// Validate an environment configuration file.
    ///
    /// Equivalent to `torrust-tracker-deployer validate <path>`.
    ///
    /// # Errors
    ///
    /// Returns [`ValidateCommandHandlerError`] if the file cannot be read
    /// or the configuration is invalid.
    pub fn validate(
        &self,
        config_path: &Path,
    ) -> Result<ValidationResult, ValidateCommandHandlerError> {
        let handler = ValidateCommandHandler::new();
        handler.validate(config_path)
    }

    /// Destroy the infrastructure for an environment.
    ///
    /// Equivalent to `torrust-tracker-deployer destroy <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`DestroyCommandHandlerError`] if the environment is not found,
    /// the destroy operation fails, or a repository error occurs.
    pub fn destroy(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Destroyed>, DestroyCommandHandlerError> {
        let handler = DestroyCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        handler.execute(env_name)
    }

    /// Purge all local data for an environment.
    ///
    /// This removes both the `data/{env-name}/` and `build/{env-name}/`
    /// directories. It does NOT destroy infrastructure.
    ///
    /// Equivalent to `torrust-tracker-deployer purge <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`PurgeCommandHandlerError`] if the environment is not found
    /// or the purge operation fails.
    pub fn purge(&self, env_name: &EnvironmentName) -> Result<(), PurgeCommandHandlerError> {
        let handler =
            PurgeCommandHandler::new(Arc::clone(&self.repository), self.working_dir.clone());
        handler.execute(env_name)
    }
}
