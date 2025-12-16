//! Builder for `DockerComposeContext`

// Internal crate
use crate::domain::prometheus::PrometheusConfig;

use super::database::{DatabaseConfig, MysqlSetupConfig, DRIVER_MYSQL, DRIVER_SQLITE};
use super::{DockerComposeContext, TrackerPorts};

/// Builder for `DockerComposeContext`
///
/// Provides a fluent API for constructing Docker Compose contexts with optional features.
/// Defaults to `SQLite` database configuration.
pub struct DockerComposeContextBuilder {
    ports: TrackerPorts,
    database: DatabaseConfig,
    prometheus_config: Option<PrometheusConfig>,
}

impl DockerComposeContextBuilder {
    /// Creates a new builder with default `SQLite` configuration
    pub(super) fn new(ports: TrackerPorts) -> Self {
        Self {
            ports,
            database: DatabaseConfig {
                driver: DRIVER_SQLITE.to_string(),
                mysql: None,
            },
            prometheus_config: None,
        }
    }

    /// Switches to `MySQL` database configuration
    ///
    /// # Arguments
    ///
    /// * `mysql_config` - `MySQL` setup configuration
    #[must_use]
    pub fn with_mysql(mut self, mysql_config: MysqlSetupConfig) -> Self {
        self.database = DatabaseConfig {
            driver: DRIVER_MYSQL.to_string(),
            mysql: Some(mysql_config),
        };
        self
    }

    /// Adds Prometheus configuration
    ///
    /// # Arguments
    ///
    /// * `prometheus_config` - Prometheus configuration
    #[must_use]
    pub fn with_prometheus(mut self, prometheus_config: PrometheusConfig) -> Self {
        self.prometheus_config = Some(prometheus_config);
        self
    }

    /// Builds the `DockerComposeContext`
    #[must_use]
    pub fn build(self) -> DockerComposeContext {
        DockerComposeContext {
            database: self.database,
            ports: self.ports,
            prometheus_config: self.prometheus_config,
        }
    }
}
