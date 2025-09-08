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
use thiserror::Error;

#[cfg(test)]
use std::str::FromStr;

pub use ansible_host::{AnsibleHost, AnsibleHostError};
pub use ssh_private_key_file::{SshPrivateKeyFile, SshPrivateKeyFileError};

/// Errors that can occur when creating an `InventoryContext`
#[derive(Debug, Error)]
pub enum InventoryContextError {
    #[error("Invalid ansible host: {0}")]
    InvalidAnsibleHost(#[from] AnsibleHostError),

    #[error("Invalid SSH private key file: {0}")]
    InvalidSshPrivateKeyFile(#[from] SshPrivateKeyFileError),

    #[error("Missing ansible host - must be set before building")]
    MissingAnsibleHost,

    #[error("Missing SSH private key file - must be set before building")]
    MissingSshPrivateKeyFile,
}

/// Errors that can occur when creating an `InventoryTemplate`
#[derive(Debug, Error)]
pub enum InventoryTemplateError {
    #[error("Failed to create template engine: {0}")]
    TemplateEngineCreation(String),

    #[error("Template validation failed: {0}")]
    TemplateValidation(String),

    #[error("Failed to render template: {0}")]
    TemplateRendering(String),
}

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

/// Builder for `InventoryContext` with fluent interface
#[derive(Debug, Default)]
pub struct InventoryContextBuilder {
    ansible_host: Option<AnsibleHost>,
    ansible_ssh_private_key_file: Option<SshPrivateKeyFile>,
}

impl InventoryContextBuilder {
    /// Creates a new empty builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Ansible host for the builder.
    #[must_use]
    pub fn with_host(mut self, ansible_host: AnsibleHost) -> Self {
        self.ansible_host = Some(ansible_host);
        self
    }

    /// Sets the SSH private key file path for the builder.
    #[must_use]
    pub fn with_ssh_priv_key_path(mut self, ssh_private_key_file: SshPrivateKeyFile) -> Self {
        self.ansible_ssh_private_key_file = Some(ssh_private_key_file);
        self
    }

    /// Builds the `InventoryContext`
    ///
    /// # Errors
    ///
    /// Returns an error if either `ansible_host` or `ansible_ssh_private_key_file` is missing
    pub fn build(self) -> Result<InventoryContext, InventoryContextError> {
        let ansible_host = self
            .ansible_host
            .ok_or(InventoryContextError::MissingAnsibleHost)?;

        let ansible_ssh_private_key_file = self
            .ansible_ssh_private_key_file
            .ok_or(InventoryContextError::MissingSshPrivateKeyFile)?;

        Ok(InventoryContext {
            ansible_host,
            ansible_ssh_private_key_file,
        })
    }
}

impl InventoryContext {
    /// Creates a new `InventoryContext` using typed parameters
    ///
    /// # Errors
    ///
    /// This method cannot fail with the current implementation since it takes
    /// already validated types, but returns Result for consistency with builder pattern
    pub fn new(
        ansible_host: AnsibleHost,
        ansible_ssh_private_key_file: SshPrivateKeyFile,
    ) -> Result<Self, InventoryContextError> {
        Ok(Self {
            ansible_host,
            ansible_ssh_private_key_file,
        })
    }

    /// Creates a new builder for `InventoryContext` with fluent interface
    #[must_use]
    pub fn builder() -> InventoryContextBuilder {
        InventoryContextBuilder::new()
    }

    /// Get the ansible host value as a string
    #[must_use]
    pub fn ansible_host(&self) -> String {
        self.ansible_host.as_str()
    }

    /// Get the ansible SSH private key file path as a string
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> String {
        self.ansible_ssh_private_key_file.as_str()
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
    pub fn new(
        template_file: &File,
        inventory_context: &InventoryContext,
    ) -> Result<Self, InventoryTemplateError> {
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
            .map_err(|e| InventoryTemplateError::TemplateEngineCreation(e.to_string()))?;

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
    fn it_should_create_inventory_template_successfully() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(template.ansible_ssh_private_key_file(), "/path/to/key");
    }

    #[test]
    fn it_should_generate_inventory_template_context() {
        // Use template content directly instead of file
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

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

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

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

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

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

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

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

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // This should fail because the template references an undefined variable
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to create template engine")
                || error_msg.contains("template engine")
        );
    }

    #[test]
    fn it_should_fail_when_template_validation_fails() {
        // Create template content with malformed Tera syntax
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\nmalformed={{unclosed_var\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        // Test with different malformed template syntax
        let template_content = "invalid {{{{ syntax";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let result = InventoryTemplate::new(&template_file, &inventory_context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_validate_template_at_construction_time() {
        // Create valid template content
        let template_content = "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n";

        let template_file = File::new("inventory.yml.tera", template_content.to_string()).unwrap();

        // Template validation happens during construction, not during render
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/test_key").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        let template = InventoryTemplate::new(&template_file, &inventory_context).unwrap();

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
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        assert_eq!(context.ansible_host(), "192.168.1.100");
    }

    #[test]
    fn it_should_accept_valid_ipv6_address() {
        // Test valid IPv6 address
        let host = AnsibleHost::from_str("2001:db8::1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();
        assert_eq!(context.ansible_host(), "2001:db8::1");
    }

    #[test]
    fn it_should_provide_access_to_wrapper_types() {
        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();

        // Test wrapper access
        let host_wrapper = context.ansible_host_wrapper();
        let key_wrapper = context.ansible_ssh_private_key_file_wrapper();

        assert_eq!(host_wrapper.as_str(), "10.0.0.1");
        assert_eq!(key_wrapper.as_str(), "/path/to/key");
    }

    #[test]
    fn it_should_support_builder_pattern_fluent_interface() {
        // Test the fluent builder interface as requested
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();

        assert_eq!(inventory_context.ansible_host(), "192.168.1.100");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
    }

    #[test]
    fn it_should_work_with_builder_typed_parameters() {
        // Test builder with typed parameters instead of strings
        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();

        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .unwrap();

        assert_eq!(inventory_context.ansible_host(), "10.0.0.1");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/path/to/key"
        );
    }

    #[test]
    fn it_should_fail_when_builder_missing_host() {
        // Test that builder fails when host is missing
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let result = InventoryContext::builder()
            .with_ssh_priv_key_path(ssh_key)
            .build();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing ansible host"));
    }

    #[test]
    fn it_should_fail_when_builder_missing_ssh_key() {
        // Test that builder fails when SSH key is missing
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let result = InventoryContext::builder().with_host(host).build();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing SSH private key file"));
    }

    #[test]
    fn it_should_create_new_inventory_context_with_typed_parameters() {
        // Test the new direct constructor with typed parameters
        let host = AnsibleHost::from_str("192.168.1.50").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/etc/ssh/test_key").unwrap();

        let inventory_context = InventoryContext::new(host, ssh_key).unwrap();

        assert_eq!(inventory_context.ansible_host(), "192.168.1.50");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/etc/ssh/test_key"
        );
    }
}
