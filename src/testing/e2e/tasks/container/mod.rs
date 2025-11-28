//! Container-specific E2E tasks
//!
//! This module contains tasks that are specifically designed for container-based
//! E2E testing using Docker containers instead of VMs. These tasks handle the
//! unique requirements and workflows when using testcontainers for testing
//! infrastructure deployment.
//!
//! ## Tasks Overview
//!
//! - `cleanup_infrastructure` - Stops and cleans up Docker containers after testing
//! - `preflight_cleanup` - Container-specific preflight cleanup operations

pub mod cleanup_infrastructure;
pub mod preflight_cleanup;
