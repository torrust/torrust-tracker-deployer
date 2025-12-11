//! Template wrapper for templates/ansible/inventory.yml
//!
//! This template has mandatory variables that must be provided at construction time.

pub mod context;
pub mod template;

pub use context::{
    AnsibleHost, AnsibleHostError, AnsiblePort, AnsiblePortError, InventoryContext,
    InventoryContextBuilder, InventoryContextError,
};
pub use context::{SshPrivateKeyFile, SshPrivateKeyFileError};
pub use template::InventoryTemplate;
