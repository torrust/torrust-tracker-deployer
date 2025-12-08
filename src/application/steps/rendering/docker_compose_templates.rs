//! Docker Compose template rendering step
//!
//! This module provides the `RenderDockerComposeTemplatesStep` which handles rendering
//! of Docker Compose configuration templates to the build directory. This step prepares
//! Docker Compose files for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Docker Compose configurations
//! - Integration with the `DockerComposeTemplateRenderer` for file generation
//! - Build directory preparation for deployment operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the Docker Compose files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderDockerComposeTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::docker_compose::{
    DockerComposeTemplateError, DockerComposeTemplateRenderer,
};

/// Step that renders Docker Compose templates to the build directory
///
/// This step handles the preparation of Docker Compose configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host by the `DeployComposeFilesStep`.
pub struct RenderDockerComposeTemplatesStep {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl RenderDockerComposeTemplatesStep {
    /// Creates a new `RenderDockerComposeTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager for accessing templates
    /// * `build_dir` - The build directory where templates will be rendered
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>, build_dir: PathBuf) -> Self {
        Self {
            template_manager,
            build_dir,
        }
    }

    /// Execute the template rendering step
    ///
    /// This will render Docker Compose templates to the build directory.
    ///
    /// # Returns
    ///
    /// Returns the path to the docker-compose build directory on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File copying fails
    #[instrument(
        name = "render_docker_compose_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "docker_compose",
            build_dir = %self.build_dir.display()
        )
    )]
    pub async fn execute(&self) -> Result<PathBuf, DockerComposeTemplateError> {
        info!(
            step = "render_docker_compose_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        let renderer =
            DockerComposeTemplateRenderer::new(self.template_manager.clone(), &self.build_dir);

        let compose_build_dir = renderer.render().await?;

        info!(
            step = "render_docker_compose_templates",
            compose_build_dir = %compose_build_dir.display(),
            status = "success",
            "Docker Compose templates rendered successfully"
        );

        Ok(compose_build_dir)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::docker_compose::DOCKER_COMPOSE_SUBFOLDER;

    #[tokio::test]
    async fn it_should_create_render_docker_compose_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderDockerComposeTemplatesStep::new(
            template_manager.clone(),
            build_dir.path().to_path_buf(),
        );

        assert_eq!(step.build_dir, build_dir.path());
        assert_eq!(step.template_manager.templates_dir(), templates_dir.path());
    }

    #[tokio::test]
    async fn it_should_render_templates_from_embedded_sources() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step =
            RenderDockerComposeTemplatesStep::new(template_manager, build_dir.path().to_path_buf());

        let result = step.execute().await;

        assert!(result.is_ok());
        let compose_build_dir = result.unwrap();
        assert!(compose_build_dir.join("docker-compose.yml").exists());
    }

    #[tokio::test]
    async fn it_should_render_correct_content() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step =
            RenderDockerComposeTemplatesStep::new(template_manager, build_dir.path().to_path_buf());

        let result = step.execute().await;
        assert!(result.is_ok());

        let output_content = tokio::fs::read_to_string(
            build_dir
                .path()
                .join(DOCKER_COMPOSE_SUBFOLDER)
                .join("docker-compose.yml"),
        )
        .await
        .expect("Failed to read output");

        // Verify it contains expected content from embedded template
        assert!(output_content.contains("nginx:alpine"));
        assert!(output_content.contains("demo-app"));
    }
}
