//! Template wrapper for templates/ansible/inventory.yml
//!
//! This template has mandatory variables that must be provided at construction time.

pub mod context;

use crate::template::file::File;
use crate::template::{TemplateEngineError, TemplateRenderer};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(test)]
use std::str::FromStr;

pub use context::{
    AnsibleHost, AnsibleHostError, InventoryContext, InventoryContextBuilder, InventoryContextError,
};
pub use context::{SshPrivateKeyFile, SshPrivateKeyFileError};

#[derive(Debug)]
pub struct InventoryTemplate {
    context: InventoryContext,
    content: String,
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
    ///
    /// # Panics
    ///
    /// This method will panic if cloning the already validated `InventoryContext` fails,
    /// which should never happen under normal circumstances.
    pub fn new(
        template_file: &File,
        inventory_context: InventoryContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::template::TemplateEngine::new();

        let validated_content = engine.render(
            template_file.filename(),
            template_file.content(),
            &inventory_context,
        )?;

        Ok(Self {
            context: inventory_context,
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

    /// Helper function to create an `InventoryContext` with given host and SSH key path
    fn create_inventory_context(host_ip: &str, ssh_key_path: &str) -> InventoryContext {
        let host = AnsibleHost::from_str(host_ip).unwrap();
        let ssh_key = SshPrivateKeyFile::new(ssh_key_path).unwrap();
        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap()
    }

    #[test]
    fn it_should_create_inventory_template_successfully() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("192.168.1.100", "/path/to/key");
        let template = InventoryTemplate::new(&template_file, inventory_context).unwrap();

        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(template.ansible_ssh_private_key_file(), "/path/to/key");
    }

    #[test]
    fn it_should_generate_inventory_template_context() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let template = InventoryTemplate::new(&template_file, inventory_context).unwrap();

        assert_eq!(template.ansible_host(), "10.0.0.1");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
    }

    #[test]
    fn it_should_accept_empty_template_content() {
        // Test with empty template content
        let template_file = File::new("inventory.yml.tera", String::new()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // Empty templates are valid in Tera - they just render as empty strings
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.content, "");
    }

    #[test]
    fn it_should_work_with_missing_placeholder_variables() {
        // Create template content with only one placeholder
        let template_content = "[all]\nserver ansible_host={{ansible_host}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // This is valid - templates don't need to use all available context variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=10.0.0.1"));
    }

    #[test]
    fn it_should_accept_static_template_with_no_variables() {
        // Create template content with no placeholder variables at all
        let template_content = "[all]\nserver ansible_host=192.168.1.1\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // Static templates are valid - they just don't use template variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("ansible_host=192.168.1.1"));
    }

    #[test]
    fn it_should_fail_when_template_references_undefined_variable() {
        // Create template content that references an undefined variable
        let template_content = "[all]\nserver ansible_host={{undefined_variable}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // This should fail because the template references an undefined variable
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to render template") || error_msg.contains("template"));
    }

    #[test]
    fn it_should_fail_when_template_validation_fails() {
        // Create template content with malformed Tera syntax
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\nmalformed={{unclosed_var\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        // Test with different malformed template syntax
        let template_content = "invalid {{{{ syntax";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let inventory_context = create_inventory_context("10.0.0.1", "/home/user/.ssh/id_rsa");
        let result = InventoryTemplate::new(&template_file, inventory_context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_validate_template_at_construction_time() {
        // Create valid template content
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        // Template validation happens during construction, not during render
        let inventory_context =
            create_inventory_context("192.168.1.100", "/home/user/.ssh/test_key");
        let template = InventoryTemplate::new(&template_file, inventory_context).unwrap();

        // Verify that the template was pre-validated and contains rendered content
        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/test_key"
        );
    }

    #[test]
    fn it_should_reject_invalid_ip_address() {
        // Test that invalid IP addresses are rejected by the AnsibleHost wrapper
        let result = AnsibleHost::from_str("invalid-ip");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid IP address format"));
    }

    #[test]
    fn it_should_accept_valid_ipv4_address() {
        // Test valid IPv4 address
        let context = create_inventory_context("192.168.1.100", "/path/to/key");
        assert_eq!(context.ansible_host(), "192.168.1.100");
    }

    #[test]
    fn it_should_accept_valid_ipv6_address() {
        // Test valid IPv6 address
        let context = create_inventory_context("2001:db8::1", "/path/to/key");
        assert_eq!(context.ansible_host(), "2001:db8::1");
    }
}
