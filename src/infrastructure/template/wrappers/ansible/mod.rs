//! Ansible template wrappers
//!
//! Contains wrappers only for template files that actually need variable substitution
//! and have the `.tera` extension. Static playbooks and config files are copied directly.
pub mod inventory;

// Re-export the main template structs for easier access
pub use inventory::InventoryTemplate;
