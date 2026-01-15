//! Health Check API configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::domain::tls::TlsConfig;

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

    /// TLS configuration for HTTPS termination via Caddy (optional)
    ///
    /// When present, the Health Check API will be accessible via HTTPS
    /// through the Caddy reverse proxy. This is useful when exposing
    /// health checks to external monitoring systems or load balancers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

impl HealthCheckApiConfig {
    /// Returns the TLS domain if configured
    #[must_use]
    pub fn tls_domain(&self) -> Option<&str> {
        self.tls.as_ref().map(TlsConfig::domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain_name::DomainName;

    #[test]
    fn it_should_create_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
            tls: None,
        };

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(config.tls.is_none());
    }

    #[test]
    fn it_should_create_health_check_api_config_with_tls() {
        let domain = DomainName::new("health.tracker.local").unwrap();
        let config = HealthCheckApiConfig {
            bind_address: "0.0.0.0:1313".parse().unwrap(),
            tls: Some(TlsConfig::new(domain)),
        };

        assert!(config.tls.is_some());
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }

    #[test]
    fn it_should_serialize_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
            tls: None,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "127.0.0.1:1313");
        // tls should not be serialized when None
        assert!(json.get("tls").is_none());
    }

    #[test]
    fn it_should_serialize_health_check_api_config_with_tls() {
        let domain = DomainName::new("health.tracker.local").unwrap();
        let config = HealthCheckApiConfig {
            bind_address: "0.0.0.0:1313".parse().unwrap(),
            tls: Some(TlsConfig::new(domain)),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:1313");
        assert_eq!(json["tls"]["domain"], "health.tracker.local");
    }

    #[test]
    fn it_should_deserialize_health_check_api_config() {
        let json = r#"{"bind_address": "127.0.0.1:1313"}"#;
        let config: HealthCheckApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(config.tls.is_none());
    }

    #[test]
    fn it_should_deserialize_health_check_api_config_with_tls() {
        let json = r#"{"bind_address": "0.0.0.0:1313", "tls": {"domain": "health.tracker.local"}}"#;
        let config: HealthCheckApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1313".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }
}
