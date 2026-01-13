//! HTTP tracker configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::domain::tls::TlsConfig;

/// HTTP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// TLS configuration for HTTPS termination via Caddy (optional)
    ///
    /// When present, this HTTP tracker will be accessible via HTTPS
    /// through the Caddy reverse proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_http_tracker_config() {
        let config = HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
            tls: None,
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_serialize_http_tracker_config() {
        let json = serde_json::to_value(&HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
            tls: None,
        })
        .unwrap();

        assert_eq!(json["bind_address"], "0.0.0.0:7070");
    }

    #[test]
    fn it_should_deserialize_http_tracker_config() {
        let json = r#"{"bind_address": "0.0.0.0:7070"}"#;
        let config: HttpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
    }
}
