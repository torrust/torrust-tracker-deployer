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
//! - `clean_and_prepare_templates` - Template cleanup and preparation
//! - `configure_infrastructure` - Infrastructure configuration via Ansible  
//! - `run_ansible_configuration` - Ansible playbook execution
//! - `run_deployment_validation` - Deployment validation and testing
//! - `setup_ssh_key` - SSH key generation and setup
//! - `validate_deployment` - Deployment validation and testing
//!
//! ### Container-specific tasks (`container` submodule):
//! - `cleanup_docker_container` - Docker container cleanup
//! - `configure_ssh_connectivity` - SSH connectivity setup for containers
//! - `provision_docker_infrastructure` - Docker container provisioning simulation
//! - `preflight_cleanup` - Container-specific preflight cleanup
//! - `run_provision_simulation` - Provision simulation for container-based testing
//! - `setup_docker_container` - Docker container setup and startup
//!
//! ### Virtual machine-specific tasks (`virtual_machine` submodule):
//! - `provision_infrastructure` - Infrastructure provisioning via `OpenTofu`
//! - `cleanup_infrastructure` - Infrastructure resource cleanup  
//! - `preflight_cleanup` - VM-specific preflight cleanup (`OpenTofu` + LXD)
//!
//! ### Common functionality:
//! - `preflight_cleanup_common` - Shared directory cleanup functions
//! - `preflight_cleanup` - Legacy module with common error types and functions
//!
//! These tasks are orchestrated by the E2E test binaries to provide comprehensive
//! testing coverage of the entire deployment system.

pub mod clean_and_prepare_templates;
pub mod configure_infrastructure;
pub mod container;
pub mod preflight_cleanup;
pub mod preflight_cleanup_common;
pub mod run_ansible_configuration;
pub mod run_deployment_validation;
pub mod setup_ssh_key;
pub mod validate_deployment;
pub mod virtual_machine;
