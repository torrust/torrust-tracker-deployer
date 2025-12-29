//! HTTP tracker configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// HTTP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_http_tracker_config() {
        let config = HttpTrackerConfig {
            bind_address: "0.0.0.0:7070".parse().unwrap(),
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
