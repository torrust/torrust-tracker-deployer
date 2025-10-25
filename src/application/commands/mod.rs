//! Commands Module
//!
//! This module contains delivery-agnostic commands that implement business logic
//! using the Command Pattern. Commands are synchronous and can be used from any
//! delivery mechanism (CLI, REST API, GraphQL, etc.).
//!
//! ## Commands
//!
//! - `create` - Command for creating new deployment environments

pub mod create;

// Re-export command types for convenience
pub use create::{CreateCommand, CreateCommandError};
