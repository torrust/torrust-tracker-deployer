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
use crate::domain::topology::EnabledServices;
use crate::domain::tracker::DatabaseConfig;
use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{
    DockerComposeContext, DockerComposeContextBuilder, MysqlSetupConfig, TrackerServiceContext,
};
use crate::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
use crate::infrastructure::templating::docker_compose::{
    DockerComposeProjectGenerator, DockerComposeProjectGeneratorError,
};
use crate::infrastructure::templating::TemplateMetadata;
use crate::shared::clock::Clock;
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
    clock: Arc<dyn Clock>,
}

impl<S> RenderDockerComposeTemplatesStep<S> {
    /// Creates a new `RenderDockerComposeTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `template_manager` - The template manager for accessing templates
    /// * `build_dir` - The build directory where templates will be rendered
    /// * `clock` - Clock service for generating template metadata timestamps
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        template_manager: Arc<TemplateManager>,
        build_dir: PathBuf,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment,
            template_manager,
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
    pub async fn execute(&self) -> Result<PathBuf, DockerComposeProjectGeneratorError> {
        info!(
            step = "render_docker_compose_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        let generator = DockerComposeProjectGenerator::new(&self.build_dir, &self.template_manager);

        let admin_token = self.extract_admin_token();
        let tracker = self.build_tracker_config();

        // Create contexts based on database configuration
        let database_config = self.environment.database_config();
        let (env_context, builder) = match database_config {
            DatabaseConfig::Sqlite(..) => self.create_sqlite_contexts(admin_token, tracker),
            DatabaseConfig::Mysql(mysql_config) => self.create_mysql_contexts(
                admin_token,
                tracker,
                mysql_config.port(),
                mysql_config.database_name().to_string(),
                mysql_config.username().to_string(),
                mysql_config.password().expose_secret().to_string(),
            ),
        };

        // Apply Prometheus configuration (independent of database choice)
        let builder = self.apply_prometheus_config(builder);

        // Apply Grafana configuration (independent of database choice)
        let builder = self.apply_grafana_config(builder);

        // Apply Caddy configuration (if HTTPS enabled)
        let builder = self.apply_caddy_config(builder);
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

    fn build_tracker_config(&self) -> TrackerServiceContext {
        let tracker_config = self.environment.tracker_config();

        // Determine which features are enabled (affects tracker networks)
        let has_prometheus = self.environment.prometheus_config().is_some();
        let has_mysql = matches!(
            self.environment.database_config(),
            DatabaseConfig::Mysql(..)
        );
        let has_caddy = self.has_caddy_enabled();
        let has_grafana = self.environment.grafana_config().is_some();

        // Build list of enabled services for topology context
        let mut enabled_services = Vec::new();
        if has_prometheus {
            enabled_services.push(crate::domain::topology::Service::Prometheus);
        }
        if has_grafana {
            enabled_services.push(crate::domain::topology::Service::Grafana);
        }
        if has_mysql {
            enabled_services.push(crate::domain::topology::Service::MySQL);
        }
        if has_caddy {
            enabled_services.push(crate::domain::topology::Service::Caddy);
        }

        let topology_context = EnabledServices::from(&enabled_services);

        TrackerServiceContext::from_domain_config(tracker_config, &topology_context)
    }

    /// Check if Caddy is enabled (HTTPS with at least one TLS-configured service)
    fn has_caddy_enabled(&self) -> bool {
        let user_inputs = &self.environment.context().user_inputs;

        // Check if HTTPS is configured
        if user_inputs.https().is_none() {
            return false;
        }

        let tracker = user_inputs.tracker();

        // Check if any service has TLS configured
        let tracker_api_has_tls = tracker.http_api_tls_domain().is_some();
        let http_trackers_have_tls = !tracker.http_trackers_with_tls().is_empty();
        let grafana_has_tls = user_inputs
            .grafana()
            .is_some_and(|g| g.tls_domain().is_some());

        // Caddy is enabled if HTTPS is configured AND at least one service has TLS
        tracker_api_has_tls || http_trackers_have_tls || grafana_has_tls
    }

    fn create_sqlite_contexts(
        &self,
        admin_token: String,
        tracker: TrackerServiceContext,
    ) -> (EnvContext, DockerComposeContextBuilder) {
        let metadata = TemplateMetadata::new(self.clock.now());
        let env_context = EnvContext::new(admin_token);
        let builder = DockerComposeContext::builder(tracker).with_metadata(metadata);

        (env_context, builder)
    }

    fn create_mysql_contexts(
        &self,
        admin_token: String,
        tracker: TrackerServiceContext,
        port: u16,
        database_name: String,
        username: String,
        password: PlainPassword,
    ) -> (EnvContext, DockerComposeContextBuilder) {
        // For MySQL, generate a secure root password (in production, this should be managed securely)
        let root_password = format!("{password}_root");

        let metadata = TemplateMetadata::new(self.clock.now());
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

        let builder = DockerComposeContext::builder(tracker)
            .with_metadata(metadata)
            .with_mysql(mysql_config);

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

    fn apply_caddy_config(
        &self,
        builder: DockerComposeContextBuilder,
    ) -> DockerComposeContextBuilder {
        let user_inputs = &self.environment.context().user_inputs;

        // Check if HTTPS is configured
        let Some(https_config) = user_inputs.https() else {
            return builder;
        };

        let tracker = user_inputs.tracker();

        // Check if any service has TLS configured
        let has_tracker_api_tls = tracker.http_api_tls_domain().is_some();
        let has_http_tracker_tls = !tracker.http_trackers_with_tls().is_empty();
        let has_grafana_tls = user_inputs
            .grafana()
            .is_some_and(|g| g.tls_domain().is_some());

        let has_any_tls = has_tracker_api_tls || has_http_tracker_tls || has_grafana_tls;

        // Note: The CaddyContext with full service details is built separately
        // in caddy_templates.rs for the Caddyfile.tera template. The docker-compose
        // template only needs to know if Caddy is enabled, not the service details.
        let _ = https_config; // Silence unused warning - admin_email/use_staging used in caddy_templates.rs

        // Only add Caddy if at least one service has TLS
        if has_any_tls {
            builder.with_caddy()
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

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment.clone(),
            template_manager.clone(),
            build_dir.path().to_path_buf(),
            clock,
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
        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            template_manager,
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

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let clock = Arc::new(SystemClock);
        let step = RenderDockerComposeTemplatesStep::new(
            environment,
            template_manager,
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
