//! List Command Module
//!
//! This module implements the delivery-agnostic `ListCommandHandler`
//! for listing all environments in the deployment workspace.
//!
//! ## Architecture
//!
//! The `ListCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Scans data directory via `FileRepositoryFactory`
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Read-Only Operation**: Never modifies environment state
//! - **No Network Calls**: Scans local data directory only
//! - **Lightweight Loading**: Loads only summary data (name, state, provider, `created_at`)
//! - **Graceful Degradation**: Partial failures don't stop the entire listing
//! - **Explicit Errors**: All errors implement helpful error messages with actionable guidance
//!
//! ## Information Displayed
//!
//! The command displays a summary table with:
//!
//! - Environment name
//! - Current state (including Destroyed)
//! - Provider name
//! - Creation timestamp (ISO 8601)
//!
//! ## Error Handling Strategy
//!
//! - **Empty directory**: Not an error - shows friendly message, exit code 0
//! - **Fatal errors**: Permission denied, scan failure - exit code 1
//! - **Partial failure**: Shows valid environments + warnings, exit code 0

pub mod errors;
pub mod handler;
pub mod info;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ListCommandHandlerError;
pub use handler::ListCommandHandler;
pub use info::EnvironmentList;
pub use info::EnvironmentSummary;
