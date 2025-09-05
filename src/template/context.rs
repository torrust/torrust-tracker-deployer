//! Template context types
//!
//! Defines the context structs used for template variable substitution.

use serde::Serialize;

pub trait TemplateContext {
    /// Returns list of required template variables
    fn required_variables(&self) -> Vec<&'static str>;
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
