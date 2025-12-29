//! UDP tracker configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// UDP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
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
    fn it_should_create_udp_tracker_config() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6868".parse().unwrap(),
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6868".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_serialize_udp_tracker_config() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6969".parse().unwrap(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:6969");
    }

    #[test]
    fn it_should_deserialize_udp_tracker_config() {
        let json = r#"{"bind_address": "0.0.0.0:6969"}"#;
        let config: UdpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
    }
}
