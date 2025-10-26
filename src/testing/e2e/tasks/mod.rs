//! E2E testing task modules
//!
//! This module contains all the high-level tasks that make up the end-to-end testing
//! workflow. Each task represents a significant phase in the deployment testing process
//! and can be executed independently or as part of a complete test sequence.
//!
//! ## Module Organization
//!
//! The tasks are organized by deployment target:
//!
//! ### Infrastructure-agnostic tasks (can be used with both containers and VMs):
//! - `run_create_command` - Environment creation using `CreateCommandHandler`
//! - `run_configure_command` - Infrastructure configuration via Ansible and playbook execution
//! - `run_configuration_validation` - Configuration validation and testing
//! - `run_test_command` - Deployment validation and testing
//!
//! ### Container-specific tasks (`container` submodule):
//! - `cleanup_infrastructure` - Docker container cleanup
//! - `preflight_cleanup` - Container-specific preflight cleanup
//! - `run_provision_simulation` - Provision simulation for container-based testing
//!
//! ### Virtual machine-specific tasks (`virtual_machine` submodule):
//! - `run_provision_command` - Infrastructure provisioning via `OpenTofu`
//! - `cleanup_infrastructure` - Infrastructure resource cleanup  
//! - `preflight_cleanup` - Shared directory cleanup functions and error types for both VM and container tests
//!
//! These tasks are orchestrated by the E2E test binaries to provide comprehensive
//! testing coverage of the entire deployment system.

pub mod container;
pub mod preflight_cleanup;
pub mod run_configuration_validation;
pub mod run_configure_command;
pub mod run_create_command;
pub mod run_test_command;
pub mod virtual_machine;
