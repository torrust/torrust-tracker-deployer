//! Template wrapper for rendering the docker-compose.yml file
//!
//! This module provides the `DockerComposeTemplate` type that handles rendering
//! of the docker-compose.yml.tera template with service configuration context.

use std::path::Path;

use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};

use super::context::DockerComposeContext;

/// Template wrapper for the docker-compose.yml.tera template
///
/// Handles rendering of Docker Compose service definitions from the template.
#[derive(Debug)]
pub struct DockerComposeTemplate {
    context: DockerComposeContext,
    content: String,
}

impl DockerComposeTemplate {
    /// Creates a new `DockerComposeTemplate`, validating the template content and variable substitution
    ///
    /// # Arguments
    ///
    /// * `template_file` - The docker-compose.yml.tera template file content
    /// * `context` - The context containing service configuration
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
    pub fn new(
        template_file: &File,
        context: DockerComposeContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();

        let validated_content =
            engine.render(template_file.filename(), template_file.content(), &context)?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }

    /// Get the database configuration
    #[must_use]
    pub fn database(&self) -> &crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::DatabaseConfig{
        self.context.database()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Arguments
    ///
    /// * `output_path` - The path where the docker-compose.yml file should be written
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
    fn it_should_create_docker_compose_template_with_sqlite() {
        let template_content = r#"
services:
  tracker:
    image: torrust/tracker:develop
{% if database.driver == "mysql" %}
  mysql:
    image: mysql:8.0
{% endif %}
"#;

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let context = DockerComposeContext::new_sqlite();
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        assert_eq!(template.database().driver(), "sqlite3");
        // MySQL service should not be in the rendered content
        assert!(!template.content.contains("mysql:"));
    }

    #[test]
    fn it_should_create_docker_compose_template_with_mysql() {
        let template_content = r#"
services:
  tracker:
    image: torrust/tracker:develop
{% if database.driver == "mysql" %}
  mysql:
    image: mysql:8.0
    environment:
      - MYSQL_ROOT_PASSWORD={{ database.mysql.root_password }}
{% endif %}
"#;

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let context = DockerComposeContext::new_mysql(
            "root123".to_string(),
            "tracker".to_string(),
            "user".to_string(),
            "pass".to_string(),
            3306,
        );
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        assert_eq!(template.database().driver(), "mysql");
        // MySQL service should be in the rendered content
        assert!(template.content.contains("mysql:"));
        assert!(template.content.contains("MYSQL_ROOT_PASSWORD=root123"));
    }

    #[test]
    fn it_should_render_to_file() {
        use tempfile::TempDir;

        let template_content = r"
services:
  tracker:
    image: torrust/tracker:develop
";
        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let context = DockerComposeContext::new_sqlite();
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        // Create temp directory for output
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("docker-compose.yml");

        // Render to file
        template.render(&output_path).unwrap();

        // Verify file exists and contains expected content
        assert!(output_path.exists());
        let file_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(file_content.contains("torrust/tracker:develop"));
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        let template_content = "{% if database.driver == mysql %}"; // Missing quotes and `endif`

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let context = DockerComposeContext::new_sqlite();
        let result = DockerComposeTemplate::new(&template_file, context);

        assert!(result.is_err());
    }
}
