use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::UdpTrackerConfig;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct UdpTrackerSection {
    pub bind_address: String,

    /// Domain name for the UDP tracker (optional)
    ///
    /// When present, this domain can be used in announce URLs instead of the IP.
    /// Example: `udp://tracker.example.com:6969/announce`
    ///
    /// Note: Unlike HTTP trackers, UDP does not support TLS, so there is no
    /// `use_tls_proxy` field for UDP trackers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl UdpTrackerSection {
    /// Converts this DTO to a domain `UdpTrackerConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
    /// Returns `CreateConfigError::InvalidDomain` if the domain is invalid.
    pub fn to_udp_tracker_config(&self) -> Result<UdpTrackerConfig, CreateConfigError> {
        // Validate that the bind address can be parsed as SocketAddr
        let bind_address = self.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: self.bind_address.clone(),
                source: e,
            }
        })?;

        // Reject port 0 (dynamic port assignment)
        if bind_address.port() == 0 {
            return Err(CreateConfigError::DynamicPortNotSupported {
                bind_address: self.bind_address.clone(),
            });
        }

        // Convert domain to domain type with validation (if present)
        let domain = match &self.domain {
            Some(domain_str) => {
                let domain =
                    DomainName::new(domain_str).map_err(|e| CreateConfigError::InvalidDomain {
                        domain: domain_str.clone(),
                        reason: e.to_string(),
                    })?;
                Some(domain)
            }
            None => None,
        };

        Ok(UdpTrackerConfig {
            bind_address,
            domain,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_valid_bind_address_to_udp_tracker_config() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: None,
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain.is_none());
    }

    #[test]
    fn it_should_convert_with_valid_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some("udp.tracker.local".to_string()),
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.domain.as_ref().map(DomainName::as_str),
            Some("udp.tracker.local")
        );
    }

    #[test]
    fn it_should_fail_for_invalid_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some(String::new()), // Empty domain is invalid
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidDomain { domain, .. }) = result {
            assert_eq!(domain, "");
        } else {
            panic!("Expected InvalidDomain error");
        }
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = UdpTrackerSection {
            bind_address: "invalid".to_string(),
            domain: None,
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "invalid");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_reject_port_zero() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:0".to_string(),
            domain: None,
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::DynamicPortNotSupported { bind_address }) = result {
            assert_eq!(bind_address, "0.0.0.0:0");
        } else {
            panic!("Expected DynamicPortNotSupported error");
        }
    }

    #[test]
    fn it_should_be_serializable_without_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: None,
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:6969"));
        // domain should not be present when None (skip_serializing_if)
        assert!(!json.contains("domain"));
    }

    #[test]
    fn it_should_be_serializable_with_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some("udp.tracker.local".to_string()),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:6969"));
        assert!(json.contains("domain"));
        assert!(json.contains("udp.tracker.local"));
    }

    #[test]
    fn it_should_be_deserializable_without_domain() {
        let json = r#"{"bind_address":"0.0.0.0:6969"}"#;
        let section: UdpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:6969");
        assert!(section.domain.is_none());
    }

    #[test]
    fn it_should_be_deserializable_with_domain() {
        let json = r#"{"bind_address":"0.0.0.0:6969","domain":"udp.tracker.local"}"#;
        let section: UdpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:6969");
        assert_eq!(section.domain, Some("udp.tracker.local".to_string()));
    }
}
