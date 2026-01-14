//! Docker Compose Project Generator
//!
//! This module handles Docker Compose template rendering for deployment workflows.
//! It manages the creation of build directories and processing dynamic Tera templates
//! with runtime variables (.env and docker-compose.yml).
//!
//! ## Key Features
//!
//! - **Dynamic template rendering**: Processes Tera templates with runtime variables
//! - **Structured error handling**: Provides specific error types with detailed context and source chaining
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring deployment processes
//! - **Testable design**: Modular structure that allows for comprehensive unit testing

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::docker_compose::template::renderer::docker_compose::{
    DockerComposeRenderer, DockerComposeRendererError,
};
use crate::infrastructure::templating::docker_compose::template::renderer::env::{
    EnvRenderer, EnvRendererError,
};
use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::DockerComposeContext;
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

    /// Failed to render .env template using renderer
    #[error("Failed to render .env template: {source}")]
    EnvRenderingFailed {
        #[source]
        source: EnvRendererError,
    },

    /// Failed to render docker-compose.yml template using renderer
    #[error("Failed to render docker-compose.yml template: {source}")]
    DockerComposeRenderingFailed {
        #[source]
        source: DockerComposeRendererError,
    },
}

/// Renders Docker Compose templates to a build directory
///
/// This collaborator is responsible for preparing Docker Compose templates for deployment workflows.
/// It handles dynamic Tera templates that require runtime variable substitution
/// (.env with environment variables and docker-compose.yml with service configurations).
pub struct DockerComposeProjectGenerator {
    build_dir: PathBuf,
    env_renderer: EnvRenderer,
    docker_compose_renderer: DockerComposeRenderer,
}

impl DockerComposeProjectGenerator {
    /// Default relative path for Docker Compose configuration files
    const DOCKER_COMPOSE_BUILD_PATH: &'static str = "docker-compose";

    /// Creates a new Docker Compose project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: &Arc<TemplateManager>) -> Self {
        let env_renderer = EnvRenderer::new(template_manager.clone());
        let docker_compose_renderer = DockerComposeRenderer::new(template_manager.clone());

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            env_renderer,
            docker_compose_renderer,
        }
    }

    /// Renders Docker Compose templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Docker Compose
    /// 2. Renders dynamic Tera templates with runtime variables (.env and docker-compose.yml)
    /// 3. Provides debug logging via the tracing crate
    ///
    /// # Arguments
    ///
    /// * `env_context` - Runtime context for .env template rendering (tracker admin token, etc.)
    /// * `docker_compose_context` - Runtime context for docker-compose.yml template rendering (database configuration, etc.)
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, DockerComposeProjectGeneratorError>` - Build directory path or error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Dynamic template rendering fails
    /// - Runtime variable substitution fails
    pub async fn render(
        &self,
        env_context: &EnvContext,
        docker_compose_context: &DockerComposeContext,
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

        // Render dynamic docker-compose.yml template with runtime variables using renderer
        self.docker_compose_renderer
            .render(docker_compose_context, &build_compose_dir)
            .map_err(
                |source| DockerComposeProjectGeneratorError::DockerComposeRenderingFailed {
                    source,
                },
            )?;

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
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::TrackerPorts;
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

    /// Helper function to create a test docker-compose context with `SQLite`
    fn create_test_docker_compose_context_sqlite() -> DockerComposeContext {
        // Use default test ports (matching TrackerConfig::default())
        let ports = TrackerPorts::new(
            vec![6969], // UDP ports
            vec![7070], // HTTP ports without TLS
            1212,       // API port
            false,      // API has no TLS
        );
        DockerComposeContext::builder(ports).build()
    }

    #[tokio::test]
    async fn it_should_create_build_directory_when_generating_project() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), &template_manager);
        let env_context = create_test_env_context();
        let docker_compose_context = create_test_docker_compose_context_sqlite();

        let result = generator
            .render(&env_context, &docker_compose_context)
            .await;

        assert!(result.is_ok());
        let compose_dir = build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER);
        assert!(compose_dir.exists());
        assert!(compose_dir.is_dir());
    }

    #[tokio::test]
    async fn it_should_render_docker_compose_yml_when_generating_project() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), &template_manager);
        let env_context = create_test_env_context();
        let docker_compose_context = create_test_docker_compose_context_sqlite();

        generator
            .render(&env_context, &docker_compose_context)
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
    async fn it_should_render_env_file_when_generating_project() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), &template_manager);
        let env_context = create_test_env_context();
        let docker_compose_context = create_test_docker_compose_context_sqlite();

        generator
            .render(&env_context, &docker_compose_context)
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
    async fn it_should_return_build_directory_path_when_project_generated() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");

        let generator = DockerComposeProjectGenerator::new(build_dir.path(), &template_manager);
        let env_context = create_test_env_context();
        let docker_compose_context = create_test_docker_compose_context_sqlite();

        let result = generator
            .render(&env_context, &docker_compose_context)
            .await;

        assert!(result.is_ok());
        let returned_path = result.unwrap();
        assert_eq!(
            returned_path,
            build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER)
        );
    }
}
