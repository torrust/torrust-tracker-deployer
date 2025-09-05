//! Template wrapper for templates/tofu/lxd/main.tf

use crate::template::context::TemplateContext;
use crate::template::{StaticContext, TemplateRenderer};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug)]
pub struct MainTfTemplate {
    #[allow(dead_code)]
    context: MainTfContext,
    content: String,
}

#[derive(Serialize, Debug)]
struct MainTfContext {
    // No template variables for now - this is a static template
}

impl TemplateContext for MainTfContext {
    fn required_variables(&self) -> Vec<&'static str> {
        // No required variables for static template
        vec![]
    }
}

impl MainTfTemplate {
    /// Creates a new `MainTfTemplate`, validating the template content and variable substitution
    ///
    /// # Errors
    /// Returns an error if:
    /// - Required variables are missing from the template
    /// - Template validation fails
    pub fn new(template_content: &str) -> Result<Self> {
        // Create context for static template
        let context = MainTfContext {};

        // Create a temporary engine with the template content
        let template_name = "main.tf";
        let engine =
            crate::template::TemplateEngine::with_template_content(template_name, template_content)
                .with_context(|| "Failed to create template engine with content")?;

        let validated_content = engine
            .validate_template_substitution_by_name(template_name, &context)
            .with_context(|| "Template validation failed during construction")?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }
}

impl TemplateRenderer for MainTfTemplate {
    type Context = StaticContext;

    fn template_path(&self) -> &Path {
        // Since we're working with content instead of paths, return a dummy path
        // This should be refactored in the trait if this pattern is used more widely
        Path::new("main.tf")
    }

    fn render(&self, _context: &Self::Context, output_path: &Path) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        // Write the pre-validated content directly
        std::fs::write(output_path, &self.content).with_context(|| {
            format!("Failed to write template output: {}", output_path.display())
        })?;

        Ok(())
    }

    fn validate_context(&self, _context: &Self::Context) -> Result<()> {
        // Validation is built-in since fields are mandatory at construction
        Ok(())
    }
}
