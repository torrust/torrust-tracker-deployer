//! Docker Compose template rendering step
//!
//! This module provides the `RenderDockerComposeTemplatesStep` which handles rendering
//! of Docker Compose configuration templates to the build directory. This step prepares
//! Docker Compose files for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Docker Compose configurations
//! - Integration with the `DockerComposeProjectGenerator` for file generation
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

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::domain::tracker::{DatabaseConfig, TrackerConfig};
use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{
    DockerComposeContext, DockerComposeContextBuilder, MysqlSetupConfig, TrackerPorts,
};
use crate::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
use crate::infrastructure::templating::docker_compose::{
    DockerComposeProjectGenerator, DockerComposeProjectGeneratorError,
};
use crate::shared::PlainPassword;

/// Step that renders Docker Compose templates to the build directory
///
/// This step handles the preparation of Docker Compose configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host by the `DeployComposeFilesStep`.
pub struct RenderDockerComposeTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderDockerComposeTemplatesStep<S> {
    /// Creates a new `RenderDockerComposeTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `template_manager` - The template manager for accessing templates
    /// * `build_dir` - The build directory where templates will be rendered
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        template_manager: Arc<TemplateManager>,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            environment,
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
    pub async fn execute(&self) -> Result<PathBuf, DockerComposeProjectGeneratorError> {
        info!(
            step = "render_docker_compose_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        let generator = DockerComposeProjectGenerator::new(&self.build_dir, &self.template_manager);

        let admin_token = self.extract_admin_token();
        let ports = self.build_tracker_ports();

        // Create contexts based on database configuration
        let database_config = self.environment.database_config();
        let (env_context, builder) = match database_config {
            DatabaseConfig::Sqlite(..) => Self::create_sqlite_contexts(admin_token, ports),
            DatabaseConfig::Mysql(mysql_config) => Self::create_mysql_contexts(
                admin_token,
                ports,
                mysql_config.port,
                mysql_config.database_name.clone(),
                mysql_config.username.clone(),
                mysql_config.password.expose_secret().to_string(),
            ),
        };

        // Apply Prometheus configuration (independent of database choice)
        let builder = self.apply_prometheus_config(builder);

        // Apply Grafana configuration (independent of database choice)
        let builder = self.apply_grafana_config(builder);
        let docker_compose_context = builder.build();

        // Apply Grafana credentials to env context
        let env_context = self.apply_grafana_env_context(env_context);

        let compose_build_dir = generator
            .render(&env_context, &docker_compose_context)
            .await?;

        info!(
            step = "render_docker_compose_templates",
            compose_build_dir = %compose_build_dir.display(),
            status = "success",
            "Docker Compose templates rendered successfully"
        );

        Ok(compose_build_dir)
    }

    fn extract_admin_token(&self) -> String {
        self.environment.admin_token().to_string()
    }

    fn build_tracker_ports(&self) -> TrackerPorts {
        let tracker_config = self.environment.tracker_config();
        let (udp_tracker_ports, http_tracker_ports, http_api_port) =
            Self::extract_tracker_ports(tracker_config);

        TrackerPorts {
            udp_tracker_ports,
            http_tracker_ports,
            http_api_port,
        }
    }

    fn create_sqlite_contexts(
        admin_token: String,
        ports: TrackerPorts,
    ) -> (EnvContext, DockerComposeContextBuilder) {
        let env_context = EnvContext::new(admin_token);
        let builder = DockerComposeContext::builder(ports);

        (env_context, builder)
    }

    fn create_mysql_contexts(
        admin_token: String,
        ports: TrackerPorts,
        port: u16,
        database_name: String,
        username: String,
        password: PlainPassword,
    ) -> (EnvContext, DockerComposeContextBuilder) {
        // For MySQL, generate a secure root password (in production, this should be managed securely)
        let root_password = format!("{password}_root");

        let env_context = EnvContext::new_with_mysql(
            admin_token,
            root_password.clone(),
            database_name.clone(),
            username.clone(),
            password.clone(),
        );

        let mysql_config = MysqlSetupConfig {
            root_password,
            database: database_name,
            user: username,
            password,
            port,
        };

        let builder = DockerComposeContext::builder(ports).with_mysql(mysql_config);

        (env_context, builder)
    }

    fn apply_prometheus_config(
        &self,
        builder: DockerComposeContextBuilder,
    ) -> DockerComposeContextBuilder {
        if let Some(prometheus_config) = self.environment.prometheus_config() {
            builder.with_prometheus(prometheus_config.clone())
        } else {
            builder
        }
    }

    fn apply_grafana_config(
        &self,
        builder: DockerComposeContextBuilder,
    ) -> DockerComposeContextBuilder {
        if let Some(grafana_config) = self.environment.grafana_config() {
            builder.with_grafana(grafana_config.clone())
        } else {
            builder
        }
    }

    fn apply_grafana_env_context(&self, env_context: EnvContext) -> EnvContext {
        if let Some(grafana_config) = self.environment.grafana_config() {
            env_context.with_grafana(
                grafana_config.admin_user().to_string(),
                grafana_config.admin_password().expose_secret().to_string(),
            )
        } else {
            env_context
        }
    }

    fn extract_tracker_ports(tracker_config: &TrackerConfig) -> (Vec<u16>, Vec<u16>, u16) {
        // Extract UDP tracker ports
        let udp_ports: Vec<u16> = tracker_config
            .udp_trackers
            .iter()
            .map(|tracker| tracker.bind_address.port())
            .collect();

        // Extract HTTP tracker ports
        let http_ports: Vec<u16> = tracker_config
            .http_trackers
            .iter()
            .map(|tracker| tracker.bind_address.port())
            .collect();

        // Extract HTTP API port
        let api_port = tracker_config.http_api.bind_address.port();

        (udp_ports, http_ports, api_port)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::infrastructure::templating::docker_compose::DOCKER_COMPOSE_SUBFOLDER;

    #[tokio::test]
    async fn it_should_create_render_docker_compose_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderDockerComposeTemplatesStep::new(
            environment.clone(),
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

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
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

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
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
