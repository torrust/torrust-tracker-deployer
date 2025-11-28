//! Application Layer (DDD)
//!
//! This module contains application-level services and orchestration logic.
//! The application layer coordinates domain operations and infrastructure
//! services to fulfill business use cases.
//!
//! ## Components
//!
//! - `command_handlers` - High-level deployment command handlers implementing the Command Handler pattern
//! - `services` - Shared application services used by multiple command handlers
//! - `steps` - Workflow orchestration and business process coordination

pub mod command_handlers;
pub mod services;
pub mod steps;

// Re-export command handler types for convenience
pub use command_handlers::{
    ConfigureCommandHandler, CreateCommandHandler, ProvisionCommandHandler, TestCommandHandler,
};
