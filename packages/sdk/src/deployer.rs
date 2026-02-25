//! Deployer facade — the main SDK entry point.
//!
//! [`Deployer`] provides typed access to all deployer operations without
//! requiring manual dependency wiring. Each method delegates to an
//! application-layer command handler.
//!
//! # Example
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_sdk::Deployer;
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

use torrust_tracker_deployer_lib::application::command_handlers::configure::{
    ConfigureCommandHandler, ConfigureCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandlerError;
use torrust_tracker_deployer_lib::application::command_handlers::destroy::{
    DestroyCommandHandler, DestroyCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::list::{
    EnvironmentList, ListCommandHandler, ListCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::provision::{
    ProvisionCommandHandler, ProvisionCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::purge::errors::PurgeCommandHandlerError;
use torrust_tracker_deployer_lib::application::command_handlers::purge::handler::PurgeCommandHandler;
use torrust_tracker_deployer_lib::application::command_handlers::release::{
    ReleaseCommandHandler, ReleaseCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::run::{
    RunCommandHandler, RunCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::show::{
    EnvironmentInfo, ShowCommandHandler, ShowCommandHandlerError,
};
use torrust_tracker_deployer_lib::application::command_handlers::test::{
    TestCommandHandler, TestCommandHandlerError, TestResult,
};
use torrust_tracker_deployer_lib::application::command_handlers::validate::{
    ValidateCommandHandler, ValidateCommandHandlerError, ValidationResult,
};
use torrust_tracker_deployer_lib::application::traits::CommandProgressListener;
use torrust_tracker_deployer_lib::application::traits::RepositoryProvider;
use torrust_tracker_deployer_lib::application::CreateCommandHandler;
use torrust_tracker_deployer_lib::domain::environment::repository::EnvironmentRepository;
use torrust_tracker_deployer_lib::domain::EnvironmentName;
use torrust_tracker_deployer_lib::shared::Clock;

use super::builder::DeployerBuilder;
use super::error::CreateEnvironmentFromFileError;

/// The main entry point for SDK consumers.
///
/// Provides typed access to all deployer operations without requiring
/// manual dependency wiring. Construct via [`Deployer::builder`].
///
/// `Deployer` is `Clone`, `Send`, and `Sync` — it can be shared across threads
/// or stored in `Arc<Deployer>` for concurrent workflows.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_sdk::{
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
#[derive(Clone)]
pub struct Deployer {
    working_dir: PathBuf,
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    file_repository_factory: Arc<dyn RepositoryProvider>,
    clock: Arc<dyn Clock>,
    data_directory: Arc<Path>,
    listener: Arc<dyn CommandProgressListener + Send + Sync>,
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
        file_repository_factory: Arc<dyn RepositoryProvider>,
        clock: Arc<dyn Clock>,
        data_directory: Arc<Path>,
        listener: Arc<dyn CommandProgressListener + Send + Sync>,
    ) -> Self {
        Self {
            working_dir,
            repository,
            file_repository_factory,
            clock,
            data_directory,
            listener,
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
    ) -> Result<EnvironmentName, CreateCommandHandlerError> {
        let handler = CreateCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        handler
            .execute(config, &self.working_dir)
            .map(|env| env.name().clone())
    }

    /// Create a new deployment environment from a JSON configuration file.
    ///
    /// This is a convenience wrapper that reads the file, parses the JSON, and
    /// creates the environment in one step — mirroring the CLI's
    /// `--env-file <path>` flag.
    ///
    /// # Errors
    ///
    /// Returns [`CreateEnvironmentFromFileError::Load`] if the file cannot be
    /// read or is malformed, or [`CreateEnvironmentFromFileError::Create`] if
    /// the environment creation fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use torrust_tracker_deployer_sdk::Deployer;
    ///
    /// let deployer = Deployer::builder()
    ///     .working_dir("/path/to/workspace")
    ///     .build()
    ///     .unwrap();
    ///
    /// let env_name = deployer
    ///     .create_environment_from_file(Path::new("envs/my-env.json"))
    ///     .unwrap();
    /// println!("Created: {env_name}");
    /// ```
    pub fn create_environment_from_file(
        &self,
        path: &Path,
    ) -> Result<EnvironmentName, CreateEnvironmentFromFileError> {
        let config = EnvironmentCreationConfig::from_file(path)?;
        Ok(self.create_environment(config)?)
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

    /// Check whether a named environment exists in the workspace.
    ///
    /// Returns `Ok(true)` if the environment is found, `Ok(false)` if it does
    /// not exist, or `Err` for unexpected repository failures.
    ///
    /// This is a convenience wrapper around [`show`](Self::show) that avoids
    /// forcing callers to pattern-match on `ShowCommandHandlerError`.
    ///
    /// # Errors
    ///
    /// Returns [`ShowCommandHandlerError::LoadError`] if the repository
    /// encounters an unexpected error (filesystem/IO). A "not found" result
    /// is **not** an error — it returns `Ok(false)`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_sdk::Deployer;
    /// use torrust_tracker_deployer_lib::domain::EnvironmentName;
    ///
    /// let deployer = Deployer::builder()
    ///     .working_dir("/path/to/workspace")
    ///     .build()
    ///     .unwrap();
    ///
    /// let name = EnvironmentName::new("my-env").unwrap();
    /// if deployer.exists(&name).unwrap() {
    ///     println!("environment already exists");
    /// }
    /// ```
    pub fn exists(&self, env_name: &EnvironmentName) -> Result<bool, ShowCommandHandlerError> {
        match self.show(env_name) {
            Ok(_) => Ok(true),
            Err(ShowCommandHandlerError::EnvironmentNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
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
            Arc::clone(&self.file_repository_factory),
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
    pub fn destroy(&self, env_name: &EnvironmentName) -> Result<(), DestroyCommandHandlerError> {
        let handler = DestroyCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        handler.execute(env_name).map(|_| ())
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

    // ===================================================================
    // Async operations — require infrastructure (LXD / SSH / cloud)
    // ===================================================================

    /// Provision infrastructure for a created environment.
    ///
    /// Runs `OpenTofu` to create the VM instance, waits for SSH connectivity,
    /// and transitions the environment to the `Provisioned` state.
    ///
    /// Equivalent to `torrust-tracker-deployer provision <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`ProvisionCommandHandlerError`] if the environment is not found,
    /// is in the wrong state, or provisioning fails.
    pub async fn provision(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), ProvisionCommandHandlerError> {
        let handler = ProvisionCommandHandler::new(
            Arc::clone(&self.clock),
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
        );
        let listener: &dyn CommandProgressListener = &*self.listener;
        handler.execute(env_name, Some(listener)).await.map(|_| ())
    }

    /// Configure a provisioned environment.
    ///
    /// Runs Ansible playbooks to install required software and configure the
    /// VM, transitioning the environment to the `Configured` state.
    ///
    /// Equivalent to `torrust-tracker-deployer configure <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigureCommandHandlerError`] if the environment is not
    /// found, is in the wrong state, or configuration fails.
    pub fn configure(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), ConfigureCommandHandlerError> {
        let handler = ConfigureCommandHandler::new(
            Arc::clone(&self.clock),
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
        );
        let listener: &dyn CommandProgressListener = &*self.listener;
        handler.execute(env_name, Some(listener)).map(|_| ())
    }

    /// Release software to a configured environment.
    ///
    /// Renders and deploys configuration files (Docker Compose, Caddy,
    /// Prometheus, Grafana, etc.), transitioning the environment to the
    /// `Released` state.
    ///
    /// Equivalent to `torrust-tracker-deployer release <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`ReleaseCommandHandlerError`] if the environment is not found,
    /// is in the wrong state, or the release operation fails.
    pub async fn release(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), ReleaseCommandHandlerError> {
        let handler = ReleaseCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        let listener: &dyn CommandProgressListener = &*self.listener;
        handler.execute(env_name, Some(listener)).await.map(|_| ())
    }

    /// Start services on a released environment.
    ///
    /// Runs `docker compose up` on the remote instance, transitioning the
    /// environment to the `Running` state.
    ///
    /// Equivalent to `torrust-tracker-deployer run <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`RunCommandHandlerError`] if the environment is not found,
    /// is in the wrong state, or starting services fails.
    #[allow(clippy::result_large_err)]
    pub fn run_services(&self, env_name: &EnvironmentName) -> Result<(), RunCommandHandlerError> {
        let handler = RunCommandHandler::new(
            self.repository.clone() as Arc<dyn EnvironmentRepository>,
            Arc::clone(&self.clock),
        );
        handler.execute(env_name).map(|_| ())
    }

    /// Test a deployed environment.
    ///
    /// Verifies connectivity and DNS resolution for the running instance.
    ///
    /// Equivalent to `torrust-tracker-deployer test <name>`.
    ///
    /// # Errors
    ///
    /// Returns [`TestCommandHandlerError`] if the environment is not found
    /// or the test fails.
    pub async fn test(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<TestResult, TestCommandHandlerError> {
        let handler =
            TestCommandHandler::new(self.repository.clone() as Arc<dyn EnvironmentRepository>);
        handler.execute(env_name).await
    }
}

/// Compile-time assertions that [`Deployer`] satisfies `Send + Sync`.
///
/// These fail to compile if any inner field loses thread-safety.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Deployer>();
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `Deployer` implements `Clone`, `Send`, and `Sync`
    /// so it can be shared across threads without wrapping in `Arc<Mutex<_>>`.
    #[test]
    fn it_should_be_clone_send_and_sync() {
        fn assert_clone<T: Clone>() {}
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_clone::<Deployer>();
        assert_send::<Deployer>();
        assert_sync::<Deployer>();
    }
}
