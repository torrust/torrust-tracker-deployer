//! Ansible integration for configuration management
//!
//! This module provides Ansible-specific functionality for the deployment system,
//! primarily focused on template rendering for Ansible playbooks and inventory files.
//!
//! The main component is `AnsibleTemplateRenderer` which handles the generation
//! of Ansible configuration files with dynamic content like VM IP addresses and SSH keys.
pub mod template;

pub use template::{AnsibleTemplateRenderer, InventoryTemplateRenderer};

/// Subdirectory name for Ansible-related files within the build directory.
///
/// Ansible playbooks, inventory files, and configuration templates
/// will be rendered to `build_dir/{ANSIBLE_SUBFOLDER}/`.
pub const ANSIBLE_SUBFOLDER: &str = "ansible";
