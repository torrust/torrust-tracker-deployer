//! HTTP API configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::domain::tls::TlsConfig;
use crate::shared::ApiToken;

/// HTTP API configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiConfig {
    /// Bind address (e.g., "0.0.0.0:1212")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Admin access token for HTTP API authentication
    pub admin_token: ApiToken,

    /// TLS configuration for HTTPS termination via Caddy (optional)
    ///
    /// When present, the HTTP API will be accessible via HTTPS
    /// through the Caddy reverse proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_http_api_config() {
        let config = HttpApiConfig {
            bind_address: "0.0.0.0:1212".parse().unwrap(),
            admin_token: "test_token".to_string().into(),
            tls: None,
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "test_token");
    }

    #[test]
    fn it_should_serialize_http_api_config() {
        let config = HttpApiConfig {
            bind_address: "0.0.0.0:1212".parse().unwrap(),
            admin_token: "token123".to_string().into(),
            tls: None,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:1212");
        assert_eq!(json["admin_token"], "token123");
    }

    #[test]
    fn it_should_deserialize_http_api_config() {
        let json = r#"{"bind_address": "0.0.0.0:1212", "admin_token": "MyToken"}"#;
        let config: HttpApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "MyToken");
    }
}
