//! Template wrapper for rendering the .env file
//!
//! This module provides the `EnvTemplate` type that handles rendering
//! of the env.tera template with environment variable context.

use std::path::Path;

use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};

use super::context::EnvContext;

/// Template wrapper for the env.tera template
///
/// Handles rendering of Docker Compose environment variables from the template.
#[derive(Debug)]
pub struct EnvTemplate {
    context: EnvContext,
    content: String,
}

impl EnvTemplate {
    /// Creates a new `EnvTemplate`, validating the template content and variable substitution
    ///
    /// # Arguments
    ///
    /// * `template_file` - The env.tera template file content
    /// * `env_context` - The context containing environment variables
    ///
    /// # Returns
    ///
    /// * `Result<Self, TemplateEngineError>` - The validated template or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template syntax is invalid
    /// - Required variables cannot be substituted
    /// - Template validation fails
    pub fn new(template_file: &File, env_context: EnvContext) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();

        let validated_content = engine.render(
            template_file.filename(),
            template_file.content(),
            &env_context,
        )?;

        Ok(Self {
            context: env_context,
            content: validated_content,
        })
    }

    /// Get the tracker API admin token
    #[must_use]
    pub fn tracker_api_admin_token(&self) -> &str {
        self.context.tracker_api_admin_token()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Arguments
    ///
    /// * `output_path` - The path where the .env file should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), FileOperationError>` - Success or file operation error
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

    #[test]
    fn it_should_create_env_template_successfully() {
        let template_content = "TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN={{ tracker.api_admin_token }}\n";

        let template_file = File::new(".env.tera", template_content.to_string()).unwrap();

        let env_context = EnvContext::new("MyToken123".to_string());
        let template = EnvTemplate::new(&template_file, env_context).unwrap();

        assert_eq!(template.tracker_api_admin_token(), "MyToken123");
    }

    #[test]
    fn it_should_render_template_with_substituted_variables() {
        let template_content = "TOKEN={{ tracker.api_admin_token }}\n";

        let template_file = File::new(".env.tera", template_content.to_string()).unwrap();

        let env_context = EnvContext::new("SecretToken".to_string());
        let template = EnvTemplate::new(&template_file, env_context).unwrap();

        // Verify the content has the substituted value
        assert!(template.content.contains("TOKEN=SecretToken"));
    }

    #[test]
    fn it_should_accept_empty_template_content() {
        let template_file = File::new(".env.tera", String::new()).unwrap();

        let env_context = EnvContext::new("TestToken".to_string());
        let result = EnvTemplate::new(&template_file, env_context);

        // Empty templates are valid in Tera
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.content, "");
    }

    #[test]
    fn it_should_work_with_missing_placeholder_variables() {
        // Template with no placeholders
        let template_content = "STATIC_VALUE=123\n";

        let template_file = File::new(".env.tera", template_content.to_string()).unwrap();

        let env_context = EnvContext::new("UnusedToken".to_string());
        let result = EnvTemplate::new(&template_file, env_context);

        // Templates don't need to use all available context variables
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.content.contains("STATIC_VALUE=123"));
    }

    #[test]
    fn it_should_render_to_file() {
        use tempfile::TempDir;

        let template_content = "ADMIN_TOKEN={{ tracker.api_admin_token }}\n";
        let template_file = File::new(".env.tera", template_content.to_string()).unwrap();

        let env_context = EnvContext::new("FileTestToken".to_string());
        let template = EnvTemplate::new(&template_file, env_context).unwrap();

        // Create temp directory for output
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join(".env");

        // Render to file
        template.render(&output_path).unwrap();

        // Verify file was created and contains expected content
        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("ADMIN_TOKEN=FileTestToken"));
    }
}
