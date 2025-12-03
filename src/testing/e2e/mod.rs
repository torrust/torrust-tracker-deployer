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
//! - `process_runner` - Black-box CLI process execution utilities
//! - `tasks` - High-level testing tasks and workflows
//!
//! ## Testing Workflow
//!
//! The E2E testing system orchestrates complete deployment scenarios including
//! provisioning, configuration, validation, and cleanup phases to ensure
//! the entire deployment system works correctly.
//!
//! ## Dependency Verification
//!
//! E2E test binaries use the `torrust-dependency-installer` package to verify
//! required system dependencies are installed before running tests.

pub mod container;
pub mod containers;
pub mod context;
mod process_runner;
pub mod tasks;

// Re-export for convenience
pub use container::Services;

// Re-export provisioned container types for backward compatibility
pub use containers::{ContainerError, RunningProvisionedContainer, StoppedProvisionedContainer};

// Re-export black-box testing types
pub use process_runner::{ProcessResult, ProcessRunner};

/// Subdirectory name for LXD `OpenTofu` configuration files within the build directory.
///
/// This constant defines the path where `OpenTofu`/Terraform configuration files
/// and state for the LXD provider will be managed: `build_dir/tofu/lxd/`.
///
/// Note: This is specific to LXD provider testing. Other providers (e.g., Hetzner)
/// would use their own paths like `tofu/hetzner`.
pub const LXD_OPENTOFU_SUBFOLDER: &str = "tofu/lxd";
