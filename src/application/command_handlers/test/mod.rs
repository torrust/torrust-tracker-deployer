//! Test Command Module
//!
//! This module implements the delivery-agnostic `TestCommandHandler`
//! for orchestrating infrastructure validation business logic.
//!
//! ## Architecture
//!
//! The `TestCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Loads environment state via `EnvironmentRepository`
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Asynchronous**: Uses async/await for network operations
//! - **Runtime State Validation**: Accepts any environment state, validates at runtime
//! - **Explicit Errors**: All errors implement helpful error messages with actionable guidance
//!
//! ## Validation Workflow
//!
//! The command handler orchestrates a multi-step validation workflow:
//!
//! 1. **Validate cloud-init completion** - Ensure system initialization is complete
//! 2. **Validate Docker installation** - Verify Docker is installed and running
//! 3. **Validate Docker Compose installation** - Verify Docker Compose is available
//!
//! ## State Management
//!
//! Unlike `provision` and `configure` handlers, the test handler does not transition
//! environment state. It accepts an environment name, loads the environment from storage,
//! and performs runtime validation checks regardless of the environment's compile-time state.

pub mod errors;
pub mod handler;
pub mod result;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::TestCommandHandlerError;
pub use handler::TestCommandHandler;
pub use result::TestResult;
