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
//! ## Available Operations
//!
//! | Method | Async | Description |
//! |--------|-------|-------------|
//! | [`Deployer::create_environment`] | No | Create a new deployment environment |
//! | [`Deployer::create_environment_from_file`] | No | Create an environment from a JSON file |
//! | [`Deployer::show`] | No | Show information about an environment |
//! | [`Deployer::exists`] | No | Check whether a named environment exists |
//! | [`Deployer::list`] | No | List all environments in the workspace |
//! | [`Deployer::validate`] | No | Validate an environment configuration file |
//! | [`Deployer::destroy`] | No | Destroy infrastructure for an environment |
//! | [`Deployer::purge`] | No | Remove all local data for an environment |
//! | [`Deployer::provision`] | Yes | Provision infrastructure (OpenTofu + SSH) |
//! | [`Deployer::configure`] | No | Configure a provisioned environment (Ansible) |
//! | [`Deployer::release`] | Yes | Deploy software to a configured environment |
//! | [`Deployer::run_services`] | No | Start services on a released environment |
//! | [`Deployer::test`] | Yes | Test a deployed environment |

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
pub use crate::application::command_handlers::test::TestResult;
pub use crate::application::command_handlers::validate::ValidationResult;

// === Error types ===
pub use crate::application::command_handlers::configure::ConfigureCommandHandlerError;
pub use crate::application::command_handlers::create::config::ConfigLoadError;
pub use crate::application::command_handlers::create::CreateCommandHandlerError;
pub use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
pub use crate::application::command_handlers::list::ListCommandHandlerError;
pub use crate::application::command_handlers::provision::ProvisionCommandHandlerError;
pub use crate::application::command_handlers::purge::errors::PurgeCommandHandlerError;
pub use crate::application::command_handlers::release::ReleaseCommandHandlerError;
pub use crate::application::command_handlers::run::RunCommandHandlerError;
pub use crate::application::command_handlers::show::ShowCommandHandlerError;
pub use crate::application::command_handlers::test::TestCommandHandlerError;
pub use crate::application::command_handlers::validate::ValidateCommandHandlerError;
pub use error::{CreateEnvironmentFromFileError, SdkError};

// === Extension points ===
pub use crate::application::traits::{CommandProgressListener, NullProgressListener};
