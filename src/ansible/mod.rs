//! Ansible integration for configuration management
//!
//! This module provides Ansible-specific functionality for the deployment system,
//! primarily focused on template rendering for Ansible playbooks and inventory files.
//!
//! The main component is `AnsibleTemplateRenderer` which handles the generation
//! of Ansible configuration files with dynamic content like VM IP addresses and SSH keys.

pub mod inventory_template_renderer;
pub mod template_renderer;

pub use inventory_template_renderer::InventoryTemplateRenderer;
pub use template_renderer::AnsibleTemplateRenderer;
