//! Virtual machine-specific E2E tasks
//!
//! This module contains tasks that are specifically designed for VM-based
//! E2E testing using LXD virtual machines. These tasks handle the unique
//! requirements and workflows when using `OpenTofu` and LXD for production-like
//! infrastructure provisioning.

pub mod cleanup_infrastructure;
pub mod preflight_cleanup;
pub mod run_provision_command;
