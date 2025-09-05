//! Template wrapper for templates/ansible/install-docker.yml

use crate::template::context::TemplateContext;
use crate::template::{StaticContext, TemplateRenderer};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug)]
pub struct InstallDockerTemplate {
    #[allow(dead_code)]
    context: InstallDockerContext,
    content: String,
}

#[derive(Serialize, Debug)]
struct InstallDockerContext {
    // No template variables for now - this is a static template
}

impl TemplateContext for InstallDockerContext {
    // No required methods - Tera handles validation
}

impl InstallDockerTemplate {
    /// Creates a new `InstallDockerTemplate`, validating the template content and variable substitution
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Required variables are missing from the template
    /// - Template validation fails
    pub fn new(template_content: &str) -> Result<Self> {
        // Create context for static template
        let context = InstallDockerContext {};

        // Create a temporary engine with the template content
        let template_name = "install-docker.yml";
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

impl TemplateRenderer for InstallDockerTemplate {
    type Context = StaticContext;

    fn template_path(&self) -> &Path {
        // Since we're working with content instead of paths, return a dummy path
        // This should be refactored in the trait if this pattern is used more widely
        Path::new("install-docker.yml")
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
