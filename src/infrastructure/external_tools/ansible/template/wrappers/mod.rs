//! Ansible template wrappers
//!
//! Contains wrappers only for template files that actually need variable substitution
//! and have the `.tera` extension. Static playbooks and config files are copied directly.
pub mod firewall_playbook;
pub mod inventory;
pub mod variables;

// Re-export the main template structs for easier access
pub use firewall_playbook::FirewallPlaybookTemplate;
pub use inventory::InventoryTemplate;
pub use variables::AnsibleVariablesTemplate;
