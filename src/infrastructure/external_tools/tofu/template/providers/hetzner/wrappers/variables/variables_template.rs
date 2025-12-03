//! `VariablesTemplate` type and implementation for Hetzner Cloud.

use std::path::Path;

use crate::domain::template::file::File;
use crate::domain::template::{write_file_with_dir_creation, TemplateEngine};
use crate::infrastructure::external_tools::tofu::template::common::wrappers::VariablesTemplateError;

use super::context::VariablesContext;

/// Template wrapper for Hetzner Cloud `OpenTofu` variables rendering
///
/// Validates and renders `variables.tfvars.tera` templates with `VariablesContext`
/// to produce dynamic infrastructure variable files for Hetzner Cloud.
#[derive(Debug)]
pub struct VariablesTemplate {
    context: VariablesContext,
    content: String,
}

impl VariablesTemplate {
    /// Creates a new Hetzner variables template with validation
    ///
    /// # Arguments
    ///
    /// * `template_file` - The template file containing variables.tfvars.tera content
    /// * `context` - The context containing Hetzner-specific runtime values
    ///
    /// # Returns
    ///
    /// * `Ok(VariablesTemplate)` if template validation succeeds
    /// * `Err(VariablesTemplateError)` if validation fails
    ///
    /// # Errors
    ///
    /// Returns `TemplateEngineError` if the template has syntax errors or validation fails
    pub fn new(
        template_file: &File,
        context: VariablesContext,
    ) -> Result<Self, VariablesTemplateError> {
        let mut engine = TemplateEngine::new();

        let validated_content =
            engine.render(template_file.filename(), template_file.content(), &context)?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }

    /// Get the instance name value
    #[must_use]
    pub fn instance_name(&self) -> &str {
        self.context.instance_name.as_str()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Errors
    /// Returns `FileOperationError::DirectoryCreation` if the parent directory cannot be created,
    /// or `FileOperationError::FileWrite` if the file cannot be written
    pub fn render(&self, output_path: &Path) -> Result<(), VariablesTemplateError> {
        write_file_with_dir_creation(output_path, &self.content)?;
        Ok(())
    }

    /// Gets the context used by this template
    #[must_use]
    pub fn context(&self) -> &VariablesContext {
        &self.context
    }

    /// Gets the rendered content
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::InstanceName;
    use tempfile::NamedTempFile;

    fn create_test_context() -> VariablesContext {
        VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-instance".to_string()).unwrap())
            .with_hcloud_api_token("test-api-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA... test@example.com".to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn it_should_create_variables_template_successfully() {
        let template_content = r#"hcloud_api_token = "{{ hcloud_api_token }}"
server_name = "{{ instance_name }}""#;

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();

        let result = VariablesTemplate::new(&template_file, context);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        let template_content = r#"hcloud_api_token = "{{ hcloud_api_token
server_name = "{{ instance_name }}""#; // Missing closing }}

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();

        let result = VariablesTemplate::new(&template_file, context);
        assert!(matches!(
            result.unwrap_err(),
            VariablesTemplateError::TemplateEngineError { .. }
        ));
    }

    #[test]
    fn it_should_accept_static_template_with_no_variables() {
        let template_content = r#"hcloud_api_token = "hardcoded-token"
server_type = "cx22""#;

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();

        let result = VariablesTemplate::new(&template_file, context);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_accept_empty_template_content() {
        let template_content = "";

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();

        let result = VariablesTemplate::new(&template_file, context);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_render_variables_template_successfully() {
        let template_content = r#"# OpenTofu Variables for Hetzner Cloud
hcloud_api_token = "{{ hcloud_api_token }}"
server_name = "{{ instance_name }}"
server_type = "{{ server_type }}""#;

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();
        let variables_template = VariablesTemplate::new(&template_file, context).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let result = variables_template.render(temp_file.path());

        assert!(result.is_ok());

        // Verify rendered content
        let rendered_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(rendered_content.contains(r#"hcloud_api_token = "test-api-token""#));
        assert!(rendered_content.contains(r#"server_name = "test-instance""#));
        assert!(rendered_content.contains(r#"server_type = "cx22""#));
    }

    #[test]
    fn it_should_provide_access_to_context() {
        let template_file = File::new("variables.tfvars.tera", String::new()).unwrap();
        let context = create_test_context();
        let variables_template = VariablesTemplate::new(&template_file, context).unwrap();

        assert_eq!(
            variables_template.context().instance_name.as_str(),
            "test-instance"
        );
    }

    #[test]
    fn it_should_provide_access_to_rendered_content() {
        let template_content = r#"server_name = "{{ instance_name }}""#;
        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();
        let variables_template = VariablesTemplate::new(&template_file, context).unwrap();

        assert!(variables_template.content().contains("test-instance"));
    }

    #[test]
    fn it_should_work_with_missing_placeholder_variables() {
        // Template has no placeholders but context has values - should work fine
        let template_content = r#"hcloud_api_token = "hardcoded"
server_type = "cx22""#;

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();
        let variables_template = VariablesTemplate::new(&template_file, context).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let result = variables_template.render(temp_file.path());

        assert!(result.is_ok());

        let rendered_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(rendered_content.contains(r#"hcloud_api_token = "hardcoded""#));
    }

    #[test]
    fn it_should_validate_template_at_construction_time() {
        let template_content = r#"hcloud_api_token = "{{ undefined_variable }}"
server_type = "cx22""#;

        let template_file =
            File::new("variables.tfvars.tera", template_content.to_string()).unwrap();
        let context = create_test_context();

        // Should fail at construction, not during render
        let result = VariablesTemplate::new(&template_file, context);
        assert!(matches!(
            result.unwrap_err(),
            VariablesTemplateError::TemplateEngineError { .. }
        ));
    }

    #[test]
    fn it_should_generate_variables_template_context() {
        let template_file =
            File::new("variables.tfvars.tera", "{{ instance_name }}".to_string()).unwrap();
        let context = VariablesContext::builder()
            .with_instance_name(InstanceName::new("dynamic-vm".to_string()).unwrap())
            .with_hcloud_api_token("dynamic-token".to_string())
            .with_server_type("cx32".to_string())
            .with_server_location("fsn1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build()
            .unwrap();

        let variables_template = VariablesTemplate::new(&template_file, context).unwrap();
        assert_eq!(
            variables_template.context().instance_name.as_str(),
            "dynamic-vm"
        );
    }
}
