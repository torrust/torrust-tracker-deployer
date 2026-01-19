//! Health Check API configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::shared::domain_name::DomainName;

/// Health Check API configuration
///
/// The Health Check API is a minimal HTTP endpoint used by Docker and container
/// orchestration tools to verify service health. It's separate from the main HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheckApiConfig {
    /// Bind address (e.g., "127.0.0.1:1313")
    ///
    /// Conventionally uses port 1313, though this is configurable
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Domain name for external HTTPS access (optional)
    ///
    /// When present, defines the domain at which this service will be accessible.
    /// Caddy uses this for automatic certificate management.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<DomainName>,

    /// Whether to use TLS proxy via Caddy (default: false)
    ///
    /// When true:
    /// - Caddy handles HTTPS termination with automatic certificates
    /// - Requires a domain to be configured
    /// - Service receives plain HTTP from Caddy internally
    #[serde(default)]
    pub use_tls_proxy: bool,
}

impl HealthCheckApiConfig {
    /// Returns whether TLS proxy is enabled
    #[must_use]
    pub fn uses_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }

    /// Returns the TLS domain if TLS proxy is configured
    #[must_use]
    pub fn tls_domain(&self) -> Option<&str> {
        if self.use_tls_proxy {
            self.domain.as_ref().map(DomainName::as_str)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
            domain: None,
            use_tls_proxy: false,
        };

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy);
        assert!(config.domain.is_none());
    }

    #[test]
    fn it_should_create_health_check_api_config_with_tls_proxy() {
        let domain = DomainName::new("health.tracker.local").unwrap();
        let config = HealthCheckApiConfig {
            bind_address: "0.0.0.0:1313".parse().unwrap(),
            domain: Some(domain),
            use_tls_proxy: true,
        };

        assert!(config.uses_tls_proxy());
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }

    #[test]
    fn it_should_serialize_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
            domain: None,
            use_tls_proxy: false,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "127.0.0.1:1313");
        // domain should not be serialized when None
        assert!(json.get("domain").is_none());
        // use_tls_proxy should be serialized
        assert_eq!(json["use_tls_proxy"], false);
    }

    #[test]
    fn it_should_serialize_health_check_api_config_with_tls_proxy() {
        let domain = DomainName::new("health.tracker.local").unwrap();
        let config = HealthCheckApiConfig {
            bind_address: "0.0.0.0:1313".parse().unwrap(),
            domain: Some(domain),
            use_tls_proxy: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:1313");
        assert_eq!(json["domain"], "health.tracker.local");
        assert_eq!(json["use_tls_proxy"], true);
    }

    #[test]
    fn it_should_deserialize_health_check_api_config() {
        let json = r#"{"bind_address": "127.0.0.1:1313", "use_tls_proxy": false}"#;
        let config: HealthCheckApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy);
    }

    #[test]
    fn it_should_deserialize_health_check_api_config_with_tls_proxy() {
        let json = r#"{"bind_address": "0.0.0.0:1313", "domain": "health.tracker.local", "use_tls_proxy": true}"#;
        let config: HealthCheckApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1313".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }

    #[test]
    fn it_should_return_none_for_tls_domain_when_tls_proxy_disabled() {
        let domain = DomainName::new("health.tracker.local").unwrap();
        let config = HealthCheckApiConfig {
            bind_address: "0.0.0.0:1313".parse().unwrap(),
            domain: Some(domain),
            use_tls_proxy: false,
        };

        assert!(!config.uses_tls_proxy());
        assert!(config.tls_domain().is_none());
    }
}
