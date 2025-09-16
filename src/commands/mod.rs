//! High-level deployment commands (Level 1 of Three-Level Architecture)
//!
//! This module contains the top-level commands that orchestrate complete deployment workflows.
//! Commands combine multiple steps to provide cohesive functionality for different deployment
//! scenarios.
//!
//! ## Available Commands
//!
//! - `configure` - Infrastructure configuration and software installation
//! - `provision` - Infrastructure provisioning using `OpenTofu`
//! - `test` - Deployment testing and validation
//!
//! Each command follows the three-level architecture pattern by orchestrating steps (Level 2)
//! which in turn use remote actions (Level 3) for actual system interactions.

pub mod configure;
pub mod provision;
pub mod test;

pub use configure::ConfigureCommand;
pub use provision::ProvisionCommand;
pub use test::TestCommand;
