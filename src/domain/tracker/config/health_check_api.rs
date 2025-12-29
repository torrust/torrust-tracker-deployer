//! Health Check API configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
        };

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_serialize_health_check_api_config() {
        let config = HealthCheckApiConfig {
            bind_address: "127.0.0.1:1313".parse().unwrap(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "127.0.0.1:1313");
    }

    #[test]
    fn it_should_deserialize_health_check_api_config() {
        let json = r#"{"bind_address": "127.0.0.1:1313"}"#;
        let config: HealthCheckApiConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
    }
}
