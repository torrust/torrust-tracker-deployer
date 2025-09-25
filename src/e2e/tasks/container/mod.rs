//! Container-specific E2E tasks
//!
//! This module contains tasks that are specifically designed for container-based
//! E2E testing using Docker containers instead of VMs. These tasks handle the
//! unique requirements and workflows when using testcontainers for testing
//! infrastructure deployment.

pub mod preflight_cleanup;
pub mod provision_docker_infrastructure;
