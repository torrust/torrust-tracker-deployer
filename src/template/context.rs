//! Template context types
//!
//! Defines the context structs used for template variable substitution.

use serde::Serialize;

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
