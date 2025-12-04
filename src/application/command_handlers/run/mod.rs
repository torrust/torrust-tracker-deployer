//! Run Command Module
//!
//! This module implements the delivery-agnostic `RunCommandHandler`
//! for orchestrating the execution of the deployed software stack.
//!
//! ## Architecture
//!
//! The `RunCommandHandler` implements the Command Pattern and uses Dependency Injection
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
//! ## Run Workflow
//!
//! The command handler orchestrates a multi-step workflow:
//!
//! 1. **Load environment** - Retrieve environment from repository
//! 2. **Validate state** - Ensure environment is in a valid state for running
//! 3. **Start services** - Start the deployed software stack on the target instance
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts environment in `Released` state
//! - Transitions to `Environment<Running>` at start
//! - Returns `Environment<Running>` on success
//! - Transitions to `Environment<RunFailed>` on error
//!
//! State is persisted after each transition using the injected repository.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::RunCommandHandlerError;
pub use handler::RunCommandHandler;
