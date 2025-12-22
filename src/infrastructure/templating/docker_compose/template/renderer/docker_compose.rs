//! # docker-compose.yml Template Renderer
//!
//! This module handles rendering of the `docker-compose.yml.tera` template for Docker Compose deployments.
//! It's responsible for creating `docker-compose.yml` files with service configurations from dynamic configuration.
//!
//! ## Responsibilities
//!
//! - Load the `docker-compose.yml.tera` template file
//! - Process template with runtime context (database configuration, etc.)
//! - Render final `docker-compose.yml` file for Docker Compose consumption

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{
    DockerComposeContext, DockerComposeTemplate,
};

/// Errors that can occur during docker-compose.yml template rendering
#[derive(Error, Debug)]
pub enum DockerComposeRendererError {
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

    /// Failed to create docker-compose template with provided context
    #[error("Failed to create DockerComposeTemplate: {source}")]
    DockerComposeTemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    /// Failed to render docker-compose template to output file
    #[error("Failed to render docker-compose.yml template to file: {source}")]
    DockerComposeTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Handles rendering of the docker-compose.yml.tera template for Docker Compose deployments
///
/// This collaborator is responsible for all docker-compose.yml template-specific operations:
/// - Loading the docker-compose.yml.tera template
/// - Processing it with runtime context (database configuration, etc.)
/// - Rendering the final docker-compose.yml file for Docker Compose consumption
pub struct DockerComposeRenderer {
    template_manager: Arc<TemplateManager>,
}

impl DockerComposeRenderer {
    /// Template filename for the docker-compose.yml Tera template
    const DOCKER_COMPOSE_TEMPLATE_FILE: &'static str = "docker-compose.yml.tera";

    /// Output filename for the rendered docker-compose.yml file
    const DOCKER_COMPOSE_OUTPUT_FILE: &'static str = "docker-compose.yml";

    /// Default template path prefix for Docker Compose templates
    const DOCKER_COMPOSE_TEMPLATE_PATH: &'static str = "docker-compose";

    /// Creates a new docker-compose.yml template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the docker-compose.yml.tera template with the provided context
    ///
    /// This method:
    /// 1. Loads the docker-compose.yml.tera template from the template manager
    /// 2. Reads the template content
    /// 3. Creates a File object for template processing
    /// 4. Creates a `DockerComposeTemplate` with the runtime context
    /// 5. Renders the template to docker-compose.yml in the output directory
    ///
    /// # Arguments
    ///
    /// * `context` - The context containing service configuration
    /// * `output_dir` - The directory where docker-compose.yml should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), DockerComposeRendererError>` - Success or error from the template rendering operation
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
        context: &DockerComposeContext,
        output_dir: &Path,
    ) -> Result<(), DockerComposeRendererError> {
        tracing::debug!("Rendering docker-compose.yml template with runtime variables");

        // Get the docker-compose.yml template path
        let template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| DockerComposeRendererError::TemplatePathFailed {
                file_name: Self::DOCKER_COMPOSE_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read the template file content
        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            DockerComposeRendererError::TeraTemplateReadFailed {
                file_name: Self::DOCKER_COMPOSE_TEMPLATE_FILE.to_string(),
                source,
            }
        })?;

        // Create File object for template processing
        let template_file = File::new(Self::DOCKER_COMPOSE_TEMPLATE_FILE, template_content)
            .map_err(|source| DockerComposeRendererError::FileCreationFailed {
                file_name: Self::DOCKER_COMPOSE_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Create the template with context
        let docker_compose_template = DockerComposeTemplate::new(&template_file, context.clone())
            .map_err(|source| {
            DockerComposeRendererError::DockerComposeTemplateCreationFailed { source }
        })?;

        // Render to output file
        let output_path = output_dir.join(Self::DOCKER_COMPOSE_OUTPUT_FILE);
        docker_compose_template
            .render(&output_path)
            .map_err(
                |source| DockerComposeRendererError::DockerComposeTemplateRenderFailed { source },
            )?;

        tracing::debug!(
            output_path = %output_path.display(),
            "docker-compose.yml template rendered successfully"
        );

        Ok(())
    }

    /// Builds the template path for the docker-compose.yml.tera file
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path
    fn build_template_path() -> String {
        format!(
            "{}/{}",
            Self::DOCKER_COMPOSE_TEMPLATE_PATH,
            Self::DOCKER_COMPOSE_TEMPLATE_FILE
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{
        DockerComposeContext, MysqlSetupConfig, TrackerPorts,
    };

    #[test]
    fn it_should_create_renderer_with_template_manager() {
        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let renderer = DockerComposeRenderer::new(template_manager);

        // Verify the renderer is created (smoke test)
        assert!(std::mem::size_of_val(&renderer) > 0);
    }

    #[test]
    fn it_should_build_correct_template_path() {
        let path = DockerComposeRenderer::build_template_path();
        assert_eq!(path, "docker-compose/docker-compose.yml.tera");
    }

    #[test]
    fn it_should_render_docker_compose_with_mysql_service_when_driver_is_mysql() {
        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let mysql_config = MysqlSetupConfig {
            root_password: "rootpass123".to_string(),
            database: "tracker_db".to_string(),
            user: "tracker_user".to_string(),
            password: "userpass123".to_string(),
            port: 3306,
        };
        let mysql_context = DockerComposeContext::builder(ports)
            .with_mysql(mysql_config)
            .build();

        let renderer = DockerComposeRenderer::new(template_manager);
        let output_dir = TempDir::new().unwrap();

        let result = renderer.render(&mysql_context, output_dir.path());
        assert!(
            result.is_ok(),
            "Rendering with MySQL context should succeed"
        );

        let output_path = output_dir.path().join("docker-compose.yml");
        let content = std::fs::read_to_string(&output_path)
            .expect("Should be able to read rendered docker-compose.yml");

        // Verify MySQL service is present
        assert!(
            content.contains("mysql:"),
            "Rendered output should contain mysql service"
        );
        assert!(
            content.contains("image: mysql:8.0"),
            "Should use MySQL 8.0 image"
        );

        // Verify MySQL environment variables use environment variable references
        assert!(
            content.contains("MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}"),
            "Should reference MYSQL_ROOT_PASSWORD from .env file"
        );
        assert!(
            content.contains("MYSQL_DATABASE=${MYSQL_DATABASE}"),
            "Should reference MYSQL_DATABASE from .env file"
        );
        assert!(
            content.contains("MYSQL_USER=${MYSQL_USER}"),
            "Should reference MYSQL_USER from .env file"
        );
        assert!(
            content.contains("MYSQL_PASSWORD=${MYSQL_PASSWORD}"),
            "Should reference MYSQL_PASSWORD from .env file"
        );

        // Verify MySQL healthcheck
        assert!(
            content.contains("healthcheck:"),
            "Should have healthcheck section"
        );
        assert!(
            content.contains("mysqladmin"),
            "Should use mysqladmin for healthcheck"
        );
        assert!(content.contains("ping"), "Should use ping command");

        // Verify MySQL volume
        assert!(
            content.contains("mysql_data:"),
            "Should have mysql_data volume definition"
        );
        assert!(
            content.contains("driver: local"),
            "Volume should use local driver"
        );

        // Verify port mapping
        assert!(
            content.contains("3306:3306"),
            "Should expose MySQL port 3306"
        );
    }

    #[test]
    fn it_should_not_render_mysql_service_when_driver_is_sqlite() {
        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let sqlite_context = DockerComposeContext::builder(ports).build();

        let renderer = DockerComposeRenderer::new(template_manager);
        let output_dir = TempDir::new().unwrap();

        let result = renderer.render(&sqlite_context, output_dir.path());
        assert!(
            result.is_ok(),
            "Rendering with SQLite context should succeed: {:?}",
            result.err()
        );

        let output_path = output_dir.path().join("docker-compose.yml");
        let content = std::fs::read_to_string(&output_path)
            .expect("Should be able to read rendered docker-compose.yml");

        // Verify MySQL service is NOT present
        assert!(
            !content.contains("image: mysql:8.0"),
            "Should not contain MySQL service"
        );
        assert!(
            !content.contains("mysqladmin"),
            "Should not contain MySQL healthcheck"
        );
        assert!(
            !content.contains("mysql_data:"),
            "Should not contain mysql_data volume"
        );
    }

    #[test]
    fn it_should_render_prometheus_service_when_config_is_present() {
        use crate::domain::prometheus::PrometheusConfig;

        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(15).expect("15 is non-zero"));
        let context = DockerComposeContext::builder(ports)
            .with_prometheus(prometheus_config)
            .build();

        let renderer = DockerComposeRenderer::new(template_manager);
        let output_dir = TempDir::new().unwrap();

        let result = renderer.render(&context, output_dir.path());
        assert!(
            result.is_ok(),
            "Rendering with Prometheus context should succeed"
        );

        let output_path = output_dir.path().join("docker-compose.yml");
        let rendered_content = std::fs::read_to_string(&output_path)
            .expect("Should be able to read rendered docker-compose.yml");

        // Verify Prometheus service is present
        assert!(
            rendered_content.contains("prometheus:"),
            "Rendered output should contain prometheus service"
        );
        assert!(
            rendered_content.contains("image: prom/prometheus:v3.0.1"),
            "Should use Prometheus v3.0.1 image"
        );
        assert!(
            rendered_content.contains("container_name: prometheus"),
            "Should set container name"
        );

        // Verify port is bound to localhost only (not exposed to external network)
        assert!(
            rendered_content.contains("127.0.0.1:9090:9090"),
            "Prometheus port 9090 should be bound to localhost only (not exposed to external network)"
        );

        // Verify volume mount
        assert!(
            rendered_content.contains("./storage/prometheus/etc:/etc/prometheus:Z"),
            "Should mount Prometheus config directory"
        );

        // Verify service dependency
        assert!(
            rendered_content.contains("depends_on:"),
            "Should have depends_on section"
        );
        assert!(
            rendered_content.contains("- tracker"),
            "Should depend on tracker"
        );

        // Verify network segmentation (security enhancement)
        assert!(
            rendered_content.contains("- metrics_network"),
            "Should be on metrics_network for tracker ↔ Prometheus communication"
        );
    }

    #[test]
    fn it_should_not_render_prometheus_service_when_config_is_absent() {
        let temp_dir = TempDir::new().unwrap();
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let context = DockerComposeContext::builder(ports).build();

        let renderer = DockerComposeRenderer::new(template_manager);
        let output_dir = TempDir::new().unwrap();

        let result = renderer.render(&context, output_dir.path());
        assert!(
            result.is_ok(),
            "Rendering without Prometheus context should succeed"
        );

        let output_path = output_dir.path().join("docker-compose.yml");
        let rendered_content = std::fs::read_to_string(&output_path)
            .expect("Should be able to read rendered docker-compose.yml");

        // Verify Prometheus service is NOT present
        assert!(
            !rendered_content.contains("image: prom/prometheus:v3.0.1"),
            "Should not contain Prometheus service when config absent"
        );
        assert!(
            !rendered_content.contains("container_name: prometheus"),
            "Should not have prometheus container"
        );
        assert!(
            !rendered_content.contains("./storage/prometheus/etc:/etc/prometheus:Z"),
            "Should not have prometheus volume mount"
        );
    }
}
