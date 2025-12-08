//! Docker Compose Project Generator
//!
//! This module handles Docker Compose template rendering for deployment workflows.
//! It manages the creation of build directories, copying static template files (docker-compose.yml),
//! and processing dynamic Tera templates with runtime variables (.env).
//!
//! ## Key Features
//!
//! - **Static file copying**: Handles Docker Compose files that don't need templating
//! - **Dynamic template rendering**: Processes Tera templates with runtime variables
//! - **Structured error handling**: Provides specific error types with detailed context and source chaining
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring deployment processes
//! - **Testable design**: Modular structure that allows for comprehensive unit testing

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::docker_compose::template::renderer::env::{
    EnvRenderer, EnvRendererError,
};
use crate::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;

/// Errors that can occur during Docker Compose project generation
#[derive(Error, Debug)]
pub enum DockerComposeProjectGeneratorError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to get template path from template manager
    #[error("Failed to get template path for '{file_name}': {source}")]
    TemplatePathFailed {
        file_name: String,
        #[source]
        source: TemplateManagerError,
    },

    /// Failed to copy static template file
    #[error("Failed to copy static template file '{file_name}' to build directory: {source}")]
    StaticFileCopyFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to render .env template using renderer
    #[error("Failed to render .env template: {source}")]
    EnvRenderingFailed {
        #[source]
        source: EnvRendererError,
    },
}

/// Renders Docker Compose templates to a build directory
///
/// This collaborator is responsible for preparing Docker Compose templates for deployment workflows.
/// It handles both static files (docker-compose.yml) and dynamic Tera templates that
/// require runtime variable substitution (.env with environment variables).
pub struct DockerComposeProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    env_renderer: EnvRenderer,
}

impl DockerComposeProjectGenerator {
    /// Default relative path for Docker Compose configuration files
    const DOCKER_COMPOSE_BUILD_PATH: &'static str = "docker-compose";

    /// Default template path prefix for Docker Compose templates
    const DOCKER_COMPOSE_TEMPLATE_PATH: &'static str = "docker-compose";

    /// Creates a new Docker Compose project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let env_renderer = EnvRenderer::new(template_manager.clone());

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            template_manager,
            env_renderer,
        }
    }

    /// Renders Docker Compose templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Docker Compose
    /// 2. Renders dynamic Tera templates with runtime variables (.env)
    /// 3. Copies static templates (docker-compose.yml) from the template manager
    /// 4. Provides debug logging via the tracing crate
    ///
    /// # Arguments
    ///
    /// * `env_context` - Runtime context for .env template rendering (tracker admin token, etc.)
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, DockerComposeProjectGeneratorError>` - Build directory path or error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Template copying fails
    /// - Template manager cannot provide required templates
    /// - Dynamic template rendering fails
    /// - Runtime variable substitution fails
    pub async fn render(
        &self,
        env_context: &EnvContext,
    ) -> Result<PathBuf, DockerComposeProjectGeneratorError> {
        tracing::info!(
            template_type = "docker_compose",
            "Rendering Docker Compose templates"
        );

        // Create build directory structure
        let build_compose_dir = self.create_build_directory().await?;

        // Render dynamic .env template with runtime variables using renderer
        self.env_renderer
            .render(env_context, &build_compose_dir)
            .map_err(|source| DockerComposeProjectGeneratorError::EnvRenderingFailed { source })?;

        // Copy static Docker Compose files
        self.copy_static_templates(&self.template_manager, &build_compose_dir)
            .await?;

        tracing::debug!(
            template_type = "docker_compose",
            output_dir = %build_compose_dir.display(),
            "Docker Compose templates rendered"
        );

        tracing::info!(
            template_type = "docker_compose",
            status = "complete",
            "Docker Compose templates ready"
        );

        Ok(build_compose_dir)
    }

    /// Builds the full Docker Compose build directory path
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The complete path to the Docker Compose build directory
    fn build_compose_directory(&self) -> PathBuf {
        self.build_dir.join(Self::DOCKER_COMPOSE_BUILD_PATH)
    }

    /// Builds the template path for a specific file in the Docker Compose template directory
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the template file
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for the specified file
    fn build_template_path(file_name: &str) -> String {
        format!("{}/{file_name}", Self::DOCKER_COMPOSE_TEMPLATE_PATH)
    }

    /// Creates the Docker Compose build directory structure
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, DockerComposeProjectGeneratorError>` - The created build directory path or an error
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    async fn create_build_directory(&self) -> Result<PathBuf, DockerComposeProjectGeneratorError> {
        let build_compose_dir = self.build_compose_directory();
        tokio::fs::create_dir_all(&build_compose_dir)
            .await
            .map_err(
                |source| DockerComposeProjectGeneratorError::DirectoryCreationFailed {
                    directory: build_compose_dir.display().to_string(),
                    source,
                },
            )?;
        Ok(build_compose_dir)
    }

    /// Copies static Docker Compose template files that don't require variable substitution
    ///
    /// This includes docker-compose.yml that uses native Docker Compose variable substitution
    /// from the .env file.
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Source of template files
    /// * `destination_dir` - Directory where static files will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), DockerComposeProjectGeneratorError>` - Success or error from file copying operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide required template paths
    /// - File copying fails for any of the specified files
    async fn copy_static_templates(
        &self,
        template_manager: &TemplateManager,
        destination_dir: &Path,
    ) -> Result<(), DockerComposeProjectGeneratorError> {
        tracing::debug!("Copying static Docker Compose template files");

        // Copy docker-compose.yml
        self.copy_static_file(template_manager, "docker-compose.yml", destination_dir)
            .await?;

        tracing::debug!("Successfully copied 1 static template file");

        Ok(())
    }

    /// Copies a single static template file from template manager to destination
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Source of template files
    /// * `file_name` - Name of the file to copy (without path prefix)
    /// * `destination_dir` - Directory where the file will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), DockerComposeProjectGeneratorError>` - Success or error from the file copying operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide the template path
    /// - File copying fails
    async fn copy_static_file(
        &self,
        template_manager: &TemplateManager,
        file_name: &str,
        destination_dir: &Path,
    ) -> Result<(), DockerComposeProjectGeneratorError> {
        let template_path = Self::build_template_path(file_name);

        let source_path = template_manager
            .get_template_path(&template_path)
            .map_err(
                |source| DockerComposeProjectGeneratorError::TemplatePathFailed {
                    file_name: file_name.to_string(),
                    source,
                },
            )?;

        let destination_path = destination_dir.join(file_name);

        tokio::fs::copy(&source_path, &destination_path)
            .await
            .map_err(
                |source| DockerComposeProjectGeneratorError::StaticFileCopyFailed {
                    file_name: file_name.to_string(),
                    source,
                },
            )?;

        tracing::trace!(
            file = file_name,
            source = %source_path.display(),
            destination = %destination_path.display(),
            "Copied static template file"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::docker_compose::DOCKER_COMPOSE_SUBFOLDER;

    /// Creates a `TemplateManager` that uses the embedded templates
    ///
    /// This tests the real integration with embedded templates by creating
    /// a `TemplateManager` pointing to a temp directory where templates
    /// will be extracted on-demand.
    fn create_template_manager_with_embedded() -> (Arc<TemplateManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = Arc::new(TemplateManager::new(temp_dir.path()));
        (manager, temp_dir)
    }

    /// Helper function to create a test .env context
    fn create_test_env_context() -> EnvContext {
        EnvContext::new("TestAdminToken123".to_string())
    }

    #[tokio::test]
    async fn test_project_generator_creates_build_directory() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), template_manager);
        let env_context = create_test_env_context();

        let result = generator.render(&env_context).await;

        assert!(result.is_ok());
        let compose_dir = build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER);
        assert!(compose_dir.exists());
        assert!(compose_dir.is_dir());
    }

    #[tokio::test]
    async fn test_project_generator_copies_docker_compose_yml() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), template_manager);
        let env_context = create_test_env_context();

        generator
            .render(&env_context)
            .await
            .expect("Failed to render templates");

        let compose_file = build_dir
            .path()
            .join(DOCKER_COMPOSE_SUBFOLDER)
            .join("docker-compose.yml");
        assert!(compose_file.exists());
        assert!(compose_file.is_file());
    }

    #[tokio::test]
    async fn test_project_generator_renders_env_file() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), template_manager);
        let env_context = create_test_env_context();

        generator
            .render(&env_context)
            .await
            .expect("Failed to render templates");

        let env_file = build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER).join(".env");
        assert!(env_file.exists());
        assert!(env_file.is_file());

        // Verify content contains the admin token
        let content = std::fs::read_to_string(&env_file).expect("Failed to read .env file");
        assert!(content.contains("TestAdminToken123"));
    }

    #[tokio::test]
    async fn test_project_generator_returns_build_directory_path() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), template_manager);
        let env_context = create_test_env_context();

        let result = generator.render(&env_context).await;

        assert!(result.is_ok());
        let returned_path = result.unwrap();
        assert_eq!(
            returned_path,
            build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER)
        );
    }
}
