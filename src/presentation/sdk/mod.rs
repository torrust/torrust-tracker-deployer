//! SDK Module — Programmatic API for the Torrust Tracker Deployer
//!
//! This module provides a typed Rust SDK for interacting with the deployer
//! programmatically. It is an alternative delivery mechanism to the CLI,
//! sitting in the Presentation Layer alongside the CLI controllers.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
//!
//! let deployer = Deployer::builder()
//!     .working_dir("/path/to/workspace")
//!     .build()
//!     .expect("Failed to initialize deployer");
//!
//! // List all environments
//! let environments = deployer.list().expect("Failed to list environments");
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Presentation Layer
//! ├── CLI delivery     (input/ → dispatch/ → controllers/ → views/)
//! └── SDK delivery     (sdk/)                                        ← this module
//!     │
//!     ▼
//! Application Layer    (command_handlers/)
//!     │
//!     ▼
//! Domain Layer         (environment/, template/, topology/, ...)
//! ```
//!
//! The [`Deployer`] facade delegates to application-layer command handlers,
//! which are the same handlers used by the CLI. The SDK simply provides a
//! more ergonomic entry point for programmatic consumers.
//!
//! ## Available Operations (Proof of Concept)
//!
//! | Method | Description |
//! |--------|-------------|
//! | [`Deployer::create_environment`] | Create a new deployment environment |
//! | [`Deployer::create_environment_from_file`] | Create an environment from a JSON file |
//! | [`Deployer::show`] | Show information about an environment |
//! | [`Deployer::exists`] | Check whether a named environment exists |
//! | [`Deployer::list`] | List all environments in the workspace |
//! | [`Deployer::validate`] | Validate an environment configuration file |
//! | [`Deployer::destroy`] | Destroy infrastructure for an environment |
//! | [`Deployer::purge`] | Remove all local data for an environment |

mod builder;
mod deployer;
mod error;

// === Core facade ===
pub use builder::{DeployerBuildError, DeployerBuilder};
pub use deployer::Deployer;

// === Domain types (inputs/outputs) ===
pub use crate::domain::environment::state::{
    Configured, Created, Destroyed, Provisioned, Released, Running,
};
pub use crate::domain::{
    AnyEnvironmentState, BackupConfig, Environment, EnvironmentName, EnvironmentNameError,
    HetznerConfig, InstanceName, LxdConfig, ProfileName, Provider, ProviderConfig,
};

// === Configuration types (for create_environment) ===
pub use crate::application::command_handlers::create::config::{
    EnvironmentCreationConfig, EnvironmentCreationConfigBuildError,
    EnvironmentCreationConfigBuilder,
};

// === Result types ===
pub use crate::application::command_handlers::list::EnvironmentList;
pub use crate::application::command_handlers::show::EnvironmentInfo;
pub use crate::application::command_handlers::validate::ValidationResult;

// === Error types ===
pub use crate::application::command_handlers::create::config::ConfigLoadError;
pub use crate::application::command_handlers::create::CreateCommandHandlerError;
pub use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
pub use crate::application::command_handlers::list::ListCommandHandlerError;
pub use crate::application::command_handlers::purge::errors::PurgeCommandHandlerError;
pub use crate::application::command_handlers::show::ShowCommandHandlerError;
pub use crate::application::command_handlers::validate::ValidateCommandHandlerError;
pub use error::{CreateEnvironmentFromFileError, SdkError};

// === Extension points ===
pub use crate::application::traits::{CommandProgressListener, NullProgressListener};
