//! Error types for the SDK.
//!
//! - [`CreateEnvironmentFromFileError`] — for [`super::deployer::Deployer::create_environment_from_file`]
//! - [`SdkError`] — unified error enum covering all Deployer operations

use thiserror::Error;

use crate::application::command_handlers::create::config::ConfigLoadError;
use crate::application::command_handlers::create::CreateCommandHandlerError;
use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
use crate::application::command_handlers::list::ListCommandHandlerError;
use crate::application::command_handlers::purge::errors::PurgeCommandHandlerError;
use crate::application::command_handlers::show::ShowCommandHandlerError;
use crate::application::command_handlers::validate::ValidateCommandHandlerError;

use super::builder::DeployerBuildError;

/// Errors that can occur in [`super::deployer::Deployer::create_environment_from_file`].
///
/// Combines file loading failures with configuration creation failures.
#[derive(Debug, Error)]
pub enum CreateEnvironmentFromFileError {
    /// The configuration file could not be loaded or parsed.
    #[error("Failed to load environment configuration: {0}")]
    Load(#[from] ConfigLoadError),

    /// The environment could not be created from the loaded configuration.
    #[error("Failed to create environment: {0}")]
    Create(#[from] CreateCommandHandlerError),
}

/// Unified error type covering every [`super::deployer::Deployer`] operation.
///
/// Each variant corresponds to one operation (or the builder). Prefer the
/// individual per-operation error types when you only consume a single method;
/// use `SdkError` when you want a single `Result<_, SdkError>` across multiple
/// operations — for example in an AI-agent workflow function.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::presentation::sdk::{Deployer, SdkError};
/// use torrust_tracker_deployer_lib::domain::EnvironmentName;
///
/// fn ensure_env_exists(deployer: &Deployer, name: &EnvironmentName) -> Result<(), SdkError> {
///     if !deployer.exists(name)? {
///         let list = deployer.list()?;
///         println!("Known environments: {}", list.total_count);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Error)]
pub enum SdkError {
    /// [`super::builder::DeployerBuilder::build`] failed.
    #[error(transparent)]
    Build(#[from] DeployerBuildError),

    /// [`super::deployer::Deployer::create_environment`] failed.
    #[error(transparent)]
    Create(#[from] CreateCommandHandlerError),

    /// [`super::deployer::Deployer::create_environment_from_file`] failed.
    #[error(transparent)]
    CreateFromFile(#[from] CreateEnvironmentFromFileError),

    /// [`super::deployer::Deployer::show`] or [`super::deployer::Deployer::exists`] failed.
    #[error(transparent)]
    Show(#[from] ShowCommandHandlerError),

    /// [`super::deployer::Deployer::list`] failed.
    #[error(transparent)]
    List(#[from] ListCommandHandlerError),

    /// [`super::deployer::Deployer::validate`] failed.
    #[error(transparent)]
    Validate(#[from] ValidateCommandHandlerError),

    /// [`super::deployer::Deployer::destroy`] failed.
    #[error(transparent)]
    Destroy(#[from] DestroyCommandHandlerError),

    /// [`super::deployer::Deployer::purge`] failed.
    #[error(transparent)]
    Purge(#[from] PurgeCommandHandlerError),
}
