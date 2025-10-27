//! Destroy Command Module
//!
//! This module implements the delivery-agnostic `DestroyCommandHandler`
//! for orchestrating infrastructure destruction business logic.
//!
//! ## Architecture
//!
//! The `DestroyCommandHandler` implements the Command Pattern and uses Dependency Injection
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
//! - **Idempotent**: Can be safely executed multiple times on the same environment
//!
//! ## Destruction Workflow
//!
//! The command handler orchestrates a multi-step workflow:
//!
//! 1. **Load environment** - Retrieve environment from repository
//! 2. **Check current state** - Handle already-destroyed environments gracefully
//! 3. **Destroy infrastructure** - Remove VMs and resources via `OpenTofu` (if provisioned)
//! 4. **Clean up state files** - Remove data and build directories
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts environment in any state (via environment name lookup)
//! - Transitions to `Environment<Destroying>` at start
//! - Returns `Environment<Destroyed>` on success
//! - Transitions to `Environment<DestroyFailed>` on error
//!
//! State is persisted after each transition using the injected repository.
//!
//! ## Idempotency
//!
//! The destroy operation is idempotent - running it multiple times on the same
//! environment will succeed without errors, whether infrastructure was previously
//! provisioned or not.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::DestroyCommandHandlerError;
pub use handler::DestroyCommandHandler;
