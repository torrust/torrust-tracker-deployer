//! Context for the docker-compose.yml.tera template
//!
//! This module defines the structure and validation for Docker Compose services
//! that will be rendered into the docker-compose.yml file.

// External crates
use serde::Serialize;

// Internal crate
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::infrastructure::templating::caddy::CaddyContext;

// Submodules
mod builder;
mod database;
mod ports;

// Re-exports
pub use builder::DockerComposeContextBuilder;
pub use database::{DatabaseConfig, MysqlSetupConfig};
pub use ports::{TrackerPorts, TrackerServiceConfig};

/// Context for rendering the docker-compose.yml template
///
/// Contains all variables needed for the Docker Compose service configuration.
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Tracker service configuration (ports, networks)
    pub tracker: TrackerServiceConfig,
    /// Prometheus configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus_config: Option<PrometheusConfig>,
    /// Grafana configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana_config: Option<GrafanaConfig>,
    /// Caddy TLS proxy configuration (optional)
    ///
    /// When present, Caddy reverse proxy is deployed for TLS termination.
    /// When absent, services are exposed directly over HTTP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caddy_config: Option<CaddyContext>,
    /// Whether Grafana has TLS enabled (port should not be exposed if true)
    #[serde(default)]
    pub grafana_has_tls: bool,
}

impl DockerComposeContext {
    /// Creates a new builder for `DockerComposeContext`
    ///
    /// The builder starts with `SQLite` as the default database configuration.
    /// Use `with_mysql()` to switch to `MySQL` configuration.
    ///
    /// # Arguments
    ///
    /// * `ports` - Tracker port configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{DockerComposeContext, TrackerServiceConfig, MysqlSetupConfig};
    ///
    /// let tracker_config = TrackerServiceConfig::new(
    ///     vec![6868, 6969],        // UDP ports (always exposed)
    ///     vec![7070],              // HTTP ports without TLS
    ///     1212,                    // API port
    ///     false,                   // API has no TLS
    ///     false,                   // has_prometheus
    ///     false,                   // has_mysql
    ///     false,                   // has_caddy
    /// );
    ///
    /// // SQLite (default)
    /// let context = DockerComposeContext::builder(tracker_config.clone()).build();
    /// assert_eq!(context.database().driver(), "sqlite3");
    ///
    /// // MySQL
    /// let mysql_config = MysqlSetupConfig {
    ///     root_password: "root_pass".to_string(),
    ///     database: "db".to_string(),
    ///     user: "user".to_string(),
    ///     password: "pass".to_string(),
    ///     port: 3306,
    /// };
    /// let context = DockerComposeContext::builder(tracker_config)
    ///     .with_mysql(mysql_config)
    ///     .build();
    /// assert_eq!(context.database().driver(), "mysql");
    /// ```
    #[must_use]
    pub fn builder(ports: TrackerPorts) -> DockerComposeContextBuilder {
        DockerComposeContextBuilder::new(ports)
    }

    /// Get the database configuration
    #[must_use]
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    /// Get the tracker service configuration
    #[must_use]
    pub fn tracker(&self) -> &TrackerServiceConfig {
        &self.tracker
    }

    /// Get the Prometheus configuration if present
    #[must_use]
    pub fn prometheus_config(&self) -> Option<&PrometheusConfig> {
        self.prometheus_config.as_ref()
    }

    /// Get the Grafana configuration if present
    #[must_use]
    pub fn grafana_config(&self) -> Option<&GrafanaConfig> {
        self.grafana_config.as_ref()
    }

    /// Get the Caddy TLS proxy configuration if present
    #[must_use]
    pub fn caddy_config(&self) -> Option<&CaddyContext> {
        self.caddy_config.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create `TrackerServiceConfig` for tests (no TLS, no networks)
    fn test_tracker_config() -> TrackerServiceConfig {
        TrackerServiceConfig::new(
            vec![6868, 6969], // UDP ports
            vec![7070],       // HTTP ports without TLS
            1212,             // API port
            false,            // API has no TLS
            false,            // has_prometheus
            false,            // has_mysql
            false,            // has_caddy
        )
    }

    #[test]
    fn it_should_create_context_with_sqlite_configuration() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        assert_eq!(context.database().driver(), "sqlite3");
        assert!(context.database().mysql().is_none());
        assert_eq!(context.tracker().udp_tracker_ports, vec![6868, 6969]);
        assert_eq!(context.tracker().http_tracker_ports_without_tls, vec![7070]);
        assert_eq!(context.tracker().http_api_port, 1212);
    }

    #[test]
    fn it_should_create_context_with_mysql_configuration() {
        let tracker = test_tracker_config();
        let mysql_config = MysqlSetupConfig {
            root_password: "root123".to_string(),
            database: "tracker".to_string(),
            user: "tracker_user".to_string(),
            password: "pass456".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(tracker)
            .with_mysql(mysql_config)
            .build();

        assert_eq!(context.database().driver(), "mysql");
        assert!(context.database().mysql().is_some());

        let mysql = context.database().mysql().unwrap();
        assert_eq!(mysql.root_password, "root123");
        assert_eq!(mysql.database, "tracker");
        assert_eq!(mysql.user, "tracker_user");
        assert_eq!(mysql.password, "pass456");
        assert_eq!(mysql.port, 3306);

        assert_eq!(context.tracker().udp_tracker_ports, vec![6868, 6969]);
        assert_eq!(context.tracker().http_tracker_ports_without_tls, vec![7070]);
        assert_eq!(context.tracker().http_api_port, 1212);
    }

    #[test]
    fn it_should_be_serializable_with_sqlite() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("sqlite3"));
        assert!(!serialized.contains("\"driver\":\"mysql\""));
    }

    #[test]
    fn it_should_be_serializable_with_mysql() {
        let tracker = test_tracker_config();
        let mysql_config = MysqlSetupConfig {
            root_password: "root".to_string(),
            database: "db".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(tracker)
            .with_mysql(mysql_config)
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("mysql"));
        assert!(serialized.contains("root"));
        assert!(serialized.contains("db"));
        assert!(serialized.contains("\"user\":\"user\""));
        assert!(serialized.contains("pass"));
        assert!(serialized.contains("3306"));
    }

    #[test]
    fn it_should_be_cloneable() {
        let tracker = test_tracker_config();
        let mysql_config = MysqlSetupConfig {
            root_password: "root".to_string(),
            database: "db".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(tracker)
            .with_mysql(mysql_config)
            .build();

        let cloned = context.clone();
        assert_eq!(cloned.database().driver(), "mysql");
    }

    #[test]
    fn it_should_not_include_prometheus_config_by_default() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        assert!(context.prometheus_config().is_none());
    }

    #[test]
    fn it_should_include_prometheus_config_when_added() {
        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        assert!(context.prometheus_config().is_some());
        assert_eq!(
            context
                .prometheus_config()
                .unwrap()
                .scrape_interval_in_secs(),
            30
        );
    }

    #[test]
    fn it_should_not_serialize_prometheus_config_when_absent() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(!serialized.contains("prometheus_config"));
    }

    #[test]
    fn it_should_serialize_prometheus_config_when_present() {
        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(20).expect("20 is non-zero"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("prometheus_config"));
        assert!(serialized.contains("\"scrape_interval_in_secs\":20"));
    }
}
