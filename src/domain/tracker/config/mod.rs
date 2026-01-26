//! Tracker configuration domain types
//!
//! This module contains the main tracker configuration and component types
//! used for deploying the Torrust Tracker.

use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::{BindingAddress, Protocol};
use crate::domain::topology::{
    EnabledServices, Network, NetworkDerivation, PortBinding, PortDerivation, Service,
};
use crate::shared::DomainName;

mod core;
mod health_check_api;
mod http;
mod http_api;
mod udp;

pub use core::{
    DatabaseConfig, MysqlConfig, MysqlConfigError, SqliteConfig, SqliteConfigError,
    TrackerCoreConfig,
};
pub use health_check_api::{HealthCheckApiConfig, HealthCheckApiConfigError};
pub use http::{HttpTrackerConfig, HttpTrackerConfigError};
pub use http_api::{HttpApiConfig, HttpApiConfigError};
pub use udp::{UdpTrackerConfig, UdpTrackerConfigError};

/// Checks if a socket address is bound to localhost (127.0.0.1 or `::1`).
///
/// This is used to validate that TLS-enabled services don't bind to localhost,
/// since Caddy runs in a separate container and cannot reach localhost addresses.
///
/// # Returns
///
/// `true` if the address is IPv4 localhost (127.0.0.1) or IPv6 localhost (`::1`),
/// `false` otherwise.
///
/// # Note
///
/// This intentionally checks only exact localhost addresses (127.0.0.1 and `::1`),
/// not the entire 127.0.0.0/8 loopback range, as per design decision.
#[must_use]
pub fn is_localhost(addr: &SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(ipv4) => ipv4 == Ipv4Addr::LOCALHOST,
        IpAddr::V6(ipv6) => ipv6 == Ipv6Addr::LOCALHOST,
    }
}

/// Tracker deployment configuration
///
/// This structure mirrors the real tracker configuration but only includes
/// user-configurable fields that are exposed via the environment.json file.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::{
///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
/// };
///
/// let tracker_config = TrackerConfig::new(
///     TrackerCoreConfig::new(
///         DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
///         false,
///     ),
///     vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
///     vec![HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap()],
///     HttpApiConfig::new(
///         "0.0.0.0:1212".parse().unwrap(),
///         "MyAccessToken".to_string().into(),
///         None,
///         false,
///     ).expect("valid config"),
///     HealthCheckApiConfig::new(
///         "127.0.0.1:1313".parse().unwrap(),
///         None,
///         false,
///     ).expect("valid config"),
/// ).expect("valid config");
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrackerConfig {
    /// Core tracker configuration
    core: TrackerCoreConfig,

    /// UDP tracker instances
    udp_trackers: Vec<UdpTrackerConfig>,

    /// HTTP tracker instances
    http_trackers: Vec<HttpTrackerConfig>,

    /// HTTP API configuration
    http_api: HttpApiConfig,

    /// Health Check API configuration
    health_check_api: HealthCheckApiConfig,
}

/// Error type for tracker configuration validation failures
#[derive(Debug, Clone, PartialEq)]
pub enum TrackerConfigError {
    /// Multiple services attempting to bind to the same socket address
    DuplicateSocketAddress {
        /// The conflicting socket address
        address: SocketAddr,
        /// The protocol (UDP or TCP)
        protocol: Protocol,
        /// Names of services attempting to bind to this address
        services: Vec<String>,
    },
}

impl fmt::Display for TrackerConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSocketAddress {
                address,
                protocol,
                services,
            } => {
                let services_list = services
                    .iter()
                    .map(|s| format!("'{s}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "Socket address conflict: {services_list} cannot bind to {address} ({protocol})\n\
                    Tip: Assign different port numbers to each service"
                )
            }
        }
    }
}

impl std::error::Error for TrackerConfigError {}

impl TrackerConfigError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::DuplicateSocketAddress {
                address,
                protocol,
                services,
            } => {
                use std::fmt::Write;

                let mut help =
                    String::from("Socket Address Conflict - Detailed Troubleshooting:\n\n");

                help.push_str("Conflicting services:\n");
                for service in services {
                    let _ = writeln!(help, "  - {service}: {address} ({protocol})");
                }
                help.push('\n');

                help.push_str("Why this fails:\n");
                let _ = write!(
                    help,
                    "Two services using the same protocol ({protocol}) cannot bind to the same\n\
                    IP address and port number. The second service will fail with\n\
                    \"Address already in use\" error.\n\n"
                );

                help.push_str("How to fix:\n");
                help.push_str(
                    "1. Assign different port numbers to each service\n\
                    2. Or configure only one service to use this address\n\n",
                );

                help.push_str("Note:\n");
                help.push_str(
                    "Services using different protocols (UDP vs TCP) CAN share the same port.\n\
                    See: docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md\n",
                );

                help
            }
        }
    }
}

impl TrackerConfig {
    /// Creates a new `TrackerConfig` with validated aggregate invariants.
    ///
    /// This constructor validates that no socket address conflicts exist
    /// (multiple services binding to the same IP:port:protocol combination).
    ///
    /// # Errors
    ///
    /// Returns `TrackerConfigError::DuplicateSocketAddress` if multiple services
    /// using the same protocol attempt to bind to the same socket address.
    ///
    /// # Note
    ///
    /// Individual component validation (port != 0, TLS requires domain, localhost
    /// cannot use TLS) is enforced by the child config types at their construction
    /// time. This constructor only validates aggregate-level invariants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{
    ///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
    ///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
    /// };
    ///
    /// let config = TrackerConfig::new(
    ///     TrackerCoreConfig::new(
    ///         DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
    ///         false,
    ///     ),
    ///     vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
    ///     vec![HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap()],
    ///     HttpApiConfig::new(
    ///         "0.0.0.0:1212".parse().unwrap(),
    ///         "MyAccessToken".to_string().into(),
    ///         None,
    ///         false,
    ///     ).unwrap(),
    ///     HealthCheckApiConfig::new(
    ///         "127.0.0.1:1313".parse().unwrap(),
    ///         None,
    ///         false,
    ///     ).unwrap(),
    /// ).expect("valid config");
    /// ```
    pub fn new(
        core: TrackerCoreConfig,
        udp_trackers: Vec<UdpTrackerConfig>,
        http_trackers: Vec<HttpTrackerConfig>,
        http_api: HttpApiConfig,
        health_check_api: HealthCheckApiConfig,
    ) -> Result<Self, TrackerConfigError> {
        let config = Self {
            core,
            udp_trackers,
            http_trackers,
            http_api,
            health_check_api,
        };

        // Validate aggregate-level invariants
        // (Child components are already validated at their construction)
        config.check_socket_address_conflicts()?;

        Ok(config)
    }

    /// Returns the core tracker configuration.
    #[must_use]
    pub fn core(&self) -> &TrackerCoreConfig {
        &self.core
    }

    /// Returns the UDP tracker configurations.
    #[must_use]
    pub fn udp_trackers(&self) -> &[UdpTrackerConfig] {
        &self.udp_trackers
    }

    /// Returns the HTTP tracker configurations.
    #[must_use]
    pub fn http_trackers(&self) -> &[HttpTrackerConfig] {
        &self.http_trackers
    }

    /// Returns the HTTP API configuration.
    #[must_use]
    pub fn http_api(&self) -> &HttpApiConfig {
        &self.http_api
    }

    /// Returns the Health Check API configuration.
    #[must_use]
    pub fn health_check_api(&self) -> &HealthCheckApiConfig {
        &self.health_check_api
    }

    /// Returns whether the tracker is configured to use `MySQL` database.
    ///
    /// This is useful for determining if MySQL-related infrastructure
    /// (like storage directories) needs to be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{
    ///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
    ///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
    /// };
    ///
    /// let tracker_config = TrackerConfig::new(
    ///     TrackerCoreConfig::new(
    ///         DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
    ///         false,
    ///     ),
    ///     vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
    ///     vec![HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap()],
    ///     HttpApiConfig::new(
    ///         "0.0.0.0:1212".parse().unwrap(),
    ///         "MyAccessToken".to_string().into(),
    ///         None,
    ///         false,
    ///     ).expect("valid config"),
    ///     HealthCheckApiConfig::new(
    ///         "127.0.0.1:1313".parse().unwrap(),
    ///         None,
    ///         false,
    ///     ).expect("valid config"),
    /// ).expect("valid config");
    ///
    /// // SQLite config -> not MySQL
    /// assert!(!tracker_config.uses_mysql());
    /// ```
    #[must_use]
    pub fn uses_mysql(&self) -> bool {
        matches!(self.core.database(), DatabaseConfig::Mysql(_))
    }

    /// Checks for socket address conflicts
    ///
    /// Validates that no two services using the same protocol attempt to bind
    /// to the same socket address (IP + port).
    fn check_socket_address_conflicts(&self) -> Result<(), TrackerConfigError> {
        let bindings = self.collect_bindings();
        Self::check_for_conflicts(bindings)
    }

    /// Checks for socket address conflicts in the collected bindings
    ///
    /// Examines the binding map to find any addresses that have multiple
    /// services attempting to use them with the same protocol.
    ///
    /// # Errors
    ///
    /// Returns `TrackerConfigError::DuplicateSocketAddress` if any binding
    /// address is shared by multiple services.
    fn check_for_conflicts(
        bindings: HashMap<BindingAddress, Vec<String>>,
    ) -> Result<(), TrackerConfigError> {
        for (binding, services) in bindings {
            if services.len() > 1 {
                return Err(TrackerConfigError::DuplicateSocketAddress {
                    address: *binding.socket(),
                    protocol: binding.protocol(),
                    services,
                });
            }
        }

        Ok(())
    }

    /// Collects all binding addresses with their service names
    ///
    /// Creates a map of binding addresses (socket + protocol) to service names.
    /// This allows identifying which services are attempting to bind to the same
    /// socket address with the same protocol.
    fn collect_bindings(&self) -> HashMap<BindingAddress, Vec<String>> {
        let mut bindings: HashMap<BindingAddress, Vec<String>> = HashMap::new();

        // Add UDP trackers
        Self::register_trackers(
            &mut bindings,
            &self.udp_trackers,
            Protocol::Udp,
            "UDP Tracker",
        );

        // Add HTTP trackers
        Self::register_trackers(
            &mut bindings,
            &self.http_trackers,
            Protocol::Tcp,
            "HTTP Tracker",
        );

        // Add HTTP API
        Self::register_binding(
            &mut bindings,
            self.http_api.bind_address(),
            Protocol::Tcp,
            "HTTP API",
        );

        // Add Health Check API
        Self::register_binding(
            &mut bindings,
            self.health_check_api.bind_address(),
            Protocol::Tcp,
            "Health Check API",
        );

        bindings
    }

    /// Registers multiple tracker instances in the bindings map
    ///
    /// Creates numbered service names for each tracker instance (e.g., "UDP Tracker #1").
    fn register_trackers<T>(
        bindings: &mut HashMap<BindingAddress, Vec<String>>,
        trackers: &[T],
        protocol: Protocol,
        service_name: &str,
    ) where
        T: HasBindAddress,
    {
        for (i, tracker) in trackers.iter().enumerate() {
            let service_label = format!("{service_name} #{}", i + 1);
            Self::register_binding(bindings, tracker.bind_address(), protocol, &service_label);
        }
    }

    /// Registers a single binding in the bindings map
    ///
    /// Associates the given service name with the socket address and protocol.
    fn register_binding(
        bindings: &mut HashMap<BindingAddress, Vec<String>>,
        address: SocketAddr,
        protocol: Protocol,
        service_name: &str,
    ) {
        let binding = BindingAddress::new(address, protocol);
        bindings
            .entry(binding)
            .or_default()
            .push(service_name.to_string());
    }

    /// Returns the HTTP API TLS domain if configured
    #[must_use]
    pub fn http_api_tls_domain(&self) -> Option<&str> {
        self.http_api.tls_domain().map(DomainName::as_str)
    }

    /// Returns the HTTP API port number
    #[must_use]
    pub fn http_api_port(&self) -> u16 {
        self.http_api.bind_address().port()
    }

    /// Returns the Health Check API TLS domain if configured
    #[must_use]
    pub fn health_check_api_tls_domain(&self) -> Option<&str> {
        self.health_check_api.tls_domain()
    }

    /// Returns the Health Check API port number
    #[must_use]
    pub fn health_check_api_port(&self) -> u16 {
        self.health_check_api.bind_address().port()
    }

    /// Returns HTTP trackers that have TLS proxy enabled
    ///
    /// Returns a vector of tuples containing (domain, port) for each
    /// HTTP tracker that has `use_tls_proxy: true` and a domain configured.
    #[must_use]
    pub fn http_trackers_with_tls(&self) -> Vec<(&str, u16)> {
        self.http_trackers
            .iter()
            .filter(|tracker| tracker.use_tls_proxy())
            .filter_map(|tracker| {
                tracker
                    .domain()
                    .map(|domain| (domain.as_str(), tracker.bind_address().port()))
            })
            .collect()
    }

    /// Returns true if any HTTP tracker has `use_tls_proxy: true`
    ///
    /// This is used to determine if the tracker's global `on_reverse_proxy`
    /// setting should be enabled in the tracker configuration template.
    #[must_use]
    pub fn any_http_tracker_uses_tls_proxy(&self) -> bool {
        self.http_trackers
            .iter()
            .any(http::HttpTrackerConfig::use_tls_proxy)
    }

    /// Returns true if any service has TLS proxy configured
    ///
    /// Checks if at least one of the following services has TLS enabled:
    /// - HTTP API (`use_tls_proxy: true`)
    /// - Any HTTP tracker (`use_tls_proxy: true`)
    /// - Health Check API (`use_tls_proxy: true`)
    ///
    /// This is used for cross-service validation to ensure that when the HTTPS
    /// section is defined, at least one service actually uses TLS.
    #[must_use]
    pub fn has_any_tls_configured(&self) -> bool {
        self.http_api.use_tls_proxy()
            || self
                .http_trackers
                .iter()
                .any(http::HttpTrackerConfig::use_tls_proxy)
            || self.health_check_api.use_tls_proxy()
    }
}

impl PortDerivation for TrackerConfig {
    /// Derives port bindings for the Tracker service
    ///
    /// Implements PORT-01 through PORT-06:
    /// - PORT-01: Tracker needs ports if UDP OR HTTP without TLS OR API without TLS
    /// - PORT-02: UDP ports always exposed (UDP doesn't use TLS)
    /// - PORT-03: HTTP ports WITHOUT TLS exposed directly
    /// - PORT-04: HTTP ports WITH TLS NOT exposed (Caddy handles)
    /// - PORT-05: API port exposed only when no TLS
    /// - PORT-06: API port NOT exposed when TLS
    fn derive_ports(&self) -> Vec<PortBinding> {
        let mut ports = Vec::new();

        // PORT-02: UDP ports always exposed (UDP doesn't use TLS)
        for udp_tracker in &self.udp_trackers {
            ports.push(PortBinding::udp(
                udp_tracker.bind_address().port(),
                "BitTorrent UDP announce",
            ));
        }

        // PORT-03: HTTP ports WITHOUT TLS exposed directly
        // PORT-04: HTTP ports WITH TLS NOT exposed (Caddy handles)
        for http_tracker in &self.http_trackers {
            if !http_tracker.use_tls_proxy() {
                ports.push(PortBinding::tcp(
                    http_tracker.bind_address().port(),
                    "HTTP tracker announce",
                ));
            }
        }

        // PORT-05: API exposed only when no TLS
        // PORT-06: API NOT exposed when TLS
        if !self.http_api.use_tls_proxy() {
            ports.push(PortBinding::tcp(
                self.http_api.bind_address().port(),
                "HTTP API (stats/whitelist)",
            ));
        }

        ports
    }
}

impl NetworkDerivation for TrackerConfig {
    /// Derives network assignments for the Tracker service
    ///
    /// Implements NET-01 through NET-03:
    /// - NET-01: Metrics network if Prometheus enabled
    /// - NET-02: Database network if `MySQL` enabled
    /// - NET-03: Proxy network if Caddy enabled
    fn derive_networks(&self, enabled_services: &EnabledServices) -> Vec<Network> {
        let mut networks = Vec::new();

        // NET-01: Metrics network if Prometheus enabled
        if enabled_services.has(Service::Prometheus) {
            networks.push(Network::Metrics);
        }

        // NET-02: Database network if MySQL enabled
        if enabled_services.has(Service::MySQL) {
            networks.push(Network::Database);
        }

        // NET-03: Proxy network if Caddy enabled
        if enabled_services.has(Service::Caddy) {
            networks.push(Network::Proxy);
        }

        networks
    }
}

/// Trait for types that have a bind address
///
/// Used for generic tracker registration in validation logic.
trait HasBindAddress {
    /// Returns the socket address this service binds to
    fn bind_address(&self) -> SocketAddr;
}

impl HasBindAddress for UdpTrackerConfig {
    fn bind_address(&self) -> SocketAddr {
        UdpTrackerConfig::bind_address(self)
    }
}

impl HasBindAddress for HttpTrackerConfig {
    fn bind_address(&self) -> SocketAddr {
        HttpTrackerConfig::bind_address(self)
    }
}

impl Default for TrackerConfig {
    /// Returns a default tracker configuration suitable for development and testing
    ///
    /// # Default Values
    ///
    /// - Database: `SQLite` with filename "tracker.db"
    /// - Mode: Public tracker (private = false)
    /// - UDP trackers: One instance on port 6969
    /// - HTTP trackers: One instance on port 7070
    /// - HTTP API: Bind address 0.0.0.0:1212
    /// - Admin token: `MyAccessToken`
    fn default() -> Self {
        Self::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(
                    SqliteConfig::new("tracker.db").expect("default sqlite config is valid"),
                ),
                false,
            ),
            vec![
                UdpTrackerConfig::new("0.0.0.0:6969".parse().expect("valid address"), None)
                    .expect("default UdpTrackerConfig values are always valid"),
            ],
            vec![HttpTrackerConfig::new(
                "0.0.0.0:7070".parse().expect("valid address"),
                None,
                false,
            )
            .expect("default HttpTrackerConfig values are always valid")],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().expect("valid address"),
                "MyAccessToken".to_string().into(),
                None,
                false,
            )
            .expect("default HttpApiConfig values are always valid"),
            HealthCheckApiConfig::new(
                "127.0.0.1:1313".parse().expect("valid address"),
                None,
                false,
            )
            .expect("default HealthCheckApiConfig values are always valid"),
        )
        .expect("default TrackerConfig values have no socket address conflicts")
    }
}

pub(crate) fn serialize_socket_addr<S>(addr: &SocketAddr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&addr.to_string())
}

pub(crate) fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// Raw struct for deserializing `TrackerConfig` before validation.
#[derive(Deserialize)]
struct TrackerConfigRaw {
    core: TrackerCoreConfig,
    udp_trackers: Vec<UdpTrackerConfig>,
    http_trackers: Vec<HttpTrackerConfig>,
    http_api: HttpApiConfig,
    health_check_api: HealthCheckApiConfig,
}

impl<'de> Deserialize<'de> for TrackerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = TrackerConfigRaw::deserialize(deserializer)?;
        TrackerConfig::new(
            raw.core,
            raw.udp_trackers,
            raw.http_trackers,
            raw.http_api,
            raw.health_check_api,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create a `UdpTrackerConfig` with default values.
    /// Uses the validated constructor, making tests more realistic.
    fn test_udp_tracker_config(bind_address: &str) -> UdpTrackerConfig {
        UdpTrackerConfig::new(bind_address.parse().expect("valid address"), None)
            .expect("test values should be valid")
    }

    /// Test helper to create a `UdpTrackerConfig` with domain.
    #[allow(dead_code)]
    fn test_udp_tracker_config_with_domain(
        bind_address: &str,
        domain: DomainName,
    ) -> UdpTrackerConfig {
        UdpTrackerConfig::new(bind_address.parse().expect("valid address"), Some(domain))
            .expect("test values should be valid")
    }

    /// Test helper to create an `HttpTrackerConfig` with default values (no TLS).
    fn test_http_tracker_config(bind_address: &str) -> HttpTrackerConfig {
        HttpTrackerConfig::new(bind_address.parse().expect("valid address"), None, false)
            .expect("test values should be valid")
    }

    /// Test helper with TLS options for HTTP trackers.
    #[allow(dead_code)]
    fn test_http_tracker_config_with_tls(
        bind_address: &str,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> HttpTrackerConfig {
        HttpTrackerConfig::new(
            bind_address.parse().expect("valid address"),
            domain,
            use_tls_proxy,
        )
        .expect("test values should be valid")
    }

    /// Test helper to create a `HealthCheckApiConfig` with default values (no TLS).
    fn test_health_check_api_config(bind_address: &str) -> HealthCheckApiConfig {
        HealthCheckApiConfig::new(bind_address.parse().expect("valid address"), None, false)
            .expect("test values should be valid")
    }

    /// Test helper with TLS options for Health Check API.
    #[allow(dead_code)]
    fn test_health_check_api_config_with_tls(
        bind_address: &str,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> HealthCheckApiConfig {
        HealthCheckApiConfig::new(
            bind_address.parse().expect("valid address"),
            domain,
            use_tls_proxy,
        )
        .expect("test values should be valid")
    }

    /// Test helper to create an `HttpApiConfig` with default or custom values.
    /// Uses the validated constructor, making tests more realistic.
    fn test_http_api_config(bind_address: &str, admin_token: &str) -> HttpApiConfig {
        HttpApiConfig::new(
            bind_address.parse().expect("valid address"),
            admin_token.to_string().into(),
            None,
            false,
        )
        .expect("test values should be valid")
    }

    /// Test helper with TLS options
    fn test_http_api_config_with_tls(
        bind_address: &str,
        admin_token: &str,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> HttpApiConfig {
        HttpApiConfig::new(
            bind_address.parse().expect("valid address"),
            admin_token.to_string().into(),
            domain,
            use_tls_proxy,
        )
        .expect("test values should be valid")
    }

    /// Test helper to create a `TrackerConfig` with default values.
    /// Uses the validated constructor, making tests more realistic.
    fn test_tracker_config(
        udp_trackers: Vec<UdpTrackerConfig>,
        http_trackers: Vec<HttpTrackerConfig>,
        http_api: HttpApiConfig,
        health_check_api: HealthCheckApiConfig,
    ) -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            udp_trackers,
            http_trackers,
            http_api,
            health_check_api,
        )
        .expect("test values should be valid")
    }

    /// Test helper to create a `TrackerConfig` with custom core config.
    fn test_tracker_config_with_core(
        core: TrackerCoreConfig,
        udp_trackers: Vec<UdpTrackerConfig>,
        http_trackers: Vec<HttpTrackerConfig>,
        http_api: HttpApiConfig,
        health_check_api: HealthCheckApiConfig,
    ) -> TrackerConfig {
        TrackerConfig::new(
            core,
            udp_trackers,
            http_trackers,
            http_api,
            health_check_api,
        )
        .expect("test values should be valid")
    }

    /// Test helper to create a private core config (`SQLite`)
    fn test_private_core_config() -> TrackerCoreConfig {
        TrackerCoreConfig::new(
            DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
            true,
        )
    }

    /// Test helper to create a core config with custom database name
    fn test_core_config_with_db(database_name: &str) -> TrackerCoreConfig {
        TrackerCoreConfig::new(
            DatabaseConfig::Sqlite(SqliteConfig::new(database_name).unwrap()),
            false,
        )
    }

    mod is_localhost_tests {
        use super::*;

        #[test]
        fn it_should_detect_ipv4_localhost() {
            let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
            assert!(is_localhost(&addr));
        }

        #[test]
        fn it_should_detect_ipv6_localhost() {
            let addr: SocketAddr = "[::1]:8080".parse().unwrap();
            assert!(is_localhost(&addr));
        }

        #[test]
        fn it_should_not_detect_all_interfaces_ipv4() {
            let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
            assert!(!is_localhost(&addr));
        }

        #[test]
        fn it_should_not_detect_all_interfaces_ipv6() {
            let addr: SocketAddr = "[::]:8080".parse().unwrap();
            assert!(!is_localhost(&addr));
        }

        #[test]
        fn it_should_not_detect_specific_ip() {
            let addr: SocketAddr = "10.0.0.1:8080".parse().unwrap();
            assert!(!is_localhost(&addr));
        }

        #[test]
        fn it_should_not_detect_other_127_x_addresses() {
            // Only 127.0.0.1 is considered localhost, not the entire 127.0.0.0/8 range
            let addr: SocketAddr = "127.0.0.2:8080".parse().unwrap();
            assert!(!is_localhost(&addr));
        }
    }

    #[test]
    fn it_should_create_tracker_config() {
        let config = test_tracker_config_with_core(
            test_private_core_config(),
            vec![test_udp_tracker_config("0.0.0.0:6868")],
            vec![test_http_tracker_config("0.0.0.0:7070")],
            test_http_api_config("0.0.0.0:1212", "test_token"),
            test_health_check_api_config("127.0.0.1:1313"),
        );

        assert_eq!(config.core().database().database_name(), "tracker.db");
        assert!(config.core().private());
        assert_eq!(config.udp_trackers().len(), 1);
        assert_eq!(config.http_trackers().len(), 1);
    }

    #[test]
    fn it_should_serialize_tracker_config() {
        let config = test_tracker_config_with_core(
            test_core_config_with_db("test.db"),
            vec![],
            vec![],
            test_http_api_config("0.0.0.0:1212", "token123"),
            test_health_check_api_config("127.0.0.1:1313"),
        );

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["core"]["private"], false);
        assert_eq!(json["http_api"]["admin_token"], "token123");
    }

    #[test]
    fn it_should_create_default_tracker_config() {
        let config = TrackerConfig::default();

        // Verify default database configuration
        assert_eq!(config.core().database().database_name(), "tracker.db");
        assert_eq!(config.core().database().driver_name(), "sqlite3");

        // Verify public tracker mode
        assert!(!config.core().private());

        // Verify UDP trackers (1 instance)
        assert_eq!(config.udp_trackers().len(), 1);
        assert_eq!(
            config.udp_trackers()[0].bind_address(),
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP trackers (1 instance)
        assert_eq!(config.http_trackers().len(), 1);
        assert_eq!(
            config.http_trackers()[0].bind_address(),
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP API configuration
        assert_eq!(
            config.http_api().bind_address(),
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.http_api().admin_token().expose_secret(),
            "MyAccessToken"
        );
    }

    mod validation {
        use super::*;

        #[test]
        fn it_should_accept_valid_configuration_with_unique_addresses() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![test_udp_tracker_config("0.0.0.0:6969")],
                vec![test_http_tracker_config("0.0.0.0:7070")],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_reject_duplicate_udp_tracker_ports() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![
                    test_udp_tracker_config("0.0.0.0:7070"),
                    test_udp_tracker_config("0.0.0.0:7070"),
                ],
                vec![],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Udp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"UDP Tracker #1".to_string()));
                assert!(services.contains(&"UDP Tracker #2".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_duplicate_http_tracker_ports() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![
                    test_http_tracker_config("0.0.0.0:7070"),
                    test_http_tracker_config("0.0.0.0:7070"),
                ],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_http_tracker_and_api_conflict() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![test_http_tracker_config("0.0.0.0:7070")],
                test_http_api_config("0.0.0.0:7070", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"HTTP Tracker #1".to_string()));
                assert!(services.contains(&"HTTP API".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_http_tracker_and_health_check_api_conflict() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![test_http_tracker_config("0.0.0.0:1313")],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("0.0.0.0:1313"),
            );

            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:1313".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"HTTP Tracker #1".to_string()));
                assert!(services.contains(&"Health Check API".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_allow_udp_and_http_on_same_port() {
            // This is valid because UDP and TCP use separate port spaces
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![test_udp_tracker_config("0.0.0.0:7070")],
                vec![test_http_tracker_config("0.0.0.0:7070")],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_allow_same_port_different_ips() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![
                    test_http_tracker_config("192.168.1.10:7070"),
                    test_http_tracker_config("192.168.1.20:7070"),
                ],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_provide_clear_error_message_with_fix_instructions() {
            let result = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![test_http_tracker_config("0.0.0.0:7070")],
                test_http_api_config("0.0.0.0:7070", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            );

            let error = result.unwrap_err();
            let error_message = error.to_string();

            // Verify brief error message contains essential information
            assert!(error_message.contains("Socket address conflict"));
            assert!(error_message.contains("'HTTP Tracker #1'"));
            assert!(error_message.contains("'HTTP API'"));
            assert!(error_message.contains("0.0.0.0:7070"));
            assert!(error_message.contains("TCP"));
            assert!(error_message.contains("Tip: Assign different port numbers"));

            // Verify detailed help contains comprehensive troubleshooting
            let help = error.help();
            assert!(help.contains("Socket Address Conflict - Detailed Troubleshooting"));
            assert!(help.contains("Conflicting services:"));
            assert!(help.contains("HTTP Tracker #1"));
            assert!(help.contains("HTTP API"));
            assert!(help.contains("Why this fails:"));
            assert!(help.contains("How to fix:"));
            assert!(help.contains("docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md"));
        }
    }

    mod localhost_with_tls_validation {
        use super::*;

        fn base_config() -> TrackerConfig {
            test_tracker_config(
                vec![],
                vec![],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            )
        }

        // NOTE: Tests for localhost + TLS rejection have been moved to the individual
        // config type tests (health_check_api.rs, http.rs, http_api.rs) because
        // validation is now enforced at construction time by their respective ::new()
        // methods. TrackerConfig::new() no longer needs to check for localhost + TLS
        // as it's impossible to construct invalid child configs.

        #[test]
        fn it_should_allow_localhost_without_tls() {
            // base_config has http_api on 0.0.0.0 and health_check_api on 127.0.0.1 without TLS
            let config = base_config();
            // If we got here without error, the config is valid
            assert_eq!(config.http_api().bind_address().port(), 1212);
        }

        #[test]
        fn it_should_allow_non_localhost_with_tls() {
            let domain = crate::shared::DomainName::new("api.tracker.local").unwrap();
            let config = TrackerConfig::new(
                TrackerCoreConfig::new(
                    DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                    false,
                ),
                vec![],
                vec![],
                test_http_api_config_with_tls("0.0.0.0:1212", "token", Some(domain), true),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .expect("valid config");

            assert!(config.http_api().use_tls_proxy());
        }
    }

    // =========================================================================
    // Port derivation tests (PORT-01 through PORT-06)
    // =========================================================================

    mod port_derivation {
        use super::*;

        fn default_core() -> TrackerCoreConfig {
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            )
        }

        #[test]
        fn it_should_expose_udp_ports_always() {
            // PORT-02: UDP ports always exposed (UDP doesn't use TLS)
            let config = TrackerConfig::new(
                default_core(),
                vec![
                    test_udp_tracker_config("0.0.0.0:6969"),
                    test_udp_tracker_config("0.0.0.0:6868"),
                ],
                vec![],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            let udp_ports: Vec<_> = ports
                .iter()
                .filter(|p| p.protocol() == Protocol::Udp)
                .collect();

            assert_eq!(udp_ports.len(), 2);
            assert_eq!(udp_ports[0].host_port(), 6969);
            assert_eq!(udp_ports[1].host_port(), 6868);
        }

        #[test]
        fn it_should_expose_http_ports_without_tls() {
            // PORT-03: HTTP ports WITHOUT TLS exposed directly
            let config = TrackerConfig::new(
                default_core(),
                vec![],
                vec![
                    test_http_tracker_config("0.0.0.0:7070"),
                    test_http_tracker_config("0.0.0.0:8080"),
                ],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            // Should have 2 HTTP ports + 1 API port
            let tcp_ports: Vec<_> = ports
                .iter()
                .filter(|p| p.protocol() == Protocol::Tcp)
                .collect();

            assert_eq!(tcp_ports.len(), 3);
            assert!(tcp_ports.iter().any(|p| p.host_port() == 7070));
            assert!(tcp_ports.iter().any(|p| p.host_port() == 8080));
        }

        #[test]
        fn it_should_not_expose_http_ports_with_tls() {
            // PORT-04: HTTP ports WITH TLS NOT exposed (Caddy handles)
            let domain = crate::shared::DomainName::new("tracker.example.com").unwrap();
            let config = TrackerConfig::new(
                default_core(),
                vec![],
                vec![test_http_tracker_config_with_tls(
                    "0.0.0.0:7070",
                    Some(domain),
                    true,
                )],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            // Should only have API port (7070 is hidden behind TLS)
            assert!(ports.iter().all(|p| p.host_port() != 7070));
        }

        #[test]
        fn it_should_expose_api_port_without_tls() {
            // PORT-05: API exposed only when no TLS
            let config = TrackerConfig::new(
                default_core(),
                vec![],
                vec![],
                test_http_api_config("0.0.0.0:1212", "token"),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            let api_port = ports.iter().find(|p| p.host_port() == 1212);

            assert!(api_port.is_some());
            assert_eq!(
                api_port.unwrap().description(),
                "HTTP API (stats/whitelist)"
            );
        }

        #[test]
        fn it_should_not_expose_api_port_with_tls() {
            // PORT-06: API NOT exposed when TLS
            let domain = crate::shared::DomainName::new("api.example.com").unwrap();
            let config = TrackerConfig::new(
                default_core(),
                vec![],
                vec![],
                test_http_api_config_with_tls("0.0.0.0:1212", "token", Some(domain), true),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            // API port should not be exposed when TLS is enabled
            assert!(ports.iter().all(|p| p.host_port() != 1212));
        }

        #[test]
        fn it_should_return_empty_when_all_ports_hidden_by_tls() {
            // All services behind TLS = no exposed ports from tracker
            let api_domain = crate::shared::DomainName::new("api.example.com").unwrap();
            let tracker_domain = crate::shared::DomainName::new("tracker.example.com").unwrap();

            let config = TrackerConfig::new(
                default_core(),
                vec![], // No UDP
                vec![test_http_tracker_config_with_tls(
                    "0.0.0.0:7070",
                    Some(tracker_domain),
                    true,
                )],
                test_http_api_config_with_tls("0.0.0.0:1212", "token", Some(api_domain), true),
                test_health_check_api_config("127.0.0.1:1313"),
            )
            .unwrap();

            let ports = config.derive_ports();
            assert!(ports.is_empty());
        }

        #[test]
        fn it_should_include_descriptions_for_all_ports() {
            let config = TrackerConfig::default();

            let ports = config.derive_ports();

            for port in &ports {
                assert!(!port.description().is_empty());
            }
        }
    }
}
