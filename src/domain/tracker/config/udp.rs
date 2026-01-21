//! UDP tracker configuration
//!
//! This module implements the **DDD validated constructor pattern** for UDP tracker
//! configuration. The pattern ensures that UDP tracker configs are always valid
//! after construction.
//!
//! ## Pattern Overview
//!
//! 1. **Private fields**: All fields are private to prevent bypassing validation
//! 2. **Validated constructor**: `new()` validates all invariants before creation
//! 3. **Getter methods**: Provide read-only access to field values
//! 4. **Domain error type**: Rich error enum for validation failures
//! 5. **Serde with validation**: Deserialization goes through the constructor
//!
//! ## Example
//!
//! ```rust
//! use torrust_tracker_deployer_lib::domain::tracker::UdpTrackerConfig;
//! use torrust_tracker_deployer_lib::shared::DomainName;
//!
//! // Valid configuration - succeeds
//! let config = UdpTrackerConfig::new(
//!     "0.0.0.0:6969".parse().unwrap(),
//!     None,
//! ).expect("valid config");
//!
//! // Invalid: port 0 - fails at construction
//! let result = UdpTrackerConfig::new(
//!     "0.0.0.0:0".parse().unwrap(),
//!     None,
//! );
//! assert!(result.is_err());
//! ```
//!
//! ## Reference Implementation
//!
//! See `http_api.rs` for the original reference implementation of this pattern.

use std::fmt;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::shared::DomainName;

/// Errors that can occur when creating a `UdpTrackerConfig`
///
/// These errors represent domain invariant violations. Each variant provides
/// context about what went wrong and enables the application layer to convert
/// to user-friendly error messages.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum UdpTrackerConfigError {
    /// Dynamic port assignment (port 0) is not supported
    ///
    /// Port 0 tells the OS to assign a random available port, which is not
    /// suitable for deployment configuration where ports must be known.
    #[error("dynamic port (0) is not supported for UDP tracker bind address '{0}'")]
    DynamicPortNotSupported(SocketAddr),
}

impl UdpTrackerConfigError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// This method follows the project's tiered help system pattern,
    /// providing actionable guidance for resolving configuration issues.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DynamicPortNotSupported(_) => {
                "Dynamic port assignment (port 0) is not supported.\n\
                 \n\
                 Why: Port 0 tells the operating system to assign a random available port.\n\
                 This is not suitable for deployment where ports must be known in advance\n\
                 for firewall rules, load balancers, and client configuration.\n\
                 \n\
                 Fix: Specify an explicit port number (e.g., 6969, 6868, 6881).\n\
                 \n\
                 Example: \"bind_address\": \"0.0.0.0:6969\""
            }
        }
    }
}

/// Internal struct for serde deserialization that bypasses validation
///
/// This allows us to deserialize JSON into the raw fields, then validate
/// through the constructor. This pattern ensures that even
/// deserialized configs are validated.
#[derive(Deserialize)]
struct UdpTrackerConfigRaw {
    #[serde(deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr")]
    bind_address: SocketAddr,
    #[serde(default)]
    domain: Option<DomainName>,
}

/// UDP tracker bind configuration with domain invariants enforced at construction
///
/// This type guarantees that any instance is valid according to domain rules:
/// - Bind address has a non-zero port
///
/// Note: Unlike HTTP trackers, UDP does not support TLS, so there are no
/// TLS-related validation rules.
///
/// # Construction
///
/// Use `UdpTrackerConfig::new()` to create instances with validation:
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::UdpTrackerConfig;
///
/// let config = UdpTrackerConfig::new(
///     "0.0.0.0:6969".parse().unwrap(),
///     None,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Invariants
///
/// The following invariants are enforced at construction time:
///
/// 1. **No dynamic ports**: `bind_address.port() != 0`
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
    #[serde(serialize_with = "crate::domain::tracker::config::serialize_socket_addr")]
    bind_address: SocketAddr,

    /// Domain name for announce URLs (optional)
    ///
    /// When present, this domain can be used when communicating the tracker's
    /// announce URL to users, e.g., `udp://tracker.example.com:6969/announce`
    ///
    /// Note: Unlike HTTP trackers, UDP does not support TLS, so there is no
    /// `use_tls_proxy` field for UDP trackers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    domain: Option<DomainName>,
}

impl UdpTrackerConfig {
    /// Creates a new UDP tracker configuration with validation
    ///
    /// This is the primary way to construct a `UdpTrackerConfig`. All domain
    /// invariants are validated before the instance is created.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - Socket address to bind to (e.g., "0.0.0.0:6969")
    /// * `domain` - Optional domain for announce URLs
    ///
    /// # Errors
    ///
    /// Returns `UdpTrackerConfigError` if any invariant is violated:
    ///
    /// - `DynamicPortNotSupported` - if port is 0
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::UdpTrackerConfig;
    /// use torrust_tracker_deployer_lib::shared::DomainName;
    ///
    /// // Basic configuration without domain
    /// let config = UdpTrackerConfig::new(
    ///     "0.0.0.0:6969".parse().unwrap(),
    ///     None,
    /// )?;
    ///
    /// // Configuration with domain
    /// let config_with_domain = UdpTrackerConfig::new(
    ///     "0.0.0.0:6969".parse().unwrap(),
    ///     Some(DomainName::new("tracker.example.com")?),
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        bind_address: SocketAddr,
        domain: Option<DomainName>,
    ) -> Result<Self, UdpTrackerConfigError> {
        // Invariant 1: Port 0 (dynamic assignment) is not supported
        if bind_address.port() == 0 {
            return Err(UdpTrackerConfigError::DynamicPortNotSupported(bind_address));
        }

        Ok(Self {
            bind_address,
            domain,
        })
    }

    // -------------------------------------------------------------------------
    // Getter methods - provide read-only access to fields
    // -------------------------------------------------------------------------

    /// Returns the bind address
    #[must_use]
    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    /// Returns a reference to the domain, if configured
    #[must_use]
    pub fn domain(&self) -> Option<&DomainName> {
        self.domain.as_ref()
    }
}

/// Enables deserialization with validation through the constructor
///
/// This ensures that JSON deserialization also validates the config,
/// maintaining the "always valid" invariant even for loaded data.
impl<'de> Deserialize<'de> for UdpTrackerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = UdpTrackerConfigRaw::deserialize(deserializer)?;
        Self::new(raw.bind_address, raw.domain).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for UdpTrackerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UDP tracker at {}", self.bind_address)?;
        if let Some(domain) = &self.domain {
            write!(f, " ({})", domain.as_str())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Valid construction tests
    // =========================================================================

    #[test]
    fn it_should_create_udp_tracker_config_without_domain() {
        let config = UdpTrackerConfig::new("0.0.0.0:6868".parse().unwrap(), None)
            .expect("valid config should succeed");

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:6868".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain().is_none());
    }

    #[test]
    fn it_should_create_udp_tracker_config_with_domain() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config = UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), Some(domain))
            .expect("valid config should succeed");

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.domain().map(DomainName::as_str),
            Some("tracker.example.com")
        );
    }

    // =========================================================================
    // Invariant violation tests
    // =========================================================================

    #[test]
    fn it_should_reject_port_zero() {
        let result = UdpTrackerConfig::new("0.0.0.0:0".parse().unwrap(), None);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            UdpTrackerConfigError::DynamicPortNotSupported(_)
        ));
        assert!(err.to_string().contains("dynamic port"));
    }

    #[test]
    fn it_should_provide_help_text_for_port_zero_error() {
        let err = UdpTrackerConfigError::DynamicPortNotSupported("0.0.0.0:0".parse().unwrap());

        let help = err.help();
        assert!(help.contains("Dynamic port assignment"));
        assert!(help.contains("Fix:"));
        assert!(help.contains("6969"));
    }

    // =========================================================================
    // Serialization tests
    // =========================================================================

    #[test]
    fn it_should_serialize_udp_tracker_config_without_domain() {
        let config = UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:6969");
        // domain should not be present when None (skip_serializing_if)
        assert!(json.get("domain").is_none());
    }

    #[test]
    fn it_should_serialize_udp_tracker_config_with_domain() {
        let domain = DomainName::new("udp.tracker.local").unwrap();
        let config = UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), Some(domain)).unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:6969");
        assert_eq!(json["domain"], "udp.tracker.local");
    }

    // =========================================================================
    // Deserialization tests
    // =========================================================================

    #[test]
    fn it_should_deserialize_udp_tracker_config_without_domain() {
        let json = r#"{"bind_address": "0.0.0.0:6969"}"#;
        let config: UdpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );
        assert!(config.domain().is_none());
    }

    #[test]
    fn it_should_deserialize_udp_tracker_config_with_domain() {
        let json = r#"{"bind_address": "0.0.0.0:6969", "domain": "udp.tracker.local"}"#;
        let config: UdpTrackerConfig = serde_json::from_str(json).unwrap();

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
    fn it_should_reject_port_zero_during_deserialization() {
        let json = r#"{"bind_address": "0.0.0.0:0"}"#;
        let result: Result<UdpTrackerConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("dynamic port"));
    }

    // =========================================================================
    // Display tests
    // =========================================================================

    #[test]
    fn it_should_display_without_domain() {
        let config = UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap();

        assert_eq!(config.to_string(), "UDP tracker at 0.0.0.0:6969");
    }

    #[test]
    fn it_should_display_with_domain() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config = UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), Some(domain)).unwrap();

        assert_eq!(
            config.to_string(),
            "UDP tracker at 0.0.0.0:6969 (tracker.example.com)"
        );
    }

    // =========================================================================
    // Round-trip tests
    // =========================================================================

    #[test]
    fn it_should_round_trip_through_json() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let original =
            UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), Some(domain)).unwrap();

        let json = serde_json::to_string(&original).unwrap();
        let restored: UdpTrackerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }
}
