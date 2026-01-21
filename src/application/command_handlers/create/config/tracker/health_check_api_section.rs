//! Health Check API section DTO
//!
//! This module contains the application layer DTO for Health Check API configuration.
//! It follows the **`TryFrom` pattern** for DTO to domain conversion, delegating
//! all business validation to the domain layer.
//!
//! ## Conversion Pattern
//!
//! The `TryFrom<HealthCheckApiSection> for HealthCheckApiConfig` implementation:
//! 1. Parses string fields into typed values (e.g., `String` → `SocketAddr`)
//! 2. Delegates domain validation to `HealthCheckApiConfig::new()`
//! 3. Maps domain errors to application errors via `From` implementations
//!
//! See `docs/decisions/tryfrom-for-dto-to-domain-conversion.md` for rationale.

use std::convert::TryFrom;
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

/// Converts from application DTO to domain type using `TryFrom` trait
///
/// This implementation follows the standard library convention for fallible
/// conversions, enabling use of `.try_into()` and `TryFrom::try_from()`.
///
/// # Example
///
/// ```rust,ignore
/// let section = HealthCheckApiSection {
///     bind_address: "127.0.0.1:1313".to_string(),
///     domain: None,
///     use_tls_proxy: None,
/// };
/// let config: HealthCheckApiConfig = section.try_into()?;
/// ```
impl TryFrom<HealthCheckApiSection> for HealthCheckApiConfig {
    type Error = CreateConfigError;

    fn try_from(section: HealthCheckApiSection) -> Result<Self, Self::Error> {
        // Parse bind address from string to SocketAddr
        let bind_address = section.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: section.bind_address.clone(),
                source: e,
            }
        })?;

        // Parse domain if present
        let domain = section
            .domain
            .map(|d| {
                DomainName::new(&d).map_err(|e| CreateConfigError::InvalidDomain {
                    domain: d,
                    reason: e.to_string(),
                })
            })
            .transpose()?;

        let use_tls_proxy = section.use_tls_proxy.unwrap_or(false);

        // Delegate all business validation to domain layer
        HealthCheckApiConfig::new(bind_address, domain, use_tls_proxy)
            .map_err(CreateConfigError::from)
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

    // =========================================================================
    // TryFrom conversion tests
    // =========================================================================

    #[test]
    fn it_should_convert_to_domain_config_when_bind_address_is_valid() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:1313".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let config: HealthCheckApiConfig = section.try_into().unwrap();

        assert_eq!(
            config.bind_address(),
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy());
        assert!(config.domain().is_none());
    }

    #[test]
    fn it_should_convert_to_domain_config_with_tls_proxy() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: Some("health.tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let config: HealthCheckApiConfig = section.try_into().unwrap();

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:1313".parse::<SocketAddr>().unwrap()
        );
        assert!(config.use_tls_proxy());
        assert_eq!(config.tls_domain(), Some("health.tracker.local"));
    }

    #[test]
    fn it_should_fail_when_bind_address_is_invalid() {
        let section = HealthCheckApiSection {
            bind_address: "invalid".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidBindAddress { .. }
        ));
    }

    #[test]
    fn it_should_reject_dynamic_port_assignment_via_domain_validation() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:0".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_err());
        // Port 0 is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HealthCheckApiConfigInvalid(_)
        ));
    }

    #[test]
    fn it_should_allow_ipv6_addresses() {
        let section = HealthCheckApiSection {
            bind_address: "[::1]:1313".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_allow_any_port_except_zero() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:8080".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

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

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidDomain { .. }
        ));
    }

    #[test]
    fn it_should_fail_when_use_tls_proxy_without_domain_via_domain_validation() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_err());
        // TLS without domain is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HealthCheckApiConfigInvalid(_)
        ));
    }

    #[test]
    fn it_should_reject_localhost_with_tls_via_domain_validation() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:1313".to_string(),
            domain: Some("health.tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let result: Result<HealthCheckApiConfig, _> = section.try_into();

        assert!(result.is_err());
        // Localhost + TLS is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HealthCheckApiConfigInvalid(_)
        ));
    }

    #[test]
    fn it_should_allow_domain_without_tls_proxy() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:1313".to_string(),
            domain: Some("health.tracker.local".to_string()),
            use_tls_proxy: None,
        };

        let config: HealthCheckApiConfig = section.try_into().unwrap();

        assert!(!config.use_tls_proxy());
        assert!(config.domain().is_some());
    }
}
