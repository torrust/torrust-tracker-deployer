//! Template wrapper for templates/ansible/ansible.cfg

use crate::template::context::TemplateContext;
use crate::template::{StaticContext, TemplateRenderer};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug)]
pub struct AnsibleCfgTemplate {
    #[allow(dead_code)]
    context: AnsibleCfgContext,
    content: String,
}

#[derive(Serialize, Debug)]
struct AnsibleCfgContext {
    // No template variables for now - this is a static template
}

impl TemplateContext for AnsibleCfgContext {
    fn required_variables(&self) -> Vec<&'static str> {
        // No required variables for static template
        vec![]
    }
}

impl AnsibleCfgTemplate {
    /// Creates a new `AnsibleCfgTemplate`, validating the template content and variable substitution
    ///
    /// # Errors
    /// Returns an error if:
    /// - Required variables are missing from the template
    /// - Template validation fails
    pub fn new(template_content: &str) -> Result<Self> {
        // Create context for static template
        let context = AnsibleCfgContext {};

        // Create a temporary engine with the template content
        let template_name = "ansible.cfg";
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

impl TemplateRenderer for AnsibleCfgTemplate {
    type Context = StaticContext;

    fn template_path(&self) -> &Path {
        // Since we're working with content instead of paths, return a dummy path
        // This should be refactored in the trait if this pattern is used more widely
        Path::new("ansible.cfg")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ansible_cfg_template() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("ansible.cfg");
        let output_file = temp_dir.path().join("output.cfg");

        let template_content = "[defaults]\nhost_key_checking = False";
        fs::write(&template_file, template_content)?;

        let template = AnsibleCfgTemplate::new(template_content)?;
        let ctx = StaticContext::default();

        template.render(&ctx, &output_file)?;

        let content = fs::read_to_string(&output_file)?;
        assert_eq!(content, "[defaults]\nhost_key_checking = False");

        Ok(())
    }
}
