//! Show Command Module
//!
//! This module implements the delivery-agnostic `ShowCommandHandler`
//! for displaying environment information and status.
//!
//! ## Architecture
//!
//! The `ShowCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Loads environment state via `EnvironmentRepository`
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Read-Only Operation**: Never modifies environment state
//! - **No Network Calls**: Displays stored data only (use `test` command for health checks)
//! - **State-Aware Display**: Shows information relevant to each environment state
//! - **Explicit Errors**: All errors implement helpful error messages with actionable guidance
//!
//! ## Information Displayed
//!
//! The command displays state-aware information:
//!
//! - **All states**: Environment name, current state, provider
//! - **Provisioned+**: Infrastructure details (IP, SSH credentials)
//! - **Running**: Service URLs and endpoints
//! - **All states**: Next step guidance based on current state
//!
//! ## State Handling
//!
//! All environment states are displayable, including:
//! - Created, Provisioned, Configured, Released, Running
//! - Failed states (`ProvisionFailed`, `ConfigureFailed`, etc.)
//! - Destroyed (shows historical information)

pub mod errors;
pub mod handler;
pub mod info;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ShowCommandHandlerError;
pub use handler::ShowCommandHandler;
pub use info::EnvironmentInfo;
pub use info::InfrastructureInfo;
pub use info::ServiceInfo;
