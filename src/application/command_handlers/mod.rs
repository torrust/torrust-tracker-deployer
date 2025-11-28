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
//! - `provision` - Infrastructure provisioning using `OpenTofu`
//! - `register` - Register existing instances as alternative to provisioning
//! - `test` - Deployment testing and validation
//!
//! Each command handler encapsulates a complete business workflow, handling orchestration,
//! error management, and coordination across multiple infrastructure services.

pub mod common;
pub mod configure;
pub mod create;
pub mod destroy;
pub mod provision;
pub mod register;
pub mod test;

pub use configure::ConfigureCommandHandler;
pub use create::CreateCommandHandler;
pub use destroy::DestroyCommandHandler;
pub use provision::ProvisionCommandHandler;
pub use register::RegisterCommandHandler;
pub use test::TestCommandHandler;
