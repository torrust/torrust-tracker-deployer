//! # Variables Template Renderer
//!
//! This module handles rendering of the `variables.yml.tera` template
//! with system configuration variables. It's responsible for creating the centralized
//! variables file that consolidates Ansible playbook variables.
//!
//! ## Responsibilities
//!
//! - Load the `variables.yml.tera` template file
//! - Process template with system configuration variables
//! - Render final `variables.yml` file for Ansible consumption
//!
//! ## Usage
//!
//! ```rust
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! use torrust_tracker_deployer_lib::infrastructure::templating::ansible::template::renderer::variables::VariablesRenderer;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! use torrust_tracker_deployer_lib::infrastructure::templating::ansible::template::wrappers::variables::AnsibleVariablesContext;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = VariablesRenderer::new(template_manager);
//!
//! let variables_context = AnsibleVariablesContext::new(22, None, None)?;
//! renderer.render(&variables_context, temp_dir.path())?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::ansible::template::wrappers::variables::{
    AnsibleVariablesContext, AnsibleVariablesTemplate,
};

/// Errors that can occur during variables template rendering
#[derive(Error, Debug)]
pub enum VariablesRendererError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for '{file_name}': {source}")]
    TemplatePathFailed {
        file_name: String,
        #[source]
        source: TemplateManagerError,
    },

    /// Failed to read Tera template file content
    #[error("Failed to read Tera template file '{file_name}': {source}")]
    TeraTemplateReadFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create File object from template content
    #[error("Failed to create File object for '{file_name}': {source}")]
    FileCreationFailed {
        file_name: String,
        #[source]
        source: crate::domain::template::file::Error,
    },

    /// Failed to create variables template with provided context
    #[error("Failed to create AnsibleVariablesTemplate: {source}")]
    VariablesTemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    /// Failed to render variables template to output file
    #[error("Failed to render variables template to file: {source}")]
    VariablesTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Handles rendering of the variables.yml.tera template for Ansible deployments
///
/// This collaborator is responsible for all variables template-specific operations:
/// - Loading the variables.yml.tera template
/// - Processing it with system configuration variables
/// - Rendering the final variables.yml file for Ansible consumption
pub struct VariablesRenderer {
    template_manager: Arc<TemplateManager>,
}

impl VariablesRenderer {
    /// Template filename for the variables Tera template
    const VARIABLES_TEMPLATE_FILE: &'static str = "variables.yml.tera";

    /// Output filename for the rendered variables file
    const VARIABLES_OUTPUT_FILE: &'static str = "variables.yml";

    /// Directory path for Ansible templates
    const ANSIBLE_TEMPLATE_DIR: &'static str = "ansible";

    /// Creates a new variables template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the variables.yml.tera template with the provided context
    ///
    /// This method:
    /// 1. Loads the variables.yml.tera template from the template manager
    /// 2. Reads the template content
    /// 3. Creates a File object for template processing
    /// 4. Creates a `AnsibleVariablesTemplate` with the system configuration context
    /// 5. Renders the template to variables.yml in the output directory
    ///
    /// # Arguments
    ///
    /// * `variables_context` - The context containing system configuration variables
    /// * `output_dir` - The directory where variables.yml should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), VariablesTemplateError>` - Success or error from the template rendering operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be found or read
    /// - Template content is invalid
    /// - Variable substitution fails
    /// - Output file cannot be written
    pub fn render(
        &self,
        variables_context: &AnsibleVariablesContext,
        output_dir: &Path,
    ) -> Result<(), VariablesRendererError> {
        tracing::debug!("Rendering variables template with system configuration");

        // Get the variables template path
        let variables_template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| VariablesRendererError::TemplatePathFailed {
                file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let variables_template_content = std::fs::read_to_string(&variables_template_path)
            .map_err(|source| VariablesRendererError::TeraTemplateReadFailed {
                file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Create File object for template processing
        let variables_template_file =
            File::new(Self::VARIABLES_TEMPLATE_FILE, variables_template_content).map_err(
                |source| VariablesRendererError::FileCreationFailed {
                    file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                    source,
                },
            )?;

        // Create AnsibleVariablesTemplate with system configuration context
        let variables_template =
            AnsibleVariablesTemplate::new(&variables_template_file, variables_context).map_err(
                |source| VariablesRendererError::VariablesTemplateCreationFailed { source },
            )?;

        // Render to output file
        let variables_output_path = output_dir.join(Self::VARIABLES_OUTPUT_FILE);
        variables_template
            .render(&variables_output_path)
            .map_err(|source| VariablesRendererError::VariablesTemplateRenderFailed { source })?;

        tracing::debug!(
            "Successfully rendered variables template to {}",
            variables_output_path.display()
        );

        Ok(())
    }

    /// Builds the full template path for the variables template
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for variables.yml.tera
    fn build_template_path() -> String {
        format!(
            "{}/{}",
            Self::ANSIBLE_TEMPLATE_DIR,
            Self::VARIABLES_TEMPLATE_FILE
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test variables context
    fn create_test_variables_context() -> AnsibleVariablesContext {
        AnsibleVariablesContext::new(22, None, None).expect("Failed to create variables context")
    }

    /// Helper function to create a test template directory with variables.yml.tera
    fn create_test_templates(temp_dir: &Path) -> std::io::Result<()> {
        let ansible_dir = temp_dir.join("ansible");
        fs::create_dir_all(&ansible_dir)?;

        let template_content = r"---
# Centralized Ansible Variables
ssh_port: {{ ssh_port }}
";

        fs::write(ansible_dir.join("variables.yml.tera"), template_content)?;

        Ok(())
    }

    #[test]
    fn it_should_create_variables_renderer_with_template_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let renderer = VariablesRenderer::new(template_manager.clone());

        assert!(Arc::ptr_eq(&renderer.template_manager, &template_manager));
    }

    #[test]
    fn it_should_build_correct_template_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let _renderer = VariablesRenderer::new(template_manager);

        let template_path = VariablesRenderer::build_template_path();

        assert_eq!(template_path, "ansible/variables.yml.tera");
    }

    #[test]
    fn it_should_render_variables_template_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        // Create template directory and files
        create_test_templates(&template_dir).expect("Failed to create test templates");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Setup template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = VariablesRenderer::new(template_manager);
        let variables_context = create_test_variables_context();

        // Render template
        let result = renderer.render(&variables_context, &output_dir);

        assert!(result.is_ok(), "Template rendering should succeed");

        // Verify output file exists
        let output_file = output_dir.join("variables.yml");
        assert!(output_file.exists(), "variables.yml should be created");

        // Verify output content contains expected values
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(
            output_content.contains("ssh_port: 22"),
            "Output should contain the SSH port"
        );
        assert!(
            !output_content.contains("{{ ssh_port }}"),
            "Output should not contain template variables"
        );
    }

    #[test]
    fn it_should_render_with_custom_ssh_port() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        create_test_templates(&template_dir).expect("Failed to create test templates");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = VariablesRenderer::new(template_manager);

        // Use custom SSH port
        let variables_context = AnsibleVariablesContext::new(2222, None, None)
            .expect("Failed to create variables context");

        let result = renderer.render(&variables_context, &output_dir);

        assert!(result.is_ok());

        let output_file = output_dir.join("variables.yml");
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(
            output_content.contains("ssh_port: 2222"),
            "Output should contain custom SSH port 2222"
        );
    }
}
