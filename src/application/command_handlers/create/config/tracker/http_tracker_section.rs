use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HttpTrackerConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerSection {
    pub bind_address: String,
}

impl HttpTrackerSection {
    /// Converts this DTO to a domain `HttpTrackerConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    pub fn to_http_tracker_config(&self) -> Result<HttpTrackerConfig, CreateConfigError> {
        // Validate that the bind address can be parsed as SocketAddr
        let _bind_address = self.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: self.bind_address.clone(),
                source: e,
            }
        })?;

        // For now, keep as String since domain type still uses String
        // This will be updated in Step 0.7 when we enhance domain types
        Ok(HttpTrackerConfig {
            bind_address: self.bind_address.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_valid_bind_address_to_http_tracker_config() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.bind_address, "0.0.0.0:7070");
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpTrackerSection {
            bind_address: "not-valid".to_string(),
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "not-valid");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_be_serializable() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:7070"));
    }

    #[test]
    fn it_should_be_deserializable() {
        let json = r#"{"bind_address":"0.0.0.0:7070"}"#;
        let section: HttpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:7070");
    }
}
