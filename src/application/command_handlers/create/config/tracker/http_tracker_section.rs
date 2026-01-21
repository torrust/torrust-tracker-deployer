//! HTTP tracker section DTO
//!
//! This module contains the application layer DTO for HTTP tracker configuration.
//! It follows the **`TryFrom` pattern** for DTO to domain conversion, delegating
//! all business validation to the domain layer.
//!
//! ## Conversion Pattern
//!
//! The `TryFrom<HttpTrackerSection> for HttpTrackerConfig` implementation:
//! 1. Parses string fields into typed values (e.g., `String` → `SocketAddr`)
//! 2. Delegates domain validation to `HttpTrackerConfig::new()`
//! 3. Maps domain errors to application errors via `From` implementations
//!
//! See `docs/decisions/tryfrom-for-dto-to-domain-conversion.md` for rationale.

use std::convert::TryFrom;
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

/// Converts from application DTO to domain type using `TryFrom` trait
///
/// This implementation follows the standard library convention for fallible
/// conversions, enabling use of `.try_into()` and `TryFrom::try_from()`.
///
/// # Example
///
/// ```rust,ignore
/// let section = HttpTrackerSection {
///     bind_address: "0.0.0.0:7070".to_string(),
///     domain: None,
///     use_tls_proxy: None,
/// };
/// let config: HttpTrackerConfig = section.try_into()?;
/// ```
impl TryFrom<HttpTrackerSection> for HttpTrackerConfig {
    type Error = CreateConfigError;

    fn try_from(section: HttpTrackerSection) -> Result<Self, Self::Error> {
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
        HttpTrackerConfig::new(bind_address, domain, use_tls_proxy).map_err(CreateConfigError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // TryFrom conversion tests
    // =========================================================================

    #[test]
    fn it_should_convert_valid_bind_address_to_http_tracker_config() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address(),
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy());
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpTrackerSection {
            bind_address: "not-valid".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "not-valid");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_reject_port_zero_via_domain_validation() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:0".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        // Port 0 is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HttpTrackerConfigInvalid(_)
        ));
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

        let result: Result<HttpTrackerConfig, _> = section.try_into();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.use_tls_proxy());
        assert!(config.domain().is_some());
    }

    #[test]
    fn it_should_reject_tls_proxy_without_domain_via_domain_validation() {
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        // TLS without domain is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HttpTrackerConfigInvalid(_)
        ));
    }

    #[test]
    fn it_should_reject_localhost_with_tls_via_domain_validation() {
        let section = HttpTrackerSection {
            bind_address: "127.0.0.1:7070".to_string(),
            domain: Some("tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        // Localhost + TLS is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HttpTrackerConfigInvalid(_)
        ));
    }

    #[test]
    fn it_should_accept_domain_without_tls_proxy() {
        // Domain provided but use_tls_proxy is false - domain is stored but not used for TLS
        let section = HttpTrackerSection {
            bind_address: "0.0.0.0:7070".to_string(),
            domain: Some("tracker.local".to_string()),
            use_tls_proxy: Some(false),
        };

        let result: Result<HttpTrackerConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.use_tls_proxy());
        // Domain is still stored but won't be used for TLS
        assert!(config.domain().is_some());
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
