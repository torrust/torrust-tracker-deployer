//! Application Layer (DDD)
//!
//! This module contains application-level services and orchestration logic.
//! The application layer coordinates domain operations and infrastructure
//! services to fulfill business use cases.
//!
//! ## Components
//!
//! - `commands` - High-level deployment commands implementing the Command pattern

pub mod commands;

// Re-export command types for convenience
pub use commands::{ConfigureCommand, ProvisionCommand, TestCommand};
