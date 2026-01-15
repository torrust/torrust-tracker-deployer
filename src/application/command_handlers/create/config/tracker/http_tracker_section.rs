use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::application::command_handlers::create::config::https::TlsSection;
use crate::domain::tls::TlsConfig;
use crate::domain::tracker::HttpTrackerConfig;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HttpTrackerSection {
    pub bind_address: String,

    /// Optional TLS configuration for HTTPS
    ///
    /// When present, this HTTP tracker will be proxied through Caddy with HTTPS enabled.
    /// The domain specified will be used for Let's Encrypt certificate acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsSection>,
}

impl HttpTrackerSection {
    /// Converts this DTO to a domain `HttpTrackerConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
    /// Returns `CreateConfigError::InvalidDomain` if the TLS domain is invalid.
    ///
    /// Note: Localhost + TLS validation is performed at the domain layer
    /// (see `TrackerConfig::validate()`) to avoid duplicating business rules.
    pub fn to_http_tracker_config(&self) -> Result<HttpTrackerConfig, CreateConfigError> {
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

        Ok(HttpTrackerConfig { bind_address, tls })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_valid_bind_address_to_http_tracker_config() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            tls: None,
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpTrackerSection {
            bind_address: "not-valid".to_string(),
            tls: None,
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
    fn it_should_reject_port_zero() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:0".to_string(),
            tls: None,
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::DynamicPortNotSupported { bind_address }) = result {
            assert_eq!(bind_address, "0.0.0.0:0");
        } else {
            panic!("Expected DynamicPortNotSupported error");
        }
    }

    #[test]
    fn it_should_be_serializable() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            tls: None,
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

    #[test]
    fn it_should_allow_non_localhost_with_tls() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            tls: Some(TlsSection {
                domain: "tracker.local".to_string(),
            }),
        };

        let result = section.to_http_tracker_config();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.tls.is_some());
    }
}
