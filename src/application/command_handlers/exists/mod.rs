//! Exists Command Module
//!
//! This module implements the delivery-agnostic `ExistsCommandHandler`
//! for checking whether an environment exists.
//!
//! ## Architecture
//!
//! The `ExistsCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Checks environment existence via `EnvironmentRepository`
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Read-Only Operation**: Never modifies environment state
//! - **No Network Calls**: Checks local data only
//! - **Not-Found is a Result**: "Environment not found" is `exists = false`, NOT an error
//! - **Explicit Errors**: Only repository access failures produce errors

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ExistsCommandHandlerError;
pub use handler::ExistsCommandHandler;
pub use handler::ExistsResult;
