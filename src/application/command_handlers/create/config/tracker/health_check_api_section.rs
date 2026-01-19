use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HealthCheckApiConfig;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HealthCheckApiSection {
    pub bind_address: String,

    /// Domain name for HTTPS access via Caddy reverse proxy
    ///
    /// When present with `use_tls_proxy: true`, this service will be accessible
    /// via HTTPS at this domain. The domain will be used for Let's Encrypt
    /// certificate acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Whether to proxy this service through Caddy with TLS termination
    ///
    /// When `true`, the service will be accessible via HTTPS through Caddy.
    /// Requires `domain` to be set.
    /// This is useful for exposing health checks to external monitoring systems.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tls_proxy: Option<bool>,
}

impl HealthCheckApiSection {
    /// Converts this DTO to a domain `HealthCheckApiConfig`
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
    pub fn to_health_check_api_config(&self) -> Result<HealthCheckApiConfig, CreateConfigError> {
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

        // Validate: use_tls_proxy requires domain
        if use_tls_proxy && self.domain.is_none() {
            return Err(CreateConfigError::TlsProxyWithoutDomain {
                service_type: "Health Check API".to_string(),
                bind_address: self.bind_address.clone(),
            });
        }

        // Parse domain if present
        let domain =
            match &self.domain {
                Some(domain_str) => Some(DomainName::new(domain_str).map_err(|e| {
                    CreateConfigError::InvalidDomain {
                        domain: domain_str.clone(),
                        reason: e.to_string(),
                    }
                })?),
                None => None,
            };

        Ok(HealthCheckApiConfig {
            bind_address,
            domain,
            use_tls_proxy,
        })
    }
}

impl Default for HealthCheckApiSection {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:1313".to_string(),
            domain: None,
            use_tls_proxy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_to_domain_config_when_bind_address_is_valid() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:1313".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let config = section.to_health_check_api_config().unwrap();

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy);
        assert!(config.domain.is_none());
    }

    #[test]
    fn it_should_convert_to_domain_config_with_tls_proxy() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: Some("health.tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let config = section.to_health_check_api_config().unwrap();

        assert_eq!(
            config.bind_address,
            "0.0.0.0:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(config.use_tls_proxy);
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }

    #[test]
    fn it_should_fail_when_bind_address_is_invalid() {
        let section = HealthCheckApiSection {
            bind_address: "invalid".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidBindAddress { .. }
        ));
    }

    #[test]
    fn it_should_reject_dynamic_port_assignment() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:0".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::DynamicPortNotSupported { .. }
        ));
    }

    #[test]
    fn it_should_allow_ipv6_addresses() {
        let section = HealthCheckApiSection {
            bind_address: "[::1]:1313".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_allow_any_port_except_zero() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:8080".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_provide_default_localhost_1313() {
        let section = HealthCheckApiSection::default();

        assert_eq!(section.bind_address, "127.0.0.1:1313");
        assert!(section.domain.is_none());
        assert!(section.use_tls_proxy.is_none());
    }

    #[test]
    fn it_should_fail_when_domain_is_invalid() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: Some("invalid domain with spaces".to_string()),
            use_tls_proxy: Some(true),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidDomain { .. }
        ));
    }

    #[test]
    fn it_should_fail_when_use_tls_proxy_without_domain() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::TlsProxyWithoutDomain { .. }
        ));
    }

    #[test]
    fn it_should_allow_domain_without_tls_proxy() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: Some("health.tracker.local".to_string()),
            use_tls_proxy: None,
        };

        let config = section.to_health_check_api_config().unwrap();

        assert!(!config.use_tls_proxy);
        assert!(config.domain.is_some());
    }
}
