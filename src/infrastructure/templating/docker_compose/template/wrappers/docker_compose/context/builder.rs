//! Builder for `DockerComposeContext`

use std::collections::{HashMap, HashSet};

// Internal crate
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::topology::{EnabledServices, Network, Service};

use super::caddy::CaddyServiceConfig;
use super::database::{DatabaseConfig, MysqlSetupConfig, DRIVER_MYSQL, DRIVER_SQLITE};
use super::grafana::GrafanaServiceConfig;
use super::mysql::MysqlServiceConfig;
use super::network_definition::NetworkDefinition;
use super::port_definition::PortDefinition;
use super::prometheus::PrometheusServiceConfig;
use super::{DockerComposeContext, TrackerServiceConfig};

/// Builder for `DockerComposeContext`
///
/// Provides a fluent API for constructing Docker Compose contexts with optional features.
/// Defaults to `SQLite` database configuration.
///
/// The builder collects domain configuration objects and transforms them into
/// service configuration objects with pre-computed networks at build time.
pub struct DockerComposeContextBuilder {
    tracker: TrackerServiceConfig,
    database: DatabaseConfig,
    prometheus_config: Option<PrometheusConfig>,
    grafana_config: Option<GrafanaConfig>,
    has_caddy: bool,
}

impl DockerComposeContextBuilder {
    /// Creates a new builder with default `SQLite` configuration
    pub(super) fn new(tracker: TrackerServiceConfig) -> Self {
        Self {
            tracker,
            database: DatabaseConfig {
                driver: DRIVER_SQLITE.to_string(),
                mysql: None,
            },
            prometheus_config: None,
            grafana_config: None,
            has_caddy: false,
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

    /// Enables Caddy TLS proxy
    ///
    /// When Caddy is enabled, it provides automatic HTTPS with Let's Encrypt
    /// certificates for services that have TLS enabled.
    #[must_use]
    pub fn with_caddy(mut self) -> Self {
        self.has_caddy = true;
        self
    }

    /// Builds the `DockerComposeContext`
    ///
    /// Transforms domain configuration objects into service configuration
    /// objects with pre-computed networks based on enabled features.
    ///
    /// # Panics
    ///
    /// This method does not validate port uniqueness. Use `try_build()` if you
    /// need validation to detect port conflicts before rendering.
    #[must_use]
    pub fn build(self) -> DockerComposeContext {
        // Note: This infallible version skips validation for backward compatibility.
        // Prefer try_build() for new code.
        self.build_internal()
    }

    /// Builds the `DockerComposeContext` with port conflict validation
    ///
    /// Validates that no two services expose the same host port before building
    /// the context. This prevents port conflicts that would cause Docker Compose
    /// to fail at runtime.
    ///
    /// # Errors
    ///
    /// Returns `PortConflictError` if two services try to bind the same host port.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let context = DockerComposeContext::builder(tracker_config)
    ///     .with_prometheus(prometheus_config)
    ///     .try_build()?;
    /// ```
    pub fn try_build(self) -> Result<DockerComposeContext, Box<PortConflictError>> {
        let context = self.build_internal();

        // Collect all ports from all services for validation
        Self::validate_port_uniqueness(&context)?;

        Ok(context)
    }

    /// Internal build logic shared by `build()` and `try_build()`
    fn build_internal(self) -> DockerComposeContext {
        let has_grafana = self.grafana_config.is_some();
        let has_caddy = self.has_caddy;
        let has_prometheus = self.prometheus_config.is_some();
        let has_mysql = self.database.driver == DRIVER_MYSQL;

        // Build list of enabled services for topology context
        let mut enabled_services = Vec::new();
        if has_prometheus {
            enabled_services.push(Service::Prometheus);
        }
        if has_grafana {
            enabled_services.push(Service::Grafana);
        }
        if has_mysql {
            enabled_services.push(Service::MySQL);
        }
        if has_caddy {
            enabled_services.push(Service::Caddy);
        }

        let topology_context = EnabledServices::from(&enabled_services);

        // Build Prometheus service config if enabled
        let prometheus = self
            .prometheus_config
            .as_ref()
            .map(|config| PrometheusServiceConfig::from_domain_config(config, &topology_context));

        // Build Grafana service config if enabled
        let grafana = self
            .grafana_config
            .as_ref()
            .map(|config| GrafanaServiceConfig::from_domain_config(config, &topology_context));

        // Build Caddy service config if enabled
        let caddy = if has_caddy {
            Some(CaddyServiceConfig::new())
        } else {
            None
        };

        // Build MySQL service config if enabled
        let mysql = if self.database.driver == DRIVER_MYSQL {
            Some(MysqlServiceConfig::new())
        } else {
            None
        };

        // Derive required networks from all service configurations
        let required_networks = Self::derive_required_networks(
            &self.tracker,
            prometheus.as_ref(),
            grafana.as_ref(),
            caddy.as_ref(),
            mysql.as_ref(),
        );

        DockerComposeContext {
            database: self.database,
            tracker: self.tracker,
            prometheus,
            grafana,
            caddy,
            mysql,
            required_networks,
        }
    }

    /// Derives required networks from all service configurations
    ///
    /// Collects networks from all enabled services, deduplicates them,
    /// and returns in deterministic alphabetical order.
    fn derive_required_networks(
        tracker: &TrackerServiceConfig,
        prometheus: Option<&PrometheusServiceConfig>,
        grafana: Option<&GrafanaServiceConfig>,
        caddy: Option<&CaddyServiceConfig>,
        mysql: Option<&MysqlServiceConfig>,
    ) -> Vec<NetworkDefinition> {
        let mut networks: HashSet<Network> = HashSet::new();

        // Collect from tracker (always present)
        networks.extend(tracker.networks.iter().copied());

        // Collect from optional services
        if let Some(prom) = prometheus {
            networks.extend(prom.networks.iter().copied());
        }
        if let Some(graf) = grafana {
            networks.extend(graf.networks.iter().copied());
        }
        if let Some(cad) = caddy {
            networks.extend(cad.networks.iter().copied());
        }
        if let Some(my) = mysql {
            networks.extend(my.networks.iter().copied());
        }

        // Sort for deterministic output (alphabetically by name)
        let mut result: Vec<NetworkDefinition> =
            networks.into_iter().map(NetworkDefinition::from).collect();
        result.sort_by(|a, b| a.name().cmp(b.name()));
        result
    }

    /// Validates that no two services bind the same host port
    ///
    /// Collects all ports from all services and checks for duplicates.
    fn validate_port_uniqueness(
        context: &DockerComposeContext,
    ) -> Result<(), Box<PortConflictError>> {
        // Map: host_port -> (service_name, binding_string)
        let mut port_bindings: HashMap<String, (&'static str, &PortDefinition)> = HashMap::new();

        // Helper to extract host port from binding string (e.g., "6969:6969/udp" -> "6969")
        // Also handles "127.0.0.1:9090:9090" -> "127.0.0.1:9090"
        let extract_host_port = |binding: &str| -> String {
            let parts: Vec<&str> = binding.split(':').collect();
            match parts.len() {
                2 => parts[0].to_string(),                 // "6969:6969" -> "6969"
                3 => format!("{}:{}", parts[0], parts[1]), // "127.0.0.1:9090:9090" -> "127.0.0.1:9090"
                _ => binding.to_string(),
            }
        };

        // Collect ports with service names
        let service_ports: Vec<(&'static str, &[PortDefinition])> = vec![
            ("tracker", &context.tracker.ports),
            (
                "prometheus",
                context.prometheus.as_ref().map_or(&[][..], |p| &p.ports),
            ),
            (
                "grafana",
                context.grafana.as_ref().map_or(&[][..], |g| &g.ports),
            ),
            (
                "caddy",
                context.caddy.as_ref().map_or(&[][..], |c| &c.ports),
            ),
            (
                "mysql",
                context.mysql.as_ref().map_or(&[][..], |m| &m.ports),
            ),
        ];

        for (service_name, ports) in service_ports {
            for port in ports {
                let host_port = extract_host_port(port.binding());

                if let Some((first_service, first_port)) = port_bindings.get(&host_port) {
                    return Err(Box::new(PortConflictError::new(
                        host_port,
                        first_service,
                        first_port.binding().to_string(),
                        first_port.description().to_string(),
                        service_name,
                        port.binding().to_string(),
                        port.description().to_string(),
                    )));
                }

                port_bindings.insert(host_port, (service_name, port));
            }
        }

        Ok(())
    }
}

/// Error indicating a port conflict between two services
///
/// This error is returned when two services try to bind the same host port,
/// which would cause Docker Compose to fail at runtime.
#[derive(Debug, Clone)]
pub struct PortConflictError {
    /// The conflicting host port (e.g., "9090" or "127.0.0.1:9090")
    pub host_port: String,
    /// Name of the first service that claimed the port
    pub first_service: &'static str,
    /// Port binding string of the first service
    pub first_binding: String,
    /// Description of the first port
    pub first_description: String,
    /// Name of the second service that also wants the port
    pub second_service: &'static str,
    /// Port binding string of the second service
    pub second_binding: String,
    /// Description of the second port
    pub second_description: String,
}

impl PortConflictError {
    /// Creates a new port conflict error
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host_port: String,
        first_service: &'static str,
        first_binding: String,
        first_description: String,
        second_service: &'static str,
        second_binding: String,
        second_description: String,
    ) -> Self {
        Self {
            host_port,
            first_service,
            first_binding,
            first_description,
            second_service,
            second_binding,
            second_description,
        }
    }

    /// Returns a help message suggesting how to resolve the conflict
    #[must_use]
    pub fn help(&self) -> String {
        format!(
            "Port {} is used by both '{}' ({}) and '{}' ({}).\n\
             To resolve:\n\
             - Configure different ports in your environment configuration\n\
             - Or disable one of the conflicting services",
            self.host_port,
            self.first_service,
            self.first_description,
            self.second_service,
            self.second_description
        )
    }
}

impl std::fmt::Display for PortConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Port conflict: {} ({}) and {} ({}) both bind to host port {}",
            self.first_service,
            self.first_binding,
            self.second_service,
            self.second_binding,
            self.host_port
        )
    }
}

impl std::error::Error for PortConflictError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::prometheus::PrometheusConfig;
    use crate::domain::topology::EnabledServices;
    use crate::domain::tracker::{
        DatabaseConfig as TrackerDatabaseConfig, HealthCheckApiConfig, HttpApiConfig,
        HttpTrackerConfig, SqliteConfig, TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
    };

    /// Helper to create a minimal domain tracker config with TLS (API not exposed)
    fn minimal_domain_tracker_config() -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![], // No UDP trackers
            vec![], // No HTTP trackers
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true, // TLS enabled
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a domain tracker config with specific UDP and HTTP ports
    fn domain_tracker_config_with_ports(
        udp_ports: Vec<u16>,
        http_ports: Vec<u16>,
    ) -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            udp_ports
                .into_iter()
                .map(|p| {
                    UdpTrackerConfig::new(format!("0.0.0.0:{p}").parse().unwrap(), None).unwrap()
                })
                .collect(),
            http_ports
                .into_iter()
                .map(|p| {
                    HttpTrackerConfig::new(format!("0.0.0.0:{p}").parse().unwrap(), None, false)
                        .unwrap()
                })
                .collect(),
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true, // TLS enabled
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a minimal tracker config
    fn minimal_tracker_config() -> TrackerServiceConfig {
        let domain_config = minimal_domain_tracker_config();
        let context = EnabledServices::from(&[]);
        TrackerServiceConfig::from_domain_config(&domain_config, &context)
    }

    /// Helper to create a tracker config that exposes specific ports
    fn tracker_config_with_ports(
        udp_ports: Vec<u16>,
        http_ports: Vec<u16>,
    ) -> TrackerServiceConfig {
        let domain_config = domain_tracker_config_with_ports(udp_ports, http_ports);
        let context = EnabledServices::from(&[]);
        TrackerServiceConfig::from_domain_config(&domain_config, &context)
    }

    // ==========================================================================
    // try_build validation tests
    // ==========================================================================

    #[test]
    fn it_should_build_successfully_when_no_port_conflicts() {
        let tracker = tracker_config_with_ports(vec![6969], vec![7070]);

        let result = DockerComposeContext::builder(tracker).try_build();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_build_successfully_with_prometheus() {
        let tracker = minimal_tracker_config();
        let prometheus = PrometheusConfig::default();

        let result = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus)
            .try_build();

        assert!(result.is_ok());
    }

    // ==========================================================================
    // PortConflictError tests
    // ==========================================================================

    #[test]
    fn it_should_display_port_conflict_error() {
        let error = PortConflictError::new(
            "9090".to_string(),
            "tracker",
            "9090:9090".to_string(),
            "Health check".to_string(),
            "prometheus",
            "127.0.0.1:9090:9090".to_string(),
            "Web UI".to_string(),
        );

        let display = error.to_string();

        assert!(display.contains("tracker"));
        assert!(display.contains("prometheus"));
        assert!(display.contains("9090"));
    }

    #[test]
    fn it_should_provide_help_message() {
        let error = PortConflictError::new(
            "9090".to_string(),
            "tracker",
            "9090:9090".to_string(),
            "Health check".to_string(),
            "prometheus",
            "127.0.0.1:9090:9090".to_string(),
            "Web UI".to_string(),
        );

        let help = error.help();

        assert!(help.contains("Port 9090"));
        assert!(help.contains("tracker"));
        assert!(help.contains("prometheus"));
        assert!(help.contains("To resolve"));
    }
}
