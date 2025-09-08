//! Template wrapper for templates/ansible/inventory.yml
//!
//! This template has mandatory variables that must be provided at construction time.

use crate::template::{StaticContext, TemplateRenderer};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct InventoryTemplate {
    context: InventoryContext,
    content: String,
}

#[derive(Serialize, Debug)]
struct InventoryContext {
    ansible_host: String,
    ansible_ssh_private_key_file: String,
}

impl InventoryTemplate {
    /// Creates a new `InventoryTemplate`, validating the template content and variable substitution
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template syntax is invalid
    /// - Required variables cannot be substituted
    /// - Template validation fails
    pub fn new(
        template_content: &str,
        ansible_host: &str,
        ansible_ssh_private_key_file: &str,
    ) -> Result<Self> {
        let context = InventoryContext {
            ansible_host: ansible_host.to_string(),
            ansible_ssh_private_key_file: ansible_ssh_private_key_file.to_string(),
        };

        // Create template engine and validate rendering
        let template_name = "inventory.yml";

        let engine =
            crate::template::TemplateEngine::with_template_content(template_name, template_content)
                .with_context(|| "Failed to create template engine with content")?;

        // This will fail if:
        // - Template has syntax errors
        // - Template references undefined variables
        // - Template cannot be rendered for any reason
        let validated_content = engine
            .validate_template_substitution_by_name(template_name, &context)
            .with_context(|| "Template validation failed during construction")?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }

    /// Get the ansible host value
    #[must_use]
    pub fn ansible_host(&self) -> &str {
        &self.context.ansible_host
    }

    /// Get the ansible SSH private key file path
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> &str {
        &self.context.ansible_ssh_private_key_file
    }
}

impl TemplateRenderer for InventoryTemplate {
    type Context = StaticContext; // We don't use external context since we have fields

    fn template_path(&self) -> &Path {
        // Since we're working with content instead of paths, return a dummy path
        // This should be refactored in the trait if this pattern is used more widely
        Path::new("inventory.yml")
    }

    fn render(&self, _context: &Self::Context, output_path: &Path) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        // Write the pre-validated content directly
        fs::write(output_path, &self.content).with_context(|| {
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

    #[test]
    fn test_inventory_template_creation() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template =
            InventoryTemplate::new(template_content, "192.168.1.100", "/path/to/key").unwrap();

        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(template.ansible_ssh_private_key_file(), "/path/to/key");
    }

    #[test]
    fn test_inventory_template_context_generation() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template =
            InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();

        assert_eq!(template.ansible_host(), "10.0.0.1");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
    }

    #[test]
    fn test_empty_template_content() {
        // Test with empty template content
        let result = InventoryTemplate::new("", "10.0.0.1", "/home/user/.ssh/id_rsa");

        // Empty templates are valid in Tera - they just render as empty strings
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.content, "");
    }

    #[test]
    fn test_missing_placeholder() {
        // Create template content with only one placeholder
        let template_content = "[all]\nserver ansible_host={{ansible_host}}\n";

        let result = InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa");

        // This is valid - templates don't need to use all available context variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=10.0.0.1"));
    }

    #[test]
    fn test_early_error_detection_both_variables_missing() {
        // Create template content with no placeholder variables at all
        let template_content = "[all]\nserver ansible_host=192.168.1.1\n";

        let result = InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa");

        // Static templates are valid - they just don't use template variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=192.168.1.1"));
    }

    #[test]
    fn test_undefined_variable_error() {
        // Create template content that references an undefined variable
        let template_content = "[all]\nserver ansible_host={{undefined_variable}}\n";

        let result = InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa");

        // This should fail because the template references an undefined variable
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Template validation failed during construction"));
    }

    #[test]
    fn test_early_error_detection_template_validation_fails() {
        // Create template content with malformed Tera syntax
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\nmalformed={{unclosed_var\n";

        let result = InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa");

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn test_early_error_detection_malformed_syntax() {
        // Test with different malformed template syntax
        let template_content = "invalid {{{{ syntax";

        let result = InventoryTemplate::new(template_content, "10.0.0.1", "/home/user/.ssh/id_rsa");

        assert!(result.is_err());
    }

    #[test]
    fn test_template_validation_at_construction() {
        // Create valid template content
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        // Template validation happens during construction, not during render
        let template = InventoryTemplate::new(
            template_content,
            "192.168.1.100",
            "/home/user/.ssh/test_key",
        )
        .unwrap();

        // Verify that the template was pre-validated and contains rendered content
        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/test_key"
        );
    }
}
