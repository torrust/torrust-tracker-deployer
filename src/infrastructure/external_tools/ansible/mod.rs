//! Ansible integration for configuration management
//!
//! This module provides Ansible-specific functionality for the deployment system,
//! including template rendering for Ansible configuration files.
//!
//! ## Components
//!
//! - `template` - Template renderers and context wrappers for Ansible configuration files
//!
//! Note: The Ansible adapter (`AnsibleClient`) has been moved to `crate::adapters::ansible`

pub mod template;

pub use template::{AnsibleTemplateRenderer, InventoryTemplateRenderer};

/// Subdirectory name for Ansible-related files within the build directory.
///
/// Ansible playbooks, inventory files, and configuration templates
/// will be rendered to `build_dir/{ANSIBLE_SUBFOLDER}/`.
pub const ANSIBLE_SUBFOLDER: &str = "ansible";
