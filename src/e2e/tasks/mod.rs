//! E2E testing task modules
//!
//! This module contains all the high-level tasks that make up the end-to-end testing
//! workflow. Each task represents a significant phase in the deployment testing process
//! and can be executed independently or as part of a complete test sequence.
//!
//! ## Available Tasks
//!
//! - `clean_and_prepare_templates` - Template cleanup and preparation
//! - `cleanup_infrastructure` - Infrastructure resource cleanup
//! - `configure_infrastructure` - Infrastructure configuration via Ansible
//! - `preflight_cleanup` - Pre-test cleanup of lingering resources
//! - `provision_infrastructure` - Infrastructure provisioning via `OpenTofu`
//! - `setup_ssh_key` - SSH key generation and setup
//! - `validate_deployment` - Deployment validation and testing
//!
//! These tasks are orchestrated by the E2E test binary to provide comprehensive
//! testing coverage of the entire deployment system.

pub mod clean_and_prepare_templates;
pub mod cleanup_infrastructure;
pub mod configure_infrastructure;
pub mod preflight_cleanup;
pub mod provision_infrastructure;
pub mod setup_ssh_key;
pub mod validate_deployment;
