use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::application::command_handlers::create::config::https::TlsSection;
use crate::domain::tls::TlsConfig;
use crate::domain::tracker::HttpApiConfig;
use crate::shared::secrets::PlainApiToken;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HttpApiSection {
    pub bind_address: String,
    pub admin_token: PlainApiToken,

    /// Optional TLS configuration for HTTPS
    ///
    /// When present, this service will be proxied through Caddy with HTTPS enabled.
    /// The domain specified will be used for Let's Encrypt certificate acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsSection>,
}

impl HttpApiSection {
    /// Converts this DTO to a domain `HttpApiConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
    /// Returns `CreateConfigError::InvalidDomain` if the TLS domain is invalid.
    pub fn to_http_api_config(&self) -> Result<HttpApiConfig, CreateConfigError> {
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

        // Convert TLS section to domain type with validation
        let tls = match &self.tls {
            Some(tls_section) => {
                tls_section.validate()?;
                let domain = DomainName::new(&tls_section.domain).map_err(|e| {
                    CreateConfigError::InvalidDomain {
                        domain: tls_section.domain.clone(),
                        reason: e.to_string(),
                    }
                })?;
                Some(TlsConfig::new(domain))
            }
            None => None,
        };

        Ok(HttpApiConfig {
            bind_address,
            admin_token: self.admin_token.clone().into(),
            tls,
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
            tls: None,
        };

        let result = section.to_http_api_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "MyAccessToken");
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpApiSection {
            bind_address: "invalid-address".to_string(),
            admin_token: "token".to_string(),
            tls: None,
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
    fn it_should_reject_port_zero() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:0".to_string(),
            admin_token: "token".to_string(),
            tls: None,
        };

        let result = section.to_http_api_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::DynamicPortNotSupported { bind_address }) = result {
            assert_eq!(bind_address, "0.0.0.0:0");
        } else {
            panic!("Expected DynamicPortNotSupported error");
        }
    }

    #[test]
    fn it_should_be_serializable() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "MyAccessToken".to_string(),
            tls: None,
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
