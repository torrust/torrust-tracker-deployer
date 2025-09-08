//! Template wrapper implementations organized by directory structure
//!
//! This module contains wrappers only for template files that actually need variable substitution
//! and have the `.tera` extension. Static config files are copied directly without template processing.
//!
//! ## Ansible Templates (templates/ansible/)
//! - `ansible::inventory` - templates/ansible/inventory.yml.tera (with runtime variables: `ansible_host`, `ssh_key`)

pub mod ansible;
pub mod tofu;
