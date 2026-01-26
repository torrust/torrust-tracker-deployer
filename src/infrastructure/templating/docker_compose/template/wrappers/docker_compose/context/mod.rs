//! Context for the docker-compose.yml.tera template
//!
//! This module defines the structure and validation for Docker Compose services
//! that will be rendered into the docker-compose.yml file.

// External crates
use serde::Serialize;

// Submodules
mod builder;
mod caddy;
mod database;
mod grafana;
mod mysql;
mod network_definition;
mod port_definition;
mod prometheus;
mod tracker;

// Re-exports
pub use builder::{DockerComposeContextBuilder, PortConflictError};
pub use caddy::{CaddyDockerServiceConfig, CaddyServiceConfig};
pub use database::{DatabaseConfig, MysqlSetupConfig};
pub use grafana::GrafanaServiceConfig;
pub use mysql::{MysqlDockerServiceConfig, MysqlServiceConfig};
pub use network_definition::NetworkDefinition;
pub use port_definition::PortDefinition;
pub use prometheus::PrometheusServiceConfig;
pub use tracker::{TrackerPorts, TrackerServiceConfig};

/// Context for rendering the docker-compose.yml template
///
/// Contains all variables needed for the Docker Compose service configuration.
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Tracker service configuration (ports, networks)
    pub tracker: TrackerServiceConfig,
    /// Prometheus service configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<PrometheusServiceConfig>,
    /// Grafana service configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana: Option<GrafanaServiceConfig>,
    /// Caddy TLS proxy service configuration (optional)
    ///
    /// When present, Caddy reverse proxy is deployed for TLS termination.
    /// When absent, services are exposed directly over HTTP.
    ///
    /// Note: This is separate from `CaddyContext` (used for Caddyfile.tera).
    /// This type only contains the docker-compose service definition data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caddy: Option<CaddyServiceConfig>,
    /// `MySQL` service configuration (optional)
    ///
    /// Contains network configuration for the `MySQL` service.
    /// This is separate from `MysqlSetupConfig` which contains credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<MysqlServiceConfig>,
    /// All networks required by enabled services (derived)
    ///
    /// This list is computed from the networks used by all services.
    /// The template should iterate over this for the global `networks:` section
    /// instead of using conditionals.
    pub required_networks: Vec<NetworkDefinition>,
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
    /// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
    /// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Service};
    ///
    /// // Create tracker config from domain configuration
    /// let domain_config = TrackerConfig::default();
    /// let context = EnabledServices::from(&[]);
    /// let tracker_config = TrackerServiceConfig::from_domain_config(
    ///     &domain_config,
    ///     &context,
    /// );
    ///
    /// // SQLite (default)
    /// let compose_context = DockerComposeContext::builder(tracker_config.clone()).build();
    /// assert_eq!(compose_context.database().driver(), "sqlite3");
    ///
    /// // MySQL
    /// let domain_config_for_mysql = TrackerConfig::default();
    /// let mysql_context = EnabledServices::from(&[Service::MySQL]);
    /// let tracker_config_with_mysql = TrackerServiceConfig::from_domain_config(
    ///     &domain_config_for_mysql,
    ///     &mysql_context,
    /// );
    /// let mysql_config = MysqlSetupConfig {
    ///     root_password: "root_pass".to_string(),
    ///     database: "db".to_string(),
    ///     user: "user".to_string(),
    ///     password: "pass".to_string(),
    ///     port: 3306,
    /// };
    /// let compose_context = DockerComposeContext::builder(tracker_config_with_mysql)
    ///     .with_mysql(mysql_config)
    ///     .build();
    /// assert_eq!(compose_context.database().driver(), "mysql");
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

    /// Get the Prometheus service configuration if present
    #[must_use]
    pub fn prometheus(&self) -> Option<&PrometheusServiceConfig> {
        self.prometheus.as_ref()
    }

    /// Get the Grafana service configuration if present
    #[must_use]
    pub fn grafana(&self) -> Option<&GrafanaServiceConfig> {
        self.grafana.as_ref()
    }

    /// Get the Caddy TLS proxy service configuration if present
    #[must_use]
    pub fn caddy(&self) -> Option<&CaddyServiceConfig> {
        self.caddy.as_ref()
    }

    /// Get the `MySQL` service configuration if present
    #[must_use]
    pub fn mysql(&self) -> Option<&MysqlServiceConfig> {
        self.mysql.as_ref()
    }

    /// Get all networks required by enabled services
    ///
    /// This list is derived from all service network configurations and
    /// should be used in the template for the global `networks:` section.
    #[must_use]
    pub fn required_networks(&self) -> &[NetworkDefinition] {
        &self.required_networks
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::prometheus::PrometheusConfig;
    use crate::domain::topology::EnabledServices;
    use crate::domain::tracker::{
        DatabaseConfig as TrackerDatabaseConfig, HealthCheckApiConfig, HttpApiConfig,
        HttpTrackerConfig, SqliteConfig, TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
    };

    use super::*;

    /// Helper to create a domain `TrackerConfig` for tests (standard config)
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

    /// Helper to create a domain `TrackerConfig` with specific UDP ports
    fn domain_tracker_config_with_udp_port(port: u16) -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new(format!("0.0.0.0:{port}").parse().unwrap(), None).unwrap()],
            vec![],
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

    /// Helper to create a domain `TrackerConfig` with API TLS enabled
    fn domain_tracker_config_with_api_tls(port: u16) -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new(format!("0.0.0.0:{port}").parse().unwrap(), None).unwrap()],
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true,
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create `TrackerServiceConfig` for tests (no TLS, no networks)
    fn test_tracker_config() -> TrackerServiceConfig {
        let domain_config = test_domain_tracker_config();
        let context = EnabledServices::from(&[]);
        TrackerServiceConfig::from_domain_config(&domain_config, &context)
    }

    /// Helper to create `TrackerServiceConfig` with specific network configuration
    #[allow(clippy::fn_params_excessive_bools)]
    fn tracker_config_for_networks(
        has_prometheus: bool,
        has_mysql: bool,
        has_caddy: bool,
        use_api_tls: bool,
    ) -> TrackerServiceConfig {
        use crate::domain::topology::Service;
        let domain_config = if use_api_tls {
            domain_tracker_config_with_api_tls(6868)
        } else {
            domain_tracker_config_with_udp_port(6868)
        };
        let mut services = Vec::new();
        if has_prometheus {
            services.push(Service::Prometheus);
        }
        if has_mysql {
            services.push(Service::MySQL);
        }
        if has_caddy {
            services.push(Service::Caddy);
        }
        let context = EnabledServices::from(&services);
        TrackerServiceConfig::from_domain_config(&domain_config, &context)
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
    fn it_should_not_include_prometheus_by_default() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        assert!(context.prometheus().is_none());
    }

    #[test]
    fn it_should_include_prometheus_when_added() {
        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        assert!(context.prometheus().is_some());
        assert_eq!(context.prometheus().unwrap().scrape_interval_in_secs, 30);
    }

    #[test]
    fn it_should_not_serialize_prometheus_when_absent() {
        let tracker = test_tracker_config();
        let context = DockerComposeContext::builder(tracker).build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(!serialized.contains("prometheus"));
    }

    #[test]
    fn it_should_serialize_prometheus_when_present() {
        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(20).expect("20 is non-zero"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("prometheus"));
        assert!(serialized.contains("\"scrape_interval_in_secs\":20"));
    }

    #[test]
    fn it_should_compute_prometheus_networks_without_grafana() {
        use crate::domain::topology::Network;

        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(15).expect("15 is non-zero"));
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .build();

        let prometheus = context.prometheus().unwrap();
        assert_eq!(prometheus.networks, vec![Network::Metrics]);
    }

    #[test]
    fn it_should_compute_prometheus_networks_with_grafana() {
        use crate::domain::grafana::GrafanaConfig;
        use crate::domain::topology::Network;

        let tracker = test_tracker_config();
        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(15).expect("15 is non-zero"));
        let grafana_config =
            GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false);
        let context = DockerComposeContext::builder(tracker)
            .with_prometheus(prometheus_config)
            .with_grafana(grafana_config)
            .build();

        let prometheus = context.prometheus().unwrap();
        assert_eq!(
            prometheus.networks,
            vec![Network::Metrics, Network::Visualization]
        );
    }

    #[test]
    fn it_should_compute_grafana_networks_without_caddy() {
        use crate::domain::grafana::GrafanaConfig;
        use crate::domain::topology::Network;

        let tracker = test_tracker_config();
        let grafana_config =
            GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false);
        let context = DockerComposeContext::builder(tracker)
            .with_grafana(grafana_config)
            .build();

        let grafana = context.grafana().unwrap();
        assert_eq!(grafana.networks, vec![Network::Visualization]);
        assert!(!grafana.has_tls);
    }

    #[test]
    fn it_should_compute_grafana_networks_with_caddy() {
        use crate::domain::grafana::GrafanaConfig;
        use crate::domain::topology::Network;

        let tracker = test_tracker_config();
        let grafana_config =
            GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false);
        let context = DockerComposeContext::builder(tracker)
            .with_grafana(grafana_config)
            .with_caddy()
            .build();

        let grafana = context.grafana().unwrap();
        assert_eq!(
            grafana.networks,
            vec![Network::Visualization, Network::Proxy]
        );
    }

    // P2.2: Required networks derivation tests

    mod required_networks {
        use super::*;
        use crate::domain::grafana::GrafanaConfig;

        #[test]
        fn it_should_have_empty_required_networks_for_minimal_deployment() {
            let tracker = test_tracker_config();
            let context = DockerComposeContext::builder(tracker).build();

            assert!(context.required_networks().is_empty());
        }

        #[test]
        fn it_should_include_database_network_when_mysql_enabled() {
            let tracker = tracker_config_for_networks(false, true, false, false);
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

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(network_names.contains(&"database_network"));
        }

        #[test]
        fn it_should_include_metrics_network_when_prometheus_enabled() {
            let tracker = tracker_config_for_networks(true, false, false, false);
            let prometheus_config =
                PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
            let context = DockerComposeContext::builder(tracker)
                .with_prometheus(prometheus_config)
                .build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(network_names.contains(&"metrics_network"));
        }

        #[test]
        fn it_should_include_visualization_network_when_grafana_enabled() {
            let tracker = test_tracker_config();
            let grafana_config =
                GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false);
            let context = DockerComposeContext::builder(tracker)
                .with_grafana(grafana_config)
                .build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(network_names.contains(&"visualization_network"));
        }

        #[test]
        fn it_should_include_proxy_network_when_caddy_enabled() {
            let tracker = tracker_config_for_networks(false, false, true, true);
            let context = DockerComposeContext::builder(tracker).with_caddy().build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(network_names.contains(&"proxy_network"));
        }

        #[test]
        fn it_should_not_include_database_network_when_mysql_disabled() {
            let tracker = test_tracker_config();
            let context = DockerComposeContext::builder(tracker).build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(!network_names.contains(&"database_network"));
        }

        #[test]
        fn it_should_not_include_metrics_network_when_prometheus_disabled() {
            let tracker = test_tracker_config();
            let context = DockerComposeContext::builder(tracker).build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(!network_names.contains(&"metrics_network"));
        }

        #[test]
        fn it_should_not_include_visualization_network_when_grafana_disabled() {
            let tracker = test_tracker_config();
            let context = DockerComposeContext::builder(tracker).build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(!network_names.contains(&"visualization_network"));
        }

        #[test]
        fn it_should_not_include_proxy_network_when_caddy_disabled() {
            let tracker = test_tracker_config();
            let context = DockerComposeContext::builder(tracker).build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();
            assert!(!network_names.contains(&"proxy_network"));
        }

        #[test]
        fn it_should_include_all_networks_for_full_https_deployment() {
            let tracker = tracker_config_for_networks(true, true, true, true);
            let mysql_config = MysqlSetupConfig {
                root_password: "root".to_string(),
                database: "db".to_string(),
                user: "user".to_string(),
                password: "pass".to_string(),
                port: 3306,
            };
            let prometheus_config =
                PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
            let grafana_config =
                GrafanaConfig::new("admin".to_string(), "password".to_string(), None, true);
            let context = DockerComposeContext::builder(tracker)
                .with_mysql(mysql_config)
                .with_prometheus(prometheus_config)
                .with_grafana(grafana_config)
                .with_caddy()
                .build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();

            assert_eq!(network_names.len(), 4);
            assert!(network_names.contains(&"database_network"));
            assert!(network_names.contains(&"metrics_network"));
            assert!(network_names.contains(&"visualization_network"));
            assert!(network_names.contains(&"proxy_network"));
        }

        #[test]
        fn it_should_return_networks_in_deterministic_alphabetical_order() {
            let tracker = tracker_config_for_networks(true, true, true, true);
            let mysql_config = MysqlSetupConfig {
                root_password: "root".to_string(),
                database: "db".to_string(),
                user: "user".to_string(),
                password: "pass".to_string(),
                port: 3306,
            };
            let prometheus_config =
                PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
            let grafana_config =
                GrafanaConfig::new("admin".to_string(), "password".to_string(), None, true);
            let context = DockerComposeContext::builder(tracker)
                .with_mysql(mysql_config)
                .with_prometheus(prometheus_config)
                .with_grafana(grafana_config)
                .with_caddy()
                .build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();

            // Alphabetical order
            assert_eq!(
                network_names,
                vec![
                    "database_network",
                    "metrics_network",
                    "proxy_network",
                    "visualization_network"
                ]
            );
        }

        #[test]
        fn it_should_deduplicate_networks_from_multiple_services() {
            // Prometheus and Grafana both use visualization_network
            let tracker = tracker_config_for_networks(true, false, false, false);
            let prometheus_config =
                PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
            let grafana_config =
                GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false);
            let context = DockerComposeContext::builder(tracker)
                .with_prometheus(prometheus_config)
                .with_grafana(grafana_config)
                .build();

            let network_names: Vec<&str> = context
                .required_networks()
                .iter()
                .map(NetworkDefinition::name)
                .collect();

            // visualization_network appears only once despite being used by both
            assert_eq!(
                network_names
                    .iter()
                    .filter(|n| **n == "visualization_network")
                    .count(),
                1
            );
        }
    }
}
