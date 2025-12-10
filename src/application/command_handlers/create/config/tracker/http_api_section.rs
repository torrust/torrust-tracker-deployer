use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HttpApiConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiSection {
    pub bind_address: String,
    pub admin_token: String,
}

impl HttpApiSection {
    /// Converts this DTO to a domain `HttpApiConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    pub fn to_http_api_config(&self) -> Result<HttpApiConfig, CreateConfigError> {
        // Validate that the bind address can be parsed as SocketAddr
        let bind_address = self.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: self.bind_address.clone(),
                source: e,
            }
        })?;

        // Domain type now uses SocketAddr (Step 0.7 completed)
        Ok(HttpApiConfig {
            bind_address,
            admin_token: self.admin_token.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_valid_config_to_http_api_config() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "MyAccessToken".to_string(),
        };

        let result = section.to_http_api_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token, "MyAccessToken");
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpApiSection {
            bind_address: "invalid-address".to_string(),
            admin_token: "token".to_string(),
        };

        let result = section.to_http_api_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "invalid-address");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_be_serializable() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "MyAccessToken".to_string(),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:1212"));
        assert!(json.contains("admin_token"));
        assert!(json.contains("MyAccessToken"));
    }

    #[test]
    fn it_should_be_deserializable() {
        let json = r#"{"bind_address":"0.0.0.0:1212","admin_token":"MyAccessToken"}"#;
        let section: HttpApiSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:1212");
        assert_eq!(section.admin_token, "MyAccessToken");
    }
}
