//! Error types for the SDK.
//!
//! - [`CreateEnvironmentFromFileError`] — for [`super::deployer::Deployer::create_environment_from_file`]
//! - [`SdkError`] — unified error enum covering all operations (planned for Phase 2 Task 6)

use thiserror::Error;

use crate::application::command_handlers::create::config::ConfigLoadError;
use crate::application::command_handlers::create::CreateCommandHandlerError;

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
