//! Torrust Tracker Deployer SDK
//!
//! Programmatic API for deploying Torrust Tracker instances.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_sdk::Deployer;
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
//! torrust-tracker-deployer-sdk         ← this package
//!     │
//!     ▼
//! torrust-tracker-deployer (root crate)
//!     │
//!     ▼
//! Application Layer    (command_handlers/)
//!     │
//!     ▼
//! Domain Layer         (environment/, template/, topology/, ...)
//! ```

mod builder;
mod deployer;
mod error;

// === Core facade ===
pub use builder::{DeployerBuildError, DeployerBuilder};
pub use deployer::Deployer;

// === Domain types (inputs only) ===
pub use torrust_deployer_types::{EnvironmentName, EnvironmentNameError};

// === Configuration types (for create_environment) ===
pub use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    EnvironmentCreationConfig, EnvironmentCreationConfigBuildError,
    EnvironmentCreationConfigBuilder,
};

// === Result types ===
pub use torrust_tracker_deployer_lib::application::command_handlers::list::EnvironmentList;
pub use torrust_tracker_deployer_lib::application::command_handlers::show::EnvironmentInfo;
pub use torrust_tracker_deployer_lib::application::command_handlers::test::TestResult;
pub use torrust_tracker_deployer_lib::application::command_handlers::validate::ValidationResult;

// === Error types ===
pub use error::{CreateEnvironmentFromFileError, SdkError};
pub use torrust_tracker_deployer_lib::application::command_handlers::configure::ConfigureCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::create::config::ConfigLoadError;
pub use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::destroy::DestroyCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::list::ListCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::provision::ProvisionCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::purge::errors::PurgeCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::release::ReleaseCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::run::RunCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::show::ShowCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::test::TestCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::command_handlers::validate::ValidateCommandHandlerError;
pub use torrust_tracker_deployer_lib::application::errors::{
    InvalidStateError, PersistenceError, ReleaseWorkflowStep,
};

// === Extension points ===
pub use torrust_tracker_deployer_lib::application::traits::{
    CommandProgressListener, NullProgressListener,
};
