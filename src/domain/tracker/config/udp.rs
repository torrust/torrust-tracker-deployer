//! UDP tracker configuration

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::shared::DomainName;

/// UDP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Domain name for announce URLs (optional)
    ///
    /// When present, this domain can be used when communicating the tracker's
    /// announce URL to users, e.g., `udp://tracker.example.com:6969/announce`
    ///
    /// Note: Unlike HTTP trackers, UDP does not support TLS, so there is no
    /// `use_tls_proxy` field for UDP trackers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<DomainName>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_udp_tracker_config_without_domain() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6868".parse().unwrap(),
            domain: None,
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6868".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain.is_none());
    }

    #[test]
    fn it_should_create_udp_tracker_config_with_domain() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6969".parse().unwrap(),
            domain: Some(DomainName::new("tracker.example.com").unwrap()),
        };

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.domain.as_ref().map(DomainName::as_str),
            Some("tracker.example.com")
        );
    }

    #[test]
    fn it_should_serialize_udp_tracker_config_without_domain() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6969".parse().unwrap(),
            domain: None,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:6969");
        // domain should not be present when None (skip_serializing_if)
        assert!(json.get("domain").is_none());
    }

    #[test]
    fn it_should_serialize_udp_tracker_config_with_domain() {
        let config = UdpTrackerConfig {
            bind_address: "0.0.0.0:6969".parse().unwrap(),
            domain: Some(DomainName::new("udp.tracker.local").unwrap()),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:6969");
        assert_eq!(json["domain"], "udp.tracker.local");
    }

    #[test]
    fn it_should_deserialize_udp_tracker_config_without_domain() {
        let json = r#"{"bind_address": "0.0.0.0:6969"}"#;
        let config: UdpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain.is_none());
    }

    #[test]
    fn it_should_deserialize_udp_tracker_config_with_domain() {
        let json = r#"{"bind_address": "0.0.0.0:6969", "domain": "udp.tracker.local"}"#;
        let config: UdpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.domain.as_ref().map(DomainName::as_str),
            Some("udp.tracker.local")
        );
    }
}
