//! HTTP API configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::shared::{ApiToken, DomainName};

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

    /// Domain name for HTTPS certificate acquisition (optional)
    ///
    /// When present along with `use_tls_proxy: true`, this HTTP API will be
    /// accessible via HTTPS through the Caddy reverse proxy using this domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<DomainName>,

    /// Whether to proxy this service through Caddy with TLS termination
    ///
    /// When `true`:
    /// - The service is proxied through Caddy with HTTPS enabled
    /// - `domain` field is required
    /// - Cannot be used with localhost bind addresses (`127.0.0.1`, `::1`)
    pub use_tls_proxy: bool,
}

impl HttpApiConfig {
    /// Returns true if this API uses the TLS proxy
    #[must_use]
    pub fn uses_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }

    /// Returns the domain name if TLS proxy is enabled
    #[must_use]
    pub fn tls_domain(&self) -> Option<&DomainName> {
        if self.use_tls_proxy {
            self.domain.as_ref()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_http_api_config_without_tls() {
        let config = HttpApiConfig {
            bind_address: "0.0.0.0:1212".parse().unwrap(),
            admin_token: "test_token".to_string().into(),
            domain: None,
            use_tls_proxy: false,
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "test_token");
        assert!(!config.uses_tls_proxy());
        assert!(config.tls_domain().is_none());
    }

    #[test]
    fn it_should_create_http_api_config_with_tls() {
        let config = HttpApiConfig {
            bind_address: "0.0.0.0:1212".parse().unwrap(),
            admin_token: "test_token".to_string().into(),
            domain: Some(DomainName::new("api.example.com").unwrap()),
            use_tls_proxy: true,
        };

        assert!(config.uses_tls_proxy());
        assert_eq!(
            config.tls_domain().map(DomainName::as_str),
            Some("api.example.com")
        );
    }

    #[test]
    fn it_should_serialize_http_api_config() {
        let config = HttpApiConfig {
            bind_address: "0.0.0.0:1212".parse().unwrap(),
            admin_token: "token123".to_string().into(),
            domain: None,
            use_tls_proxy: false,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:1212");
        assert_eq!(json["admin_token"], "token123");
        assert_eq!(json["use_tls_proxy"], false);
    }

    #[test]
    fn it_should_deserialize_http_api_config() {
        let json =
            r#"{"bind_address": "0.0.0.0:1212", "admin_token": "MyToken", "use_tls_proxy": false}"#;
        let config: HttpApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "MyToken");
        assert!(!config.use_tls_proxy);
    }
}
