//! Configure Command Module
//!
//! This module implements the delivery-agnostic `ConfigureCommandHandler`
//! for orchestrating infrastructure configuration business logic.
//!
//! ## Architecture
//!
//! The `ConfigureCommandHandler` implements the Command Pattern and uses Dependency Injection
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
//! ## Configuration Workflow
//!
//! The command handler orchestrates a multi-step workflow:
//!
//! 1. **Install Docker** - Install Docker engine on the provisioned VM
//! 2. **Install Docker Compose** - Install Docker Compose for container orchestration
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts `Environment<Provisioned>` as input
//! - Transitions to `Environment<Configuring>` at start
//! - Returns `Environment<Configured>` on success
//! - Transitions to `Environment<ConfigureFailed>` on error
//!
//! State is persisted after each transition using the injected repository.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ConfigureCommandHandlerError;
pub use handler::ConfigureCommandHandler;
