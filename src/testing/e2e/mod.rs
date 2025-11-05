//! End-to-End testing infrastructure
//!
//! This module provides comprehensive testing infrastructure for validating
//! the complete deployment workflow from infrastructure provisioning through
//! application deployment and validation.
//!
//! ## Module Structure
//!
//! - `container` - Service dependency injection container for E2E tests
//! - `context` - Test context configuration and management
//! - `containers` - Container management for E2E testing scenarios
//! - `dependencies` - Dependency verification for E2E test binaries
//! - `tasks` - High-level testing tasks and workflows
//!
//! ## Testing Workflow
//!
//! The E2E testing system orchestrates complete deployment scenarios including
//! provisioning, configuration, validation, and cleanup phases to ensure
//! the entire deployment system works correctly.

pub mod container;
pub mod containers;
pub mod context;
pub mod dependencies;
pub mod tasks;

// Re-export for convenience
pub use container::Services;

// Re-export provisioned container types for backward compatibility
pub use containers::{ContainerError, RunningProvisionedContainer, StoppedProvisionedContainer};
