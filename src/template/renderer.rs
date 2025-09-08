//! Template Renderer Trait
//!
//! Defines the core `TemplateRenderer` trait that all template wrappers must implement.

use anyhow::Result;
use std::path::Path;

/// Core trait for template rendering with strong typing and validation
pub trait TemplateRenderer {
    /// Renders the template with the given context to the output path
    ///
    /// # Errors
    /// Returns an error if template rendering fails or output file cannot be written
    fn render(&self, output_path: &Path) -> Result<()>;
}
