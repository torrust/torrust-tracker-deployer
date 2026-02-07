//! Application Command Handlers (Application Layer)
//!
//! This module contains high-level application command handlers that orchestrate complete
//! deployment workflows. These command handlers represent the primary use cases of the system
//! and coordinate between domain services and infrastructure adapters.
//!
//! Command handlers implement the Command Handler pattern and follow the three-level architecture:
//! Command Handlers (Level 1) → Steps (Level 2) → Remote Actions (Level 3)
//!
//! ## Available Command Handlers
//!
//! - `configure` - Infrastructure configuration and software installation
//! - `create` - Environment creation and initialization
//! - `destroy` - Infrastructure destruction and teardown
//! - `list` - List all environments in the workspace (read-only)
//! - `provision` - Infrastructure provisioning using `OpenTofu`
//! - `purge` - Remove all local environment data
//! - `register` - Register existing instances as alternative to provisioning
//! - `release` - Software release to target instances
//! - `run` - Stack execution on target instances
//! - `show` - Display environment information and status (read-only)
//! - `test` - Deployment testing and validation
//! - `validate` - Validate environment configuration files (read-only)
//!
//! Each command handler encapsulates a complete business workflow, handling orchestration,
//! error management, and coordination across multiple infrastructure services.

pub mod common;
pub mod configure;
pub mod create;
pub mod destroy;
pub mod list;
pub mod provision;
pub mod purge;
pub mod register;
pub mod release;
pub mod run;
pub mod show;
pub mod test;
pub mod validate;

pub use configure::ConfigureCommandHandler;
pub use create::CreateCommandHandler;
pub use destroy::DestroyCommandHandler;
pub use list::ListCommandHandler;
pub use provision::ProvisionCommandHandler;
pub use purge::handler::PurgeCommandHandler;
pub use register::RegisterCommandHandler;
pub use release::ReleaseCommandHandler;
pub use run::RunCommandHandler;
pub use show::ShowCommandHandler;
pub use test::TestCommandHandler;
pub use validate::ValidateCommandHandler;
