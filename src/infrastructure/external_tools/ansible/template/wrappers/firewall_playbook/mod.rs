//! Template wrapper for templates/ansible/configure-firewall.yml.tera
//!
//! This template configures UFW firewall with SSH access preservation.
//! It requires the SSH port to be provided at construction time.

pub mod context;

use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};
use anyhow::Result;
use std::path::Path;

pub use context::{
    FirewallPlaybookContext, FirewallPlaybookContextBuilder, FirewallPlaybookContextError,
};

/// Wrapper for the firewall playbook template
///
/// This wrapper validates the template syntax at construction time
/// and provides a type-safe way to render the firewall configuration
/// playbook with the correct SSH port.
#[derive(Debug)]
pub struct FirewallPlaybookTemplate {
    context: FirewallPlaybookContext,
    content: String,
}

impl FirewallPlaybookTemplate {
    /// Creates a new `FirewallPlaybookTemplate`, validating the template content and variable substitution
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
    /// This method will panic if cloning the already validated `FirewallPlaybookContext` fails,
    /// which should never happen under normal circumstances.
    pub fn new(
        template_file: &File,
        firewall_context: FirewallPlaybookContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();

        let validated_content = engine.render(
            template_file.filename(),
            template_file.content(),
            &firewall_context,
        )?;

        Ok(Self {
            context: firewall_context,
            content: validated_content,
        })
    }

    /// Get the SSH port value
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.context.ssh_port()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Errors
    ///
    /// Returns `FileOperationError::DirectoryCreation` if the parent directory cannot be created,
    /// or `FileOperationError::FileWrite` if the file cannot be written
    pub fn render(&self, output_path: &Path) -> Result<(), FileOperationError> {
        write_file_with_dir_creation(output_path, &self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::context::AnsiblePort;

    /// Helper function to create a `FirewallPlaybookContext` with the given SSH port
    fn create_firewall_context(ssh_port: u16) -> FirewallPlaybookContext {
        let port = AnsiblePort::new(ssh_port).unwrap();
        FirewallPlaybookContext::builder()
            .with_ssh_port(port)
            .build()
            .unwrap()
    }

    /// Helper function to create a minimal valid firewall template file
    fn create_minimal_template() -> File {
        let content = r#"---
- name: Configure UFW firewall
  hosts: all
  tasks:
    - name: Allow SSH on port {{ssh_port}}
      community.general.ufw:
        rule: allow
        port: "{{ssh_port}}"
"#;
        File::new("configure-firewall.yml.tera", content.to_string()).unwrap()
    }

    #[test]
    fn it_should_create_firewall_template_with_context() {
        let context = create_firewall_context(22);
        let template_file = create_minimal_template();

        let template = FirewallPlaybookTemplate::new(&template_file, context);

        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(template.ssh_port(), 22);
    }

    #[test]
    fn it_should_render_template_with_ssh_port() {
        let context = create_firewall_context(2222);
        let template_file = create_minimal_template();
        let template = FirewallPlaybookTemplate::new(&template_file, context).unwrap();

        // The rendered content should have the port substituted
        assert!(template.content.contains("2222"));
        assert!(!template.content.contains("{{ssh_port}}"));
    }

    #[test]
    fn it_should_fail_with_invalid_template_syntax() {
        let context = create_firewall_context(22);
        let invalid_template = File::new(
            "configure-firewall.yml.tera",
            "{{ unclosed_variable".to_string(),
        )
        .unwrap();

        let result = FirewallPlaybookTemplate::new(&invalid_template, context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_with_missing_variable_in_context() {
        let context = create_firewall_context(22);
        // Template references a variable that doesn't exist in context
        let template_with_missing_var = File::new(
            "configure-firewall.yml.tera",
            "Port: {{ssh_port}} and {{nonexistent_var}}".to_string(),
        )
        .unwrap();

        let result = FirewallPlaybookTemplate::new(&template_with_missing_var, context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_support_custom_ssh_ports() {
        let context = create_firewall_context(8022);
        let template_file = create_minimal_template();
        let template = FirewallPlaybookTemplate::new(&template_file, context).unwrap();

        assert_eq!(template.ssh_port(), 8022);
        assert!(template.content.contains("8022"));
    }
}
