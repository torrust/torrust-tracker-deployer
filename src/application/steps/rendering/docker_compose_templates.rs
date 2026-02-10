//! Docker Compose template rendering step
//!
//! This module provides the `RenderDockerComposeTemplatesStep` which handles rendering
//! of Docker Compose configuration templates to the build directory. This step prepares
//! Docker Compose files for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Docker Compose configurations
//! - Integration with the `DockerComposeTemplateRenderingService` for file generation
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

use crate::application::services::rendering::DockerComposeTemplateRenderingService;
use crate::application::services::rendering::DockerComposeTemplateRenderingServiceError;
use crate::domain::environment::Environment;
use crate::shared::clock::Clock;

/// Step that renders Docker Compose templates to the build directory
///
/// This step handles the preparation of Docker Compose configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host by the `DeployComposeFilesStep`.
pub struct RenderDockerComposeTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl<S> RenderDockerComposeTemplatesStep<S> {
    /// Creates a new `RenderDockerComposeTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `templates_dir` - The templates directory
    /// * `build_dir` - The build directory where templates will be rendered
    /// * `clock` - Clock service for generating template metadata timestamps
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        templates_dir: PathBuf,
        build_dir: PathBuf,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment,
            templates_dir,
            build_dir,
            clock,
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
    pub async fn execute(&self) -> Result<PathBuf, DockerComposeTemplateRenderingServiceError> {
        info!(
            step = "render_docker_compose_templates",
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        let service = DockerComposeTemplateRenderingService::from_paths(
            self.templates_dir.clone(),
            self.build_dir.clone(),
            self.clock.clone(),
        );

        let user_inputs = &self.environment.context().user_inputs;
        let admin_token = self.environment.admin_token();
        let compose_build_dir = service.render(user_inputs, admin_token).await?;

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
    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::infrastructure::templating::docker_compose::DOCKER_COMPOSE_SUBFOLDER;
    use crate::shared::clock::SystemClock;

    #[tokio::test]
    async fn it_should_create_render_docker_compose_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment.clone(),
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        assert_eq!(step.build_dir, build_dir.path());
        assert_eq!(step.templates_dir, templates_dir.path());
    }

    #[tokio::test]
    async fn it_should_render_templates_from_embedded_sources() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        let result = step.execute().await;

        assert!(result.is_ok());
        let compose_build_dir = result.unwrap();
        assert!(compose_build_dir.join("docker-compose.yml").exists());
    }

    #[tokio::test]
    async fn it_should_render_correct_content() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

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
        assert!(output_content.contains("torrust/tracker"));
        assert!(output_content.contains("./storage/tracker/lib:/var/lib/torrust/tracker"));

        // Verify dynamic ports are rendered (default TrackerConfig has 6969 UDP, 7070 HTTP, 1212 API)
        assert!(
            output_content.contains("6969:6969/udp"),
            "Should contain UDP tracker port 6969"
        );
        assert!(
            output_content.contains("7070:7070"),
            "Should contain HTTP tracker port 7070"
        );
        assert!(
            output_content.contains("1212:1212"),
            "Should contain HTTP API port 1212"
        );

        // Verify hardcoded ports are NOT present
        assert!(
            !output_content.contains("6868:6868"),
            "Should not contain hardcoded UDP port 6868"
        );
    }
}
