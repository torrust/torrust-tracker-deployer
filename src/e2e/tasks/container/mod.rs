//! Container-specific E2E tasks
//!
//! This module contains tasks that are specifically designed for container-based
//! E2E testing using Docker containers instead of VMs. These tasks handle the
//! unique requirements and workflows when using testcontainers for testing
//! infrastructure deployment.
//!
//! ## Tasks Overview
//!
//! - `cleanup_docker_container` - Stops and cleans up Docker containers after testing
//! - `configure_ssh_connectivity` - Sets up SSH connectivity specifically for containers
//! - `preflight_cleanup` - Container-specific preflight cleanup operations
//! - `provision_docker_infrastructure` - Simulates infrastructure provisioning for containers
//! - `run_provision_simulation` - Simulates provision phase for container-based testing
//! - `setup_docker_container` - Creates and starts Docker containers for testing

pub mod cleanup_docker_container;
pub mod configure_ssh_connectivity;
pub mod preflight_cleanup;
pub mod provision_docker_infrastructure;
pub mod run_provision_simulation;
pub mod setup_docker_container;
