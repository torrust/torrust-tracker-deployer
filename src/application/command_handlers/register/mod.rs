//! Register Command Module
//!
//! This module implements the delivery-agnostic `RegisterCommandHandler`
//! for registering existing instances with environments.
//!
//! ## Architecture
//!
//! The `RegisterCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Persists environment state via `EnvironmentRepository`
//! - **Clock Abstraction**: Provides deterministic time for testing via `Clock` trait
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Asynchronous**: Uses async/await for network operations
//! - **Explicit State Transitions**: Type-safe state machine for environment lifecycle
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//!
//! ## Register Workflow
//!
//! The command handler orchestrates a simple workflow:
//!
//! 1. **Load environment** - Retrieve environment in Created state
//! 2. **Validate SSH connectivity** - Test connection to existing instance
//! 3. **Update runtime outputs** - Store the instance IP address
//! 4. **Transition state** - Move from Created to Provisioned
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts `Environment<Created>` as input
//! - Returns `Environment<Provisioned>` on success
//!
//! This is an alternative path to the `provision` command - both result in
//! `Environment<Provisioned>` state, but `register` uses existing infrastructure
//! instead of provisioning new infrastructure.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::RegisterCommandHandlerError;
pub use handler::RegisterCommandHandler;
