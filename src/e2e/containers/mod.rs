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
//! ## Re-exports
//!
//! For backward compatibility, this module re-exports the provisioned container
//! functionality at the top level:
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::e2e::containers::{
//!     StoppedProvisionedContainer, RunningProvisionedContainer, ProvisionedContainerError,
//!     DockerImageBuilder, ContainerConfigBuilder
//! };
//! ```

pub mod config_builder;
pub mod docker_builder;
pub mod provisioned;

// Re-export provisioned container types for backward compatibility
pub use provisioned::{
    ProvisionedContainerError, Result, RunningProvisionedContainer, StoppedProvisionedContainer,
};

// Re-export docker builder for public use
pub use docker_builder::{DockerBuildError, DockerImageBuilder};

// Re-export container config builder for public use
pub use config_builder::ContainerConfigBuilder;
