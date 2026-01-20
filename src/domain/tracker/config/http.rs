//! HTTP tracker configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::shared::DomainName;

/// HTTP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Domain name for HTTPS certificate acquisition (optional)
    ///
    /// When present along with `use_tls_proxy: true`, this HTTP tracker will be
    /// accessible via HTTPS through the Caddy reverse proxy using this domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<DomainName>,

    /// Whether to proxy this service through Caddy with TLS termination
    ///
    /// When `true`:
    /// - The service is proxied through Caddy with HTTPS enabled
    /// - `domain` field is required
    /// - Cannot be used with localhost bind addresses (`127.0.0.1`, `::1`)
    /// - Implies the tracker's `on_reverse_proxy` should be `true`
    pub use_tls_proxy: bool,
}

impl HttpTrackerConfig {
    /// Returns true if this tracker uses the TLS proxy
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
    fn it_should_create_http_tracker_config_without_tls() {
        let config = HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
            domain: None,
            use_tls_proxy: false,
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.uses_tls_proxy());
        assert!(config.tls_domain().is_none());
    }

    #[test]
    fn it_should_create_http_tracker_config_with_tls() {
        let config = HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
            domain: Some(DomainName::new("tracker.example.com").unwrap()),
            use_tls_proxy: true,
        };

        assert!(config.uses_tls_proxy());
        assert_eq!(
            config.tls_domain().map(DomainName::as_str),
            Some("tracker.example.com")
        );
    }

    #[test]
    fn it_should_serialize_http_tracker_config() {
        let json = serde_json::to_value(&HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
            domain: None,
            use_tls_proxy: false,
        })
        .unwrap();

        assert_eq!(json["bind_address"], "0.0.0.0:7070");
        assert_eq!(json["use_tls_proxy"], false);
    }

    #[test]
    fn it_should_deserialize_http_tracker_config() {
        let json = r#"{"bind_address": "0.0.0.0:7070", "use_tls_proxy": false}"#;
        let config: HttpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy);
    }
}
