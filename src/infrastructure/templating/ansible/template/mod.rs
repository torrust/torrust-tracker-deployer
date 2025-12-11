//! Ansible template functionality
//!
//! This module provides template-related functionality for Ansible,
//! including various template renderers for different file types and
//! template wrappers for type-safe context management.

pub mod renderer;
pub mod wrappers;

pub use renderer::{AnsibleProjectGenerator, InventoryRenderer};
