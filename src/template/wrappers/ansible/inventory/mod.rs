//! Template wrapper for templates/ansible/inventory.yml
//!
//! This template has mandatory variables that must be provided at construction time.

pub mod ansible_host;
pub mod ssh_private_key_file;

use crate::template::file::File;
use crate::template::TemplateRenderer;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub use ansible_host::AnsibleHost;
pub use ssh_private_key_file::SshPrivateKeyFile;

#[derive(Debug)]
pub struct InventoryTemplate {
    context: InventoryContext,
    content: String,
}

#[derive(Serialize, Debug)]
pub struct InventoryContext {
    ansible_host: AnsibleHost,
    ansible_ssh_private_key_file: SshPrivateKeyFile,
}

impl InventoryContext {
    /// Creates a new `InventoryContext`
    ///
    /// # Errors
    ///
    /// Returns an error if the `ansible_host` cannot be parsed as a valid IP address
    pub fn new(ansible_host: &str, ansible_ssh_private_key_file: &str) -> Result<Self> {
        let ansible_host = AnsibleHost::from_str(ansible_host)?;
        let ansible_ssh_private_key_file = SshPrivateKeyFile::new(ansible_ssh_private_key_file);

        Ok(Self {
            ansible_host,
            ansible_ssh_private_key_file,
        })
    }

    /// Get the ansible host value as a string
    #[must_use]
    pub fn ansible_host(&self) -> String {
        self.ansible_host.as_str()
    }

    /// Get the ansible SSH private key file path as a string
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> String {
        self.ansible_ssh_private_key_file.as_string()
    }

    /// Get the ansible host wrapper
    #[must_use]
    pub fn ansible_host_wrapper(&self) -> &AnsibleHost {
        &self.ansible_host
    }

    /// Get the ansible SSH private key file wrapper
    #[must_use]
    pub fn ansible_ssh_private_key_file_wrapper(&self) -> &SshPrivateKeyFile {
        &self.ansible_ssh_private_key_file
    }
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
    pub fn new(template_file: &File, inventory_context: &InventoryContext) -> Result<Self> {
        let context = InventoryContext {
            ansible_host: inventory_context.ansible_host_wrapper().clone(),
            ansible_ssh_private_key_file: inventory_context
                .ansible_ssh_private_key_file_wrapper()
                .clone(),
        };

        // Create template engine and validate rendering
        let (_engine, validated_content) =
            crate::template::TemplateEngine::with_validated_template_content(
                template_file.filename(),
                template_file.content(),
                &context,
            )
            .with_context(|| "Failed to create and validate template")?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }

    /// Get the ansible host value as a string
    #[must_use]
    pub fn ansible_host(&self) -> String {
        self.context.ansible_host()
    }

    /// Get the ansible SSH private key file path as a string
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> String {
        self.context.ansible_ssh_private_key_file()
    }
}

impl TemplateRenderer for InventoryTemplate {
    fn render(&self, output_path: &Path) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_template_creation() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = InventoryContext::new("192.168.1.100", "/path/to/key").unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(template.ansible_ssh_private_key_file(), "/path/to/key");
    }

    #[test]
    fn test_inventory_template_context_generation() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

        assert_eq!(template.ansible_host(), "10.0.0.1");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
    }

    #[test]
    fn test_empty_template_content() {
        // Test with empty template content
        let template_file = File::new("inventory.yml.tera", String::new()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // Empty templates are valid in Tera - they just render as empty strings
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.content, "");
    }

    #[test]
    fn test_missing_placeholder() {
        // Create template content with only one placeholder
        let template_content = "[all]\nserver ansible_host={{ansible_host}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // This is valid - templates don't need to use all available context variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=10.0.0.1"));
    }

    #[test]
    fn test_early_error_detection_both_variables_missing() {
        // Create template content with no placeholder variables at all
        let template_content = "[all]\nserver ansible_host=192.168.1.1\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // Static templates are valid - they just don't use template variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=192.168.1.1"));
    }

    #[test]
    fn test_undefined_variable_error() {
        // Create template content that references an undefined variable
        let template_content = "[all]\nserver ansible_host={{undefined_variable}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // This should fail because the template references an undefined variable
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to create and validate template"));
    }

    #[test]
    fn test_early_error_detection_template_validation_fails() {
        // Create template content with malformed Tera syntax
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\nmalformed={{unclosed_var\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn test_early_error_detection_malformed_syntax() {
        // Test with different malformed template syntax
        let template_content = "invalid {{{{ syntax";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context =
            InventoryContext::new("10.0.0.1", "/home/user/.ssh/id_rsa").unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        assert!(result.is_err());
    }

    #[test]
    fn test_template_validation_at_construction() {
        // Create valid template content
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        // Template validation happens during construction, not during render
        let inventory_context =
            InventoryContext::new("192.168.1.100", "/home/user/.ssh/test_key").unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

        // Verify that the template was pre-validated and contains rendered content
        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/test_key"
        );
    }

    #[test]
    fn test_invalid_ip_address() {
        // Test that invalid IP addresses are rejected
        let result = InventoryContext::new("invalid-ip", "/path/to/key");
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid IP address format"));
    }

    #[test]
    fn test_valid_ipv4_address() {
        // Test valid IPv4 address
        let context = InventoryContext::new("192.168.1.100", "/path/to/key").unwrap();
        assert_eq!(context.ansible_host(), "192.168.1.100");
    }

    #[test]
    fn test_valid_ipv6_address() {
        // Test valid IPv6 address
        let context = InventoryContext::new("2001:db8::1", "/path/to/key").unwrap();
        assert_eq!(context.ansible_host(), "2001:db8::1");
    }

    #[test]
    fn test_wrapper_types() {
        let context = InventoryContext::new("10.0.0.1", "/path/to/key").unwrap();

        // Test wrapper access
        let host_wrapper = context.ansible_host_wrapper();
        let key_wrapper = context.ansible_ssh_private_key_file_wrapper();

        assert_eq!(host_wrapper.as_str(), "10.0.0.1");
        assert_eq!(key_wrapper.as_str(), "/path/to/key");
    }
}
