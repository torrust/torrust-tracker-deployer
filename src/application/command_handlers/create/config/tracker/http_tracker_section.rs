use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HttpTrackerConfig;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HttpTrackerSection {
    pub bind_address: String,

    /// Domain name for HTTPS certificate acquisition
    ///
    /// When present along with `use_tls_proxy: true`, this HTTP tracker will be
    /// accessible via HTTPS through the Caddy reverse proxy using this domain.
    /// The domain is used for Let's Encrypt certificate acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Whether to proxy this service through Caddy with TLS termination
    ///
    /// When `true`:
    /// - The service is proxied through Caddy with HTTPS enabled
    /// - `domain` field is required
    /// - Cannot be used with localhost bind addresses (`127.0.0.1`, `::1`)
    /// - Implies the tracker's `on_reverse_proxy` should be `true`
    ///
    /// When `false` or omitted:
    /// - The service is accessed directly without TLS termination
    /// - `domain` field is optional (ignored if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tls_proxy: Option<bool>,
}

impl HttpTrackerSection {
    /// Converts this DTO to a domain `HttpTrackerConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
    /// Returns `CreateConfigError::InvalidDomain` if the domain is invalid.
    /// Returns `CreateConfigError::TlsProxyWithoutDomain` if `use_tls_proxy` is true but domain is missing.
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

        let use_tls_proxy = self.use_tls_proxy.unwrap_or(false);

        // Validate: use_tls_proxy: true requires domain
        if use_tls_proxy && self.domain.is_none() {
            return Err(CreateConfigError::TlsProxyWithoutDomain {
                service_type: "HTTP tracker".to_string(),
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

        Ok(HttpTrackerConfig {
            bind_address,
            domain,
            use_tls_proxy,
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
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy);
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpTrackerSection {
            bind_address: "not-valid".to_string(),
            domain: None,
            use_tls_proxy: None,
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
            domain: None,
            use_tls_proxy: None,
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
            domain: None,
            use_tls_proxy: None,
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
        assert!(section.domain.is_none());
        assert!(section.use_tls_proxy.is_none());
    }

    #[test]
    fn it_should_allow_non_localhost_with_tls_proxy() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: Some("tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let result = section.to_http_tracker_config();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.use_tls_proxy);
        assert!(config.domain.is_some());
    }

    #[test]
    fn it_should_reject_tls_proxy_without_domain() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::TlsProxyWithoutDomain {
            service_type,
            bind_address,
        }) = result
        {
            assert_eq!(service_type, "HTTP tracker");
            assert_eq!(bind_address, "0.0.0.0:7070");
        } else {
            panic!("Expected TlsProxyWithoutDomain error");
        }
    }

    #[test]
    fn it_should_accept_domain_without_tls_proxy() {
        // Domain provided but use_tls_proxy is false - domain is ignored
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: Some("tracker.local".to_string()),
            use_tls_proxy: Some(false),
        };

        let result = section.to_http_tracker_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.use_tls_proxy);
        // Domain is still stored but won't be used for TLS
        assert!(config.domain.is_some());
    }

    #[test]
    fn it_should_deserialize_with_new_fields() {
        let json = r#"{"bind_address":"0.0.0.0:7070","domain":"tracker.example.com","use_tls_proxy":true}"#;
        let section: HttpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:7070");
        assert_eq!(section.domain, Some("tracker.example.com".to_string()));
        assert_eq!(section.use_tls_proxy, Some(true));
    }
}
