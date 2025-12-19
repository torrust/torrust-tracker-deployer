use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};
use std::path::Path;

use super::context::AnsibleVariablesContext;

/// Wrapper for the variables template
#[derive(Debug)]
pub struct AnsibleVariablesTemplate {
    content: String,
}

impl AnsibleVariablesTemplate {
    /// Creates a new template with variable substitution
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    pub fn new(
        template_file: &File,
        context: &AnsibleVariablesContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();
        let validated_content =
            engine.render(template_file.filename(), template_file.content(), context)?;

        Ok(Self {
            content: validated_content,
        })
    }

    /// Render the template to a file
    ///
    /// # Errors
    ///
    /// Returns an error if file creation or directory creation fails
    pub fn render(&self, output_path: &Path) -> Result<(), FileOperationError> {
        write_file_with_dir_creation(output_path, &self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a `AnsibleVariablesContext` with the given SSH port
    fn create_variables_context(ssh_port: u16) -> AnsibleVariablesContext {
        AnsibleVariablesContext::new(ssh_port, None, None).unwrap()
    }

    /// Helper function to create a minimal valid variables template file
    fn create_minimal_template() -> File {
        let content = r"---
# Test template
ssh_port: {{ ssh_port }}
";
        File::new("variables.yml.tera", content.to_string()).unwrap()
    }

    #[test]
    fn it_should_create_variables_template_with_context() {
        let context = create_variables_context(22);
        let template_file = create_minimal_template();

        let template = AnsibleVariablesTemplate::new(&template_file, &context);

        assert!(template.is_ok());
    }

    #[test]
    fn it_should_render_template_with_ssh_port() {
        let context = create_variables_context(2222);
        let template_file = create_minimal_template();
        let template = AnsibleVariablesTemplate::new(&template_file, &context).unwrap();

        // The rendered content should have the port substituted
        assert!(template.content.contains("2222"));
        assert!(!template.content.contains("{{ ssh_port }}"));
    }

    #[test]
    fn it_should_fail_with_invalid_template_syntax() {
        let context = create_variables_context(22);
        let invalid_template =
            File::new("variables.yml.tera", "{{ unclosed_variable".to_string()).unwrap();

        let result = AnsibleVariablesTemplate::new(&invalid_template, &context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_with_missing_variable_in_context() {
        let context = create_variables_context(22);
        // Template references a variable that doesn't exist in context
        let template_with_missing_var = File::new(
            "variables.yml.tera",
            "Port: {{ ssh_port }} and {{ nonexistent_var }}".to_string(),
        )
        .unwrap();

        let result = AnsibleVariablesTemplate::new(&template_with_missing_var, &context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_support_custom_ssh_ports() {
        let context = create_variables_context(8022);
        let template_file = create_minimal_template();
        let template = AnsibleVariablesTemplate::new(&template_file, &context).unwrap();

        assert!(template.content.contains("8022"));
    }
}
