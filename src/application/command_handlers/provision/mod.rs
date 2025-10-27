//! Provision Command Module
//!
//! This module implements the delivery-agnostic `ProvisionCommandHandler`
//! for orchestrating infrastructure provisioning business logic.
//!
//! ## Architecture
//!
//! The `ProvisionCommandHandler` implements the Command Pattern and uses Dependency Injection
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
//! ## Provisioning Workflow
//!
//! The command handler orchestrates a multi-step workflow:
//!
//! 1. **Render `OpenTofu` templates** - Generate infrastructure configuration
//! 2. **Initialize `OpenTofu`** - Set up Terraform/`OpenTofu` backend
//! 3. **Validate configuration** - Check syntax and consistency
//! 4. **Plan infrastructure** - Preview changes
//! 5. **Apply infrastructure** - Provision virtual machines
//! 6. **Get instance information** - Retrieve IP address and metadata
//! 7. **Render `Ansible` templates** - Generate configuration with runtime IP
//! 8. **Wait for SSH connectivity** - Ensure VM is reachable
//! 9. **Wait for cloud-init** - Ensure system is ready for configuration
//!
//! ## State Management
//!
//! The command handler integrates with the type-state pattern for environment lifecycle:
//!
//! - Accepts `Environment<Created>` as input
//! - Transitions to `Environment<Provisioning>` at start
//! - Returns `Environment<Provisioned>` on success
//! - Transitions to `Environment<ProvisionFailed>` on error
//!
//! State is persisted after each transition using the injected repository.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::ProvisionCommandHandlerError;
pub use handler::ProvisionCommandHandler;
