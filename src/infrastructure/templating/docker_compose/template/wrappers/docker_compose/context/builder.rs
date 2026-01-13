//! Builder for `DockerComposeContext`

// Internal crate
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::infrastructure::templating::caddy::CaddyContext;

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
    grafana_config: Option<GrafanaConfig>,
    caddy_config: Option<CaddyContext>,
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
            grafana_config: None,
            caddy_config: None,
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

    /// Adds Grafana configuration
    ///
    /// # Arguments
    ///
    /// * `grafana_config` - Grafana configuration
    #[must_use]
    pub fn with_grafana(mut self, grafana_config: GrafanaConfig) -> Self {
        self.grafana_config = Some(grafana_config);
        self
    }

    /// Adds Caddy TLS proxy configuration
    ///
    /// When Caddy is configured, it provides automatic HTTPS with Let's Encrypt
    /// certificates for services that have TLS enabled.
    ///
    /// # Arguments
    ///
    /// * `caddy_config` - Caddy configuration with services to proxy
    #[must_use]
    pub fn with_caddy(mut self, caddy_config: CaddyContext) -> Self {
        self.caddy_config = Some(caddy_config);
        self
    }

    /// Builds the `DockerComposeContext`
    #[must_use]
    pub fn build(self) -> DockerComposeContext {
        DockerComposeContext {
            database: self.database,
            ports: self.ports,
            prometheus_config: self.prometheus_config,
            grafana_config: self.grafana_config,
            caddy_config: self.caddy_config,
        }
    }
}
