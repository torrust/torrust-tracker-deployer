use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::UdpTrackerConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerSection {
    pub bind_address: String,
}

impl UdpTrackerSection {
    /// Converts this DTO to a domain `UdpTrackerConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
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

        // Domain type now uses SocketAddr (Step 0.7 completed)
        Ok(UdpTrackerConfig { bind_address })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_valid_bind_address_to_udp_tracker_config() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
        };

        let result = section.to_udp_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = UdpTrackerSection {
            bind_address: "invalid".to_string(),
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
    fn it_should_be_serializable() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:6969"));
    }

    #[test]
    fn it_should_be_deserializable() {
        let json = r#"{"bind_address":"0.0.0.0:6969"}"#;
        let section: UdpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:6969");
    }
}
