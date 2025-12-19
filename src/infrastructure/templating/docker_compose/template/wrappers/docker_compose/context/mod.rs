//! Context for the docker-compose.yml.tera template
//!
//! This module defines the structure and validation for Docker Compose services
//! that will be rendered into the docker-compose.yml file.

// External crates
use serde::Serialize;

// Internal crate
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;

// Submodules
mod builder;
mod database;
mod ports;

// Re-exports
pub use builder::DockerComposeContextBuilder;
pub use database::{DatabaseConfig, MysqlSetupConfig};
pub use ports::TrackerPorts;

/// Context for rendering the docker-compose.yml template
///
/// Contains all variables needed for the Docker Compose service configuration.
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Tracker port configuration
    pub ports: TrackerPorts,
    /// Prometheus configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus_config: Option<PrometheusConfig>,
    /// Grafana configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana_config: Option<GrafanaConfig>,
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
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::{DockerComposeContext, TrackerPorts, MysqlSetupConfig};
    ///
    /// let ports = TrackerPorts {
    ///     udp_tracker_ports: vec![6868, 6969],
    ///     http_tracker_ports: vec![7070],
    ///     http_api_port: 1212,
    /// };
    ///
    /// // SQLite (default)
    /// let context = DockerComposeContext::builder(ports.clone()).build();
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
    /// let context = DockerComposeContext::builder(ports)
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

    /// Get the tracker ports configuration
    #[must_use]
    pub fn ports(&self) -> &TrackerPorts {
        &self.ports
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_sqlite_configuration() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let context = DockerComposeContext::builder(ports).build();

        assert_eq!(context.database().driver(), "sqlite3");
        assert!(context.database().mysql().is_none());
        assert_eq!(context.ports().udp_tracker_ports, vec![6868, 6969]);
        assert_eq!(context.ports().http_tracker_ports, vec![7070]);
        assert_eq!(context.ports().http_api_port, 1212);
    }

    #[test]
    fn it_should_create_context_with_mysql_configuration() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let mysql_config = MysqlSetupConfig {
            root_password: "root123".to_string(),
            database: "tracker".to_string(),
            user: "tracker_user".to_string(),
            password: "pass456".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(ports)
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

        assert_eq!(context.ports().udp_tracker_ports, vec![6868, 6969]);
        assert_eq!(context.ports().http_tracker_ports, vec![7070]);
        assert_eq!(context.ports().http_api_port, 1212);
    }

    #[test]
    fn it_should_be_serializable_with_sqlite() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let context = DockerComposeContext::builder(ports).build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("sqlite3"));
        assert!(!serialized.contains("mysql"));
    }

    #[test]
    fn it_should_be_serializable_with_mysql() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let mysql_config = MysqlSetupConfig {
            root_password: "root".to_string(),
            database: "db".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(ports)
            .with_mysql(mysql_config)
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("mysql"));
        assert!(serialized.contains("root"));
        assert!(serialized.contains("db"));
        assert!(serialized.contains("user"));
        assert!(serialized.contains("pass"));
        assert!(serialized.contains("3306"));
    }

    #[test]
    fn it_should_be_cloneable() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let mysql_config = MysqlSetupConfig {
            root_password: "root".to_string(),
            database: "db".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            port: 3306,
        };
        let context = DockerComposeContext::builder(ports)
            .with_mysql(mysql_config)
            .build();

        let cloned = context.clone();
        assert_eq!(cloned.database().driver(), "mysql");
    }

    #[test]
    fn it_should_not_include_prometheus_config_by_default() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let context = DockerComposeContext::builder(ports).build();

        assert!(context.prometheus_config().is_none());
    }

    #[test]
    fn it_should_include_prometheus_config_when_added() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
        let context = DockerComposeContext::builder(ports)
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
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let context = DockerComposeContext::builder(ports).build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(!serialized.contains("prometheus_config"));
    }

    #[test]
    fn it_should_serialize_prometheus_config_when_present() {
        let ports = TrackerPorts {
            udp_tracker_ports: vec![6868, 6969],
            http_tracker_ports: vec![7070],
            http_api_port: 1212,
        };
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(20).expect("20 is non-zero"));
        let context = DockerComposeContext::builder(ports)
            .with_prometheus(prometheus_config)
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("prometheus_config"));
        assert!(serialized.contains("\"scrape_interval_in_secs\":20"));
    }
}
