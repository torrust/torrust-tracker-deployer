//! UDP tracker section DTO
//!
//! This module contains the application layer DTO for UDP tracker configuration.
//! It follows the **`TryFrom` pattern** for DTO to domain conversion, delegating
//! all business validation to the domain layer.
//!
//! ## Conversion Pattern
//!
//! The `TryFrom<UdpTrackerSection> for UdpTrackerConfig` implementation:
//! 1. Parses string fields into typed values (e.g., `String` → `SocketAddr`)
//! 2. Delegates domain validation to `UdpTrackerConfig::new()`
//! 3. Maps domain errors to application errors via `From` implementations
//!
//! See `docs/decisions/tryfrom-for-dto-to-domain-conversion.md` for rationale.

use std::convert::TryFrom;
use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::UdpTrackerConfig;
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct UdpTrackerSection {
    pub bind_address: String,

    /// Domain name for the UDP tracker (optional)
    ///
    /// When present, this domain can be used in announce URLs instead of the IP.
    /// Example: `udp://tracker.example.com:6969/announce`
    ///
    /// Note: Unlike HTTP trackers, UDP does not support TLS, so there is no
    /// `use_tls_proxy` field for UDP trackers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// Converts from application DTO to domain type using `TryFrom` trait
///
/// This implementation follows the standard library convention for fallible
/// conversions, enabling use of `.try_into()` and `TryFrom::try_from()`.
///
/// # Example
///
/// ```rust,ignore
/// let section = UdpTrackerSection { bind_address: "0.0.0.0:6969".to_string(), domain: None };
/// let config: UdpTrackerConfig = section.try_into()?;
/// ```
impl TryFrom<UdpTrackerSection> for UdpTrackerConfig {
    type Error = CreateConfigError;

    fn try_from(section: UdpTrackerSection) -> Result<Self, Self::Error> {
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

        // Delegate all business validation to domain layer
        UdpTrackerConfig::new(bind_address, domain).map_err(CreateConfigError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // TryFrom conversion tests
    // =========================================================================

    #[test]
    fn it_should_convert_valid_bind_address_to_udp_tracker_config() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: None,
        };

        let result: Result<UdpTrackerConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address(),
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain().is_none());
    }

    #[test]
    fn it_should_convert_with_valid_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some("udp.tracker.local".to_string()),
        };

        let result: Result<UdpTrackerConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address(),
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.domain().map(DomainName::as_str),
            Some("udp.tracker.local")
        );
    }

    #[test]
    fn it_should_fail_for_invalid_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some(String::new()), // Empty domain is invalid
        };

        let result: Result<UdpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidDomain { domain, .. }) = result {
            assert_eq!(domain, "");
        } else {
            panic!("Expected InvalidDomain error");
        }
    }

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = UdpTrackerSection {
            bind_address: "invalid".to_string(),
            domain: None,
        };

        let result: Result<UdpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "invalid");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_reject_port_zero_via_domain_validation() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:0".to_string(),
            domain: None,
        };

        let result: Result<UdpTrackerConfig, _> = section.try_into();
        assert!(result.is_err());

        // Port 0 is now rejected by domain layer
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::UdpTrackerConfigInvalid(_)
        ));
    }

    // =========================================================================
    // Serialization tests
    // =========================================================================

    #[test]
    fn it_should_be_serializable_without_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: None,
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:6969"));
        // domain should not be present when None (skip_serializing_if)
        assert!(!json.contains("domain"));
    }

    #[test]
    fn it_should_be_serializable_with_domain() {
        let section = UdpTrackerSection {
            bind_address: "0.0.0.0:6969".to_string(),
            domain: Some("udp.tracker.local".to_string()),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("bind_address"));
        assert!(json.contains("0.0.0.0:6969"));
        assert!(json.contains("domain"));
        assert!(json.contains("udp.tracker.local"));
    }

    #[test]
    fn it_should_be_deserializable_without_domain() {
        let json = r#"{"bind_address":"0.0.0.0:6969"}"#;
        let section: UdpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:6969");
        assert!(section.domain.is_none());
    }

    #[test]
    fn it_should_be_deserializable_with_domain() {
        let json = r#"{"bind_address":"0.0.0.0:6969","domain":"udp.tracker.local"}"#;
        let section: UdpTrackerSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:6969");
        assert_eq!(section.domain, Some("udp.tracker.local".to_string()));
    }
}
