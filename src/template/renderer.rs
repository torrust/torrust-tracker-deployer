//! Template Renderer Trait
//!
//! Defines the core `TemplateRenderer` trait that all template wrappers must implement.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

/// Core trait for template rendering with strong typing and validation
pub trait TemplateRenderer {
    /// Context type that provides template variables
    type Context: Serialize;

    /// Returns the path to the template file
    fn template_path(&self) -> &Path;

    /// Returns list of required template variables
    fn required_variables(&self) -> Vec<&'static str>;

    /// Renders the template with the given context to the output path
    ///
    /// # Errors
    /// Returns an error if template rendering fails or output file cannot be written
    fn render(&self, context: &Self::Context, output_path: &Path) -> Result<()>;

    /// Validates that the context contains all required variables
    ///
    /// # Errors
    /// Returns an error if any required variables are missing from the context
    fn validate_context(&self, context: &Self::Context) -> Result<()>;
}
