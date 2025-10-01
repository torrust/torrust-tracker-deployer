//! Ansible integration for configuration management
//!
//! This module provides Ansible-specific functionality for the deployment system,
//! including command-line wrapper and template rendering.
//!
//! ## Components
//!
//! - `adapter` - Ansible command-line tool wrapper (`AnsibleClient`)
//! - `template` - Template renderers and context wrappers for Ansible configuration files

pub mod adapter;
pub mod template;

pub use adapter::AnsibleClient;
pub use template::{AnsibleTemplateRenderer, InventoryTemplateRenderer};

/// Subdirectory name for Ansible-related files within the build directory.
///
/// Ansible playbooks, inventory files, and configuration templates
/// will be rendered to `build_dir/{ANSIBLE_SUBFOLDER}/`.
pub const ANSIBLE_SUBFOLDER: &str = "ansible";
