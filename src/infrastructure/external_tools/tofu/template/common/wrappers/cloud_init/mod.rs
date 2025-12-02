//! Template wrapper for templates/tofu/common/cloud-init.yml.tera
//!
//! This template has mandatory variables that must be provided at construction time.
//! This wrapper is shared by all providers since the cloud-init template is the same.

pub mod context;

use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};
use anyhow::Result;
use std::path::Path;

pub use context::{CloudInitContext, CloudInitContextBuilder, CloudInitContextError};

#[derive(Debug)]
pub struct CloudInitTemplate {
    context: CloudInitContext,
    content: String,
}

impl CloudInitTemplate {
    /// Creates a new `CloudInitTemplate`, validating the template content and variable substitution
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
    /// This method will panic if cloning the already validated `CloudInitContext` fails,
    /// which should never happen under normal circumstances.
    pub fn new(
        template_file: &File,
        cloud_init_context: CloudInitContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();

        let validated_content = engine.render(
            template_file.filename(),
            template_file.content(),
            &cloud_init_context,
        )?;

        Ok(Self {
            context: cloud_init_context,
            content: validated_content,
        })
    }

    /// Get the SSH public key value
    #[must_use]
    pub fn ssh_public_key(&self) -> &str {
        self.context.ssh_public_key()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Errors
    /// Returns `FileOperationError::DirectoryCreation` if the parent directory cannot be created,
    /// or `FileOperationError::FileWrite` if the file cannot be written
    pub fn render(&self, output_path: &Path) -> Result<(), FileOperationError> {
        write_file_with_dir_creation(output_path, &self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a `CloudInitContext` with given SSH key
    fn create_cloud_init_context(ssh_key: &str) -> CloudInitContext {
        CloudInitContext::builder()
            .with_ssh_public_key(ssh_key)
            .unwrap()
            .with_username("testuser")
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    fn it_should_create_cloud_init_template_successfully() {
        // Use template content directly instead of file
        let template_content =
            "#cloud-config\nusers:\n  - ssh_authorized_keys:\n      - {{ ssh_public_key }}\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let template = CloudInitTemplate::new(&template_file, cloud_init_context).unwrap();

        assert_eq!(template.ssh_public_key(), ssh_key);
    }

    #[test]
    fn it_should_generate_cloud_init_template_context() {
        // Use template content directly instead of file
        let template_content =
            "#cloud-config\nusers:\n  - ssh_authorized_keys:\n      - {{ ssh_public_key }}\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let template = CloudInitTemplate::new(&template_file, cloud_init_context).unwrap();

        assert_eq!(template.ssh_public_key(), ssh_key);
    }

    #[test]
    fn it_should_accept_empty_template_content() {
        // Test with empty template content
        let template_file = File::new("cloud-init.yml.tera", String::new()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        // Empty templates are valid in Tera - they just render as empty strings
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.content, "");
    }

    #[test]
    fn it_should_work_with_missing_placeholder_variables() {
        // Create template content with no placeholder variables
        let template_content = "#cloud-config\nusers:\n  - name: static_user\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        // This is valid - templates don't need to use all available context variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("static_user"));
    }

    #[test]
    fn it_should_accept_static_template_with_no_variables() {
        // Create template content with no placeholder variables at all
        let template_content = "#cloud-config\npackages:\n  - curl\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        // Static templates are valid - they just don't use template variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("curl"));
    }

    #[test]
    fn it_should_fail_when_template_references_undefined_variable() {
        // Create template content that references an undefined variable
        let template_content =
            "#cloud-config\nusers:\n  - ssh_authorized_keys:\n      - {{ undefined_variable }}\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        // This should fail because the template references an undefined variable
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to render template") || error_msg.contains("template"));
    }

    #[test]
    fn it_should_fail_when_template_validation_fails() {
        // Create template content with malformed Tera syntax
        let template_content = "#cloud-config\nusers:\n  - ssh_authorized_keys:\n      - {{ ssh_public_key }}\nmalformed={{unclosed_var\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        // Test with different malformed template syntax
        let template_content = "invalid {{{{ syntax";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let result = CloudInitTemplate::new(&template_file, cloud_init_context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_validate_template_at_construction_time() {
        // Create valid template content
        let template_content =
            "#cloud-config\nusers:\n  - ssh_authorized_keys:\n      - {{ ssh_public_key }}\n";

        let template_file = File::new("cloud-init.yml.tera", template_content.to_string()).unwrap();

        // Template validation happens during construction, not during render
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let cloud_init_context = create_cloud_init_context(ssh_key);
        let template = CloudInitTemplate::new(&template_file, cloud_init_context).unwrap();

        // Verify that the template was pre-validated and contains rendered content
        assert_eq!(template.ssh_public_key(), ssh_key);
        assert!(template.content.contains(ssh_key));
    }
}
