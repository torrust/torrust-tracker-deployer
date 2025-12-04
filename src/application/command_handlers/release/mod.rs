//! Release Command Module
//!
//! This module implements the delivery-agnostic `ReleaseCommandHandler`
//! for orchestrating software release operations on target instances.
//!
//! ## Architecture
//!
//! The `ReleaseCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Persists environment state via `EnvironmentRepository`
//! - **Clock Abstraction**: Provides deterministic time for testing via `Clock` trait
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Synchronous**: Follows existing patterns (no async/await)
//! - **Explicit State Transitions**: Type-safe state machine for environment lifecycle
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//!
//! ## Release Workflow
//!
//! The command handler orchestrates a multi-step workflow:
//!
//! 1. **Load environment** - Retrieve environment from repository
//! 2. **Validate state** - Ensure environment is in a valid state for release
//! 3. **Release software** - Deploy software to the target instance
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts environment in `Configured` state
//! - Transitions to `Environment<Releasing>` at start
//! - Returns `Environment<Released>` on success
//! - Transitions to `Environment<ReleaseFailed>` on error
//!
//! State is persisted after each transition using the injected repository.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ReleaseCommandHandlerError;
pub use handler::ReleaseCommandHandler;
