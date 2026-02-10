//! Docker Compose template rendering service
//!
//! This service handles rendering of Docker Compose configuration templates,
//! including complex context building for database variants (SQLite/MySQL),
//! topology computation, and optional service configuration.

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::topology::EnabledServices;
use crate::domain::tracker::DatabaseConfig;
use crate::domain::TemplateManager;
use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{
    DockerComposeContext, DockerComposeContextBuilder, MysqlSetupConfig, TrackerServiceContext,
};
use crate::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
use crate::infrastructure::templating::docker_compose::{
    DockerComposeProjectGenerator, DockerComposeProjectGeneratorError,
};
use crate::infrastructure::templating::TemplateMetadata;
use crate::shared::{Clock, PlainPassword};

use crate::domain::environment::user_inputs::UserInputs;

/// Service for rendering Docker Compose templates
///
/// This service encapsulates the complex logic for building Docker Compose
/// contexts including:
/// - Database variant selection (`SQLite` vs `MySQL`)
/// - Topology computation (which services are enabled)
/// - Optional service configuration (Prometheus, Grafana, Backup, Caddy)
/// - Grafana environment context
/// - `MySQL` setup configuration
pub struct DockerComposeTemplateRenderingService {
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl DockerComposeTemplateRenderingService {
    /// Create a new service with explicit dependencies
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing template source files
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - Clock service for timestamps
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        Self {
            templates_dir,
            build_dir,
            clock,
        }
    }

    /// Render Docker Compose templates with full context building
    ///
    /// This method builds the complete Docker Compose context from user inputs,
    /// including database-specific configuration, topology computation, and
    /// optional service integration.
    ///
    /// # Arguments
    ///
    /// * `user_inputs` - Complete user configuration
    /// * `admin_token` - Tracker admin token
    ///
    /// # Returns
    ///
    /// Path to the rendered docker-compose build directory
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails
    #[instrument(
        name = "docker_compose_rendering_service",
        skip_all,
        fields(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display()
        )
    )]
    pub async fn render(
        &self,
        user_inputs: &UserInputs,
        admin_token: &str,
    ) -> Result<PathBuf, DockerComposeTemplateRenderingServiceError> {
        info!(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        let template_manager = Arc::new(TemplateManager::new(self.templates_dir.clone()));
        let generator = DockerComposeProjectGenerator::new(&self.build_dir, &template_manager);

        let tracker = Self::build_tracker_config(user_inputs);
        let database_config = user_inputs.tracker().core().database();

        // Create contexts based on database configuration
        let (env_context, builder) = match database_config {
            DatabaseConfig::Sqlite(..) => {
                self.create_sqlite_contexts(admin_token.to_string(), tracker)
            }
            DatabaseConfig::Mysql(mysql_config) => self.create_mysql_contexts(
                admin_token.to_string(),
                tracker,
                mysql_config.port(),
                mysql_config.database_name().to_string(),
                mysql_config.username().to_string(),
                mysql_config.password().expose_secret().to_string(),
            ),
        };

        // Apply optional service configurations
        let builder = Self::apply_prometheus_config(builder, user_inputs);
        let builder = Self::apply_grafana_config(builder, user_inputs);
        let builder = Self::apply_backup_config(builder, user_inputs);
        let builder = Self::apply_caddy_config(builder, user_inputs);

        let docker_compose_context = builder.build();

        // Apply Grafana credentials to env context
        let env_context = Self::apply_grafana_env_context(env_context, user_inputs);

        let compose_build_dir = generator
            .render(&env_context, &docker_compose_context)
            .await
            .map_err(DockerComposeTemplateRenderingServiceError::RenderingFailed)?;

        info!(
            compose_build_dir = %compose_build_dir.display(),
            "Docker Compose templates rendered successfully"
        );

        Ok(compose_build_dir)
    }

    /// Build tracker service context with topology information
    ///
    /// Determines which services are enabled and builds the complete
    /// tracker context including network configuration.
    fn build_tracker_config(user_inputs: &UserInputs) -> TrackerServiceContext {
        let tracker_config = user_inputs.tracker();

        // Determine which features are enabled (affects tracker networks)
        let has_prometheus = user_inputs.prometheus().is_some();
        let has_mysql = matches!(
            user_inputs.tracker().core().database(),
            DatabaseConfig::Mysql(..)
        );
        let has_caddy = Self::has_caddy_enabled(user_inputs);
        let has_grafana = user_inputs.grafana().is_some();

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
    fn has_caddy_enabled(user_inputs: &UserInputs) -> bool {
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

    /// Create contexts for `SQLite` database configuration
    fn create_sqlite_contexts(
        &self,
        admin_token: String,
        tracker: TrackerServiceContext,
    ) -> (EnvContext, DockerComposeContextBuilder) {
        let metadata = TemplateMetadata::new(self.clock.now());
        let env_context = EnvContext::new(metadata.clone(), admin_token);
        let builder = DockerComposeContext::builder(tracker).with_metadata(metadata);

        (env_context, builder)
    }

    /// Create contexts for `MySQL` database configuration
    #[allow(clippy::too_many_arguments)]
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
            metadata.clone(),
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

    /// Apply Prometheus configuration if present
    fn apply_prometheus_config(
        builder: DockerComposeContextBuilder,
        user_inputs: &UserInputs,
    ) -> DockerComposeContextBuilder {
        if let Some(prometheus_config) = user_inputs.prometheus() {
            builder.with_prometheus(prometheus_config.clone())
        } else {
            builder
        }
    }

    /// Apply Grafana configuration if present
    fn apply_grafana_config(
        builder: DockerComposeContextBuilder,
        user_inputs: &UserInputs,
    ) -> DockerComposeContextBuilder {
        if let Some(grafana_config) = user_inputs.grafana() {
            builder.with_grafana(grafana_config.clone())
        } else {
            builder
        }
    }

    /// Apply Backup configuration if present
    fn apply_backup_config(
        builder: DockerComposeContextBuilder,
        user_inputs: &UserInputs,
    ) -> DockerComposeContextBuilder {
        if let Some(backup_config) = user_inputs.backup() {
            builder.with_backup(backup_config.clone())
        } else {
            builder
        }
    }

    /// Apply Caddy configuration if HTTPS and TLS services are configured
    fn apply_caddy_config(
        builder: DockerComposeContextBuilder,
        user_inputs: &UserInputs,
    ) -> DockerComposeContextBuilder {
        // Check if HTTPS is configured
        let Some(_https_config) = user_inputs.https() else {
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
        // in CaddyTemplateRenderingService. The docker-compose template only needs
        // to know if Caddy is enabled, not the service details.

        // Only add Caddy if at least one service has TLS
        if has_any_tls {
            builder.with_caddy()
        } else {
            builder
        }
    }

    /// Apply Grafana credentials to environment context if Grafana is configured
    fn apply_grafana_env_context(env_context: EnvContext, user_inputs: &UserInputs) -> EnvContext {
        if let Some(grafana_config) = user_inputs.grafana() {
            env_context.with_grafana(
                grafana_config.admin_user().to_string(),
                grafana_config.admin_password().expose_secret().to_string(),
            )
        } else {
            env_context
        }
    }
}

/// Errors that can occur during Docker Compose template rendering
#[derive(Debug, thiserror::Error)]
pub enum DockerComposeTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Docker Compose template rendering failed: {0}")]
    RenderingFailed(#[from] DockerComposeProjectGeneratorError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::shared::SystemClock;

    #[tokio::test]
    async fn it_should_create_service_with_from_paths() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let service = DockerComposeTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        assert_eq!(service.templates_dir, templates_dir.path());
        assert_eq!(service.build_dir, build_dir.path());
    }

    #[tokio::test]
    async fn it_should_render_docker_compose_templates_for_sqlite() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let service = DockerComposeTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let user_inputs = &environment.context().user_inputs;
        let admin_token = "test-admin-token";

        let result = service.render(user_inputs, admin_token).await;

        assert!(result.is_ok());
        let compose_dir = result.unwrap();
        assert!(compose_dir.exists());
        assert!(compose_dir.join("docker-compose.yml").exists());
    }

    #[tokio::test]
    async fn it_should_check_caddy_enabled_correctly() {
        // Test without HTTPS - should be false
        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let user_inputs_no_https = &environment.context().user_inputs;
        assert!(!DockerComposeTemplateRenderingService::has_caddy_enabled(
            user_inputs_no_https
        ));

        // TODO: Add test with HTTPS + TLS when EnvironmentTestBuilder supports it
    }
}
