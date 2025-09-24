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
//! ## Re-exports
//!
//! For backward compatibility, this module re-exports the provisioned container
//! functionality at the top level:
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::e2e::containers::{
//!     StoppedProvisionedContainer, RunningProvisionedContainer, ProvisionedContainerError
//! };
//! ```

pub mod provisioned;

// Re-export provisioned container types for backward compatibility
pub use provisioned::{
    ProvisionedContainerError, Result, RunningProvisionedContainer, StoppedProvisionedContainer,
};
