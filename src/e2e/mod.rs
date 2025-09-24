//! End-to-End testing infrastructure
//!
//! This module provides comprehensive testing infrastructure for validating
//! the complete deployment workflow from infrastructure provisioning through
//! application deployment and validation.
//!
//! ## Module Structure
//!
//! - `environment` - Test environment configuration and management
//! - `containers` - Container management for E2E testing scenarios
//! - `tasks` - High-level testing tasks and workflows
//!
//! ## Testing Workflow
//!
//! The E2E testing system orchestrates complete deployment scenarios including
//! provisioning, configuration, validation, and cleanup phases to ensure
//! the entire deployment system works correctly.

pub mod containers;
pub mod environment;
pub mod tasks;

// Re-export provisioned container types for backward compatibility
pub use containers::{ContainerError, RunningProvisionedContainer, StoppedProvisionedContainer};
