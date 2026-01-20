use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HttpApiConfig;
use crate::shared::secrets::PlainApiToken;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HttpApiSection {
    pub bind_address: String,
    pub admin_token: PlainApiToken,

    /// Domain name for HTTPS certificate acquisition
    ///
    /// When present along with `use_tls_proxy: true`, this service will be
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
    ///
    /// When `false` or omitted:
    /// - The service is accessed directly without TLS termination
    /// - `domain` field is optional (ignored if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tls_proxy: Option<bool>,
}

impl HttpApiSection {
    /// Converts this DTO to a domain `HttpApiConfig`
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

        let use_tls_proxy = self.use_tls_proxy.unwrap_or(false);

        // Validate: use_tls_proxy: true requires domain
        if use_tls_proxy && self.domain.is_none() {
            return Err(CreateConfigError::TlsProxyWithoutDomain {
                service_type: "HTTP API".to_string(),
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

        Ok(HttpApiConfig {
            bind_address,
            admin_token: self.admin_token.clone().into(),
            domain,
            use_tls_proxy,
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
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_http_api_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token.expose_secret(), "MyAccessToken");
        assert!(!config.use_tls_proxy);
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpApiSection {
            bind_address: "invalid-address".to_string(),
            admin_token: "token".to_string(),
            domain: None,
            use_tls_proxy: None,
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
            domain: None,
            use_tls_proxy: None,
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
            domain: None,
            use_tls_proxy: None,
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
        assert!(section.domain.is_none());
        assert!(section.use_tls_proxy.is_none());
    }

    #[test]
    fn it_should_allow_non_localhost_with_tls_proxy() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some("api.tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let result = section.to_http_api_config();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.use_tls_proxy);
        assert!(config.domain.is_some());
    }

    #[test]
    fn it_should_reject_tls_proxy_without_domain() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result = section.to_http_api_config();
        assert!(result.is_err());

        if let Err(CreateConfigError::TlsProxyWithoutDomain {
            service_type,
            bind_address,
        }) = result
        {
            assert_eq!(service_type, "HTTP API");
            assert_eq!(bind_address, "0.0.0.0:1212");
        } else {
            panic!("Expected TlsProxyWithoutDomain error");
        }
    }

    #[test]
    fn it_should_accept_domain_without_tls_proxy() {
        // Domain provided but use_tls_proxy is false - domain is ignored
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some("api.tracker.local".to_string()),
            use_tls_proxy: Some(false),
        };

        let result = section.to_http_api_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.use_tls_proxy);
        // Domain is still stored but won't be used for TLS
        assert!(config.domain.is_some());
    }

    #[test]
    fn it_should_deserialize_with_new_fields() {
        let json = r#"{"bind_address":"0.0.0.0:1212","admin_token":"token","domain":"api.example.com","use_tls_proxy":true}"#;
        let section: HttpApiSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:1212");
        assert_eq!(section.domain, Some("api.example.com".to_string()));
        assert_eq!(section.use_tls_proxy, Some(true));
    }
}
