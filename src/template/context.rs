//! Template context types
//!
//! Defines the context structs used for template variable substitution.

use serde::Serialize;

/// Context for Ansible inventory template
#[derive(Serialize, Clone, Debug)]
pub struct AnsibleInventoryContext {
    pub ansible_host: String,
    pub ansible_ssh_private_key_file: String,
}

/// Context for static templates (no variables needed)
#[derive(Serialize, Clone, Debug)]
pub struct StaticContext {
    // Empty context for templates that don't need variables
    #[serde(skip)]
    _phantom: std::marker::PhantomData<()>,
}

impl Default for StaticContext {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
