//! Template wrapper for rendering the docker-compose.yml file
//!
//! This module provides the `DockerComposeTemplate` type that handles rendering
//! of the docker-compose.yml.tera template with service configuration context.

use std::path::Path;

use crate::domain::template::file::File;
use crate::domain::template::{
    write_file_with_dir_creation, FileOperationError, TemplateEngineError,
};

use super::context::DockerComposeContext;

/// Template wrapper for the docker-compose.yml.tera template
///
/// Handles rendering of Docker Compose service definitions from the template.
#[derive(Debug)]
pub struct DockerComposeTemplate {
    context: DockerComposeContext,
    content: String,
}

impl DockerComposeTemplate {
    /// Creates a new `DockerComposeTemplate`, validating the template content and variable substitution
    ///
    /// # Arguments
    ///
    /// * `template_file` - The docker-compose.yml.tera template file content
    /// * `context` - The context containing service configuration
    ///
    /// # Returns
    ///
    /// * `Result<Self, TemplateEngineError>` - The validated template or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template syntax is invalid
    /// - Required variables cannot be substituted
    /// - Template validation fails
    pub fn new(
        template_file: &File,
        context: DockerComposeContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();

        let validated_content =
            engine.render(template_file.filename(), template_file.content(), &context)?;

        Ok(Self {
            context,
            content: validated_content,
        })
    }

    /// Get the database configuration
    #[must_use]
    pub fn database(&self) -> &crate::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::DatabaseConfig{
        self.context.database()
    }

    /// Render the template to a file at the specified output path
    ///
    /// # Arguments
    ///
    /// * `output_path` - The path where the docker-compose.yml file should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), FileOperationError>` - Success or file operation error
    ///
    /// # Errors
    ///
    /// Returns `FileOperationError::DirectoryCreation` if the parent directory cannot be created,
    /// or `FileOperationError::FileWrite` if the file cannot be written
    pub fn render(&self, output_path: &Path) -> Result<(), FileOperationError> {
        write_file_with_dir_creation(output_path, &self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::{MysqlSetupConfig, TrackerServiceContext};
    use super::*;

    use crate::domain::topology::EnabledServices;
    use crate::domain::tracker::{
        DatabaseConfig as TrackerDatabaseConfig, HealthCheckApiConfig, HttpApiConfig,
        HttpTrackerConfig, SqliteConfig, TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
    };

    /// Helper to create a domain `TrackerConfig` for tests
    fn test_domain_tracker_config() -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![
                UdpTrackerConfig::new("0.0.0.0:6868".parse().unwrap(), None).unwrap(),
                UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap(),
            ],
            vec![HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap()],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                None,
                false,
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create `TrackerServiceContext` for tests (no TLS, no networks)
    fn test_tracker_config() -> TrackerServiceContext {
        let domain_config = test_domain_tracker_config();
        let context = EnabledServices::from(&[]);
        TrackerServiceContext::from_domain_config(&domain_config, &context)
    }

    #[test]
    fn it_should_create_docker_compose_template_with_sqlite() {
        let template_content = r#"
services:
  tracker:
    image: torrust/tracker:develop
{% if database.driver == "mysql" %}
  mysql:
    image: mysql:8.4
{% endif %}
"#;

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        assert_eq!(template.database().driver(), "sqlite3");
        // MySQL service should not be in the rendered content
        assert!(!template.content.contains("mysql:"));
    }

    #[test]
    fn it_should_create_docker_compose_template_with_mysql() {
        let template_content = r#"
services:
  tracker:
    image: torrust/tracker:develop
{% if database.driver == "mysql" %}
  mysql:
    image: mysql:8.4
    environment:
      - MYSQL_ROOT_PASSWORD={{ database.mysql.root_password }}
{% endif %}
"#;

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let tracker = test_tracker_config();
        let mysql_config = MysqlSetupConfig {
            root_password: "root123".to_string(),
            database: "tracker".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(tracker)
            .with_mysql(mysql_config)
            .build();
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        assert_eq!(template.database().driver(), "mysql");
        // MySQL service should be in the rendered content
        assert!(template.content.contains("mysql:"));
        assert!(template.content.contains("MYSQL_ROOT_PASSWORD=root123"));
    }

    #[test]
    fn it_should_render_to_file() {
        use tempfile::TempDir;

        let template_content = r"
services:
  tracker:
    image: torrust/tracker:develop
";
        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        // Create temp directory for output
        // Create temp directory for output
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("docker-compose.yml");

        // Render to file
        template.render(&output_path).unwrap();

        // Verify file exists and contains expected content
        assert!(output_path.exists());
        let file_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(file_content.contains("torrust/tracker:develop"));
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        let template_content = "{% if database.driver == mysql %}"; // Missing quotes and `endif`

        let template_file =
            File::new("docker-compose.yml.tera", template_content.to_string()).unwrap();

        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();
        let result = DockerComposeTemplate::new(&template_file, context);

        assert!(result.is_err());
    }

    // Template snippet that mirrors the tracker service `networks:` block from the real template.
    // Used to verify that the conditional guard prevents an empty `networks:` key.
    // Only covers the environment → networks → ports transition, which is the exact
    // sequence that previously produced invalid YAML.
    const TRACKER_NETWORKS_TEMPLATE: &str = r#"services:
  tracker:
    environment:
      - USER_ID=1000
{%- if tracker.networks | length > 0 %}
    networks:
{%- for network in tracker.networks %}
      - {{ network }}
{%- endfor %}
{%- endif %}
{%- if tracker.ports | length > 0 %}
    ports:
{%- for port in tracker.ports %}
      - "{{ port.binding }}"
{%- endfor %}
{%- endif %}
"#;

    #[test]
    fn it_should_not_render_empty_networks_key_for_tracker_when_no_optional_services_are_configured(
    ) {
        // Arrange: minimal config — SQLite, no Prometheus, no Caddy, no MySQL.
        // tracker.networks will be an empty Vec under these conditions.
        let template_file = File::new(
            "docker-compose.yml.tera",
            TRACKER_NETWORKS_TEMPLATE.to_string(),
        )
        .unwrap();

        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        // Act
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        // Assert: an empty `networks:` key must not appear in the output.
        // Before the fix this rendered:
        //   networks:
        //   ports:
        // which is invalid YAML rejected by Docker Compose.
        assert!(
            !template.content.contains("    networks:"),
            "Empty `networks:` key must not be rendered when tracker has no networks; \
             actual output:\n{}",
            template.content
        );
    }

    #[test]
    fn it_should_render_networks_key_for_tracker_when_prometheus_is_enabled() {
        use std::num::NonZeroU32;

        use crate::domain::prometheus::PrometheusConfig;

        // Arrange: Prometheus enabled → tracker gets the metrics network.
        let template_file = File::new(
            "docker-compose.yml.tera",
            TRACKER_NETWORKS_TEMPLATE.to_string(),
        )
        .unwrap();

        let tracker = {
            let domain_config = test_domain_tracker_config();
            let enabled = EnabledServices::from(&[crate::domain::topology::Service::Prometheus]);
            TrackerServiceContext::from_domain_config(&domain_config, &enabled)
        };
        let prometheus_config =
            PrometheusConfig::new(NonZeroU32::new(15).expect("non-zero scrape interval"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        // Act
        let template = DockerComposeTemplate::new(&template_file, context).unwrap();

        // Assert: `networks:` must appear and list the metrics network.
        assert!(
            template.content.contains("    networks:"),
            "`networks:` key must be present when tracker has networks; \
             actual output:\n{}",
            template.content
        );
        assert!(
            template.content.contains("metrics_network"),
            "metrics_network must appear in tracker networks; \
             actual output:\n{}",
            template.content
        );
    }
}
