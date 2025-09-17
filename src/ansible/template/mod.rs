//! Ansible template functionality
//!
//! This module provides template-related functionality for Ansible,
//! including various template renderers for different file types.

pub mod renderer;

pub use renderer::{AnsibleTemplateRenderer, InventoryTemplateRenderer};
