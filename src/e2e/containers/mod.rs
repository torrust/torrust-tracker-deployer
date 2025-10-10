//! Container management for E2E testing
//!
//! This module provides abstractions for managing different types of containers
//! used in end-to-end testing scenarios.
//!
//! ## Available Container Types
//!
//! - **Provisioned Containers** - Docker containers that simulate provisioned instances
//!   in the deployment workflow, providing SSH access and basic system functionality.
//!
//! ## Container Collaborators
//!
//! - **Docker Image Builder** - Configurable builder for Docker images used in testing
//! - **Container Config Builder** - Flexible builder for container configurations
//!
//! ## Container Actions
//!
//! - **Container Actions** - Decoupled operations that can be performed on containers
//!   (SSH setup, connectivity checks, etc.)
//!
//! ## Re-exports
//!
//! For backward compatibility, this module re-exports the provisioned container
//! functionality at the top level:
//!
//! ```rust,no_run
//! use torrust_tracker_deployer::e2e::containers::{
//!     StoppedProvisionedContainer, RunningProvisionedContainer, ContainerError,
//!     ContainerImageBuilder, ContainerConfigBuilder
//! };
//! ```

pub mod actions;
pub mod config_builder;
pub mod errors;
pub mod executor;
pub mod image_builder;
pub mod provisioned;
pub mod timeout;

// Re-export provisioned container types for backward compatibility
pub use provisioned::{RunningProvisionedContainer, StoppedProvisionedContainer};

// Re-export error types for public use
pub use errors::{ContainerError, Result};

// Re-export timeout types for public use
pub use timeout::ContainerTimeouts;

// Re-export docker builder for public use
pub use image_builder::{ContainerBuildError, ContainerImageBuilder};

// Re-export container config builder for public use
pub use config_builder::ContainerConfigBuilder;

// Re-export executor trait for container actions
pub use executor::ContainerExecutor;
