//! # .env Template Renderer
//!
//! This module handles rendering of the `.env.tera` template for Docker Compose deployments.
//! It's responsible for creating `.env` files with environment variables from dynamic configuration.
//!
//! ## Responsibilities
//!
//! - Load the `env.tera` template file
//! - Process template with runtime context (tracker admin token, etc.)
//! - Render final `.env` file for Docker Compose consumption
//!
//! ## Usage
//!
//! ```rust
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::renderer::env::EnvRenderer;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = EnvRenderer::new(template_manager);
//!
//! let env_context = EnvContext::new("MyAccessToken".to_string());
//! renderer.render(&env_context, temp_dir.path())?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::docker_compose::template::wrappers::env::{
    EnvContext, EnvTemplate,
};

/// Errors that can occur during .env template rendering
#[derive(Error, Debug)]
pub enum EnvRendererError {
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

    /// Failed to create .env template with provided context
    #[error("Failed to create EnvTemplate: {source}")]
    EnvTemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    /// Failed to render .env template to output file
    #[error("Failed to render .env template to file: {source}")]
    EnvTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Handles rendering of the env.tera template for Docker Compose deployments
///
/// This collaborator is responsible for all .env template-specific operations:
/// - Loading the env.tera template
/// - Processing it with runtime context (tracker admin token, etc.)
/// - Rendering the final .env file for Docker Compose consumption
pub struct EnvRenderer {
    template_manager: Arc<TemplateManager>,
}

impl EnvRenderer {
    /// Template filename for the .env Tera template
    const ENV_TEMPLATE_FILE: &'static str = ".env.tera";

    /// Output filename for the rendered .env file
    const ENV_OUTPUT_FILE: &'static str = ".env";

    /// Creates a new .env template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the env.tera template with the provided context
    ///
    /// This method:
    /// 1. Loads the env.tera template from the template manager
    /// 2. Reads the template content
    /// 3. Creates a File object for template processing
    /// 4. Creates an `EnvTemplate` with the runtime context
    /// 5. Renders the template to .env in the output directory
    ///
    /// # Arguments
    ///
    /// * `env_context` - The context containing environment variables
    /// * `output_dir` - The directory where .env should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), EnvRendererError>` - Success or error from the template rendering operation
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
        env_context: &EnvContext,
        output_dir: &Path,
    ) -> Result<(), EnvRendererError> {
        tracing::debug!("Rendering .env template with runtime variables");

        // Get the .env template path
        let env_template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| EnvRendererError::TemplatePathFailed {
                file_name: Self::ENV_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let env_template_content =
            std::fs::read_to_string(&env_template_path).map_err(|source| {
                EnvRendererError::TeraTemplateReadFailed {
                    file_name: Self::ENV_TEMPLATE_FILE.to_string(),
                    source,
                }
            })?;

        // Create File object for template processing
        let env_template_file =
            File::new(Self::ENV_TEMPLATE_FILE, env_template_content).map_err(|source| {
                EnvRendererError::FileCreationFailed {
                    file_name: Self::ENV_TEMPLATE_FILE.to_string(),
                    source,
                }
            })?;

        // Create EnvTemplate with runtime context
        let env_template = EnvTemplate::new(&env_template_file, env_context.clone())
            .map_err(|source| EnvRendererError::EnvTemplateCreationFailed { source })?;

        // Render to output file
        let env_output_path = output_dir.join(Self::ENV_OUTPUT_FILE);
        env_template
            .render(&env_output_path)
            .map_err(|source| EnvRendererError::EnvTemplateRenderFailed { source })?;

        tracing::debug!(
            "Successfully rendered .env template to {}",
            env_output_path.display()
        );

        Ok(())
    }

    /// Builds the full template path for the .env template
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for env.tera
    fn build_template_path() -> String {
        format!("docker-compose/{}", Self::ENV_TEMPLATE_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test .env context
    fn create_test_env_context() -> EnvContext {
        EnvContext::new("TestAdminToken123".to_string())
    }

    /// Helper function to create a test template directory with env.tera
    fn create_test_templates(temp_dir: &Path) -> std::io::Result<()> {
        let docker_compose_dir = temp_dir.join("docker-compose");
        fs::create_dir_all(&docker_compose_dir)?;

        let template_content = r"# Docker Compose Environment Variables for Torrust Tracker
# This file is automatically generated - do not edit manually

# Path to the tracker configuration file
TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml

# Override the admin token for the tracker HTTP API
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN={{ tracker_api_admin_token }}
";

        fs::write(docker_compose_dir.join(".env.tera"), template_content)?;

        Ok(())
    }

    #[test]
    fn test_env_renderer_renders_template_successfully() {
        // Setup: Create temporary directories for templates and output
        let templates_temp_dir = TempDir::new().expect("Failed to create templates temp directory");
        let output_temp_dir = TempDir::new().expect("Failed to create output temp directory");

        create_test_templates(templates_temp_dir.path()).expect("Failed to create test templates");

        // Setup: Create template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(templates_temp_dir.path()));
        let renderer = EnvRenderer::new(template_manager);

        // Setup: Create test context
        let env_context = create_test_env_context();

        // Execute: Render the .env template
        renderer
            .render(&env_context, output_temp_dir.path())
            .expect("Failed to render .env template");

        // Verify: Check that .env file was created
        let env_output_path = output_temp_dir.path().join(".env");
        assert!(
            env_output_path.exists(),
            ".env file should exist after rendering"
        );

        // Verify: Check that rendered content contains the expected admin token
        let rendered_content =
            fs::read_to_string(&env_output_path).expect("Failed to read rendered .env file");
        assert!(
            rendered_content.contains(
                "TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=TestAdminToken123"
            ),
            "Rendered .env should contain the admin token"
        );
    }
}
