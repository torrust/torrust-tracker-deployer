//! Application Commands (Application Layer)
//!
//! This module contains high-level application commands that orchestrate complete
//! deployment workflows. These commands represent the primary use cases of the system
//! and coordinate between domain services and infrastructure adapters.
//!
//! Commands implement the Command pattern and follow the three-level architecture:
//! Commands (Level 1) → Steps (Level 2) → Remote Actions (Level 3)
//!
//! ## Available Commands
//!
//! - `configure` - Infrastructure configuration and software installation
//! - `destroy` - Infrastructure destruction and teardown
//! - `provision` - Infrastructure provisioning using `OpenTofu`
//! - `test` - Deployment testing and validation
//!
//! Each command encapsulates a complete business workflow, handling orchestration,
//! error management, and coordination across multiple infrastructure services.

pub mod configure;
pub mod destroy;
pub mod provision;
pub mod test;

pub use configure::ConfigureCommand;
pub use destroy::DestroyCommand;
pub use provision::ProvisionCommand;
pub use test::TestCommand;
