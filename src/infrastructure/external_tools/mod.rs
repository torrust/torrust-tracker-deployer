//! External Tools Integration
//!
//! This module contains integration adapters and delivery mechanisms for external
//! third-party console tools used in the deployment infrastructure.
//!
//! Each tool module contains all related functionality for that specific tool:
//! - Adapter (command-line wrapper)
//! - Template renderers and wrappers (for tools that use templates)
//!
//! ## Components
//!
//! - `ansible` - Ansible configuration management tool
//!   - `adapter` - Ansible command wrapper
//!   - `template` - Template renderers and context wrappers
//! - `lxd` - LXD container/VM management tool
//!   - `adapter` - LXD command wrapper
//! - `tofu` - `OpenTofu` infrastructure provisioning tool
//!   - `adapter` - `OpenTofu` command wrapper
//!   - `template` - Template renderers and context wrappers

pub mod ansible;
pub mod lxd;
pub mod tofu;
