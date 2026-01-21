//! HTTP tracker configuration
//!
//! This module implements the **DDD validated constructor pattern** for HTTP tracker
//! configuration. The pattern ensures that HTTP tracker configs are always valid
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
//! use torrust_tracker_deployer_lib::domain::tracker::HttpTrackerConfig;
//! use torrust_tracker_deployer_lib::shared::DomainName;
//!
//! // Valid configuration without TLS - succeeds
//! let config = HttpTrackerConfig::new(
//!     "0.0.0.0:7070".parse().unwrap(),
//!     None,
//!     false,
//! ).expect("valid config");
//!
//! // Invalid: port 0 - fails at construction
//! let result = HttpTrackerConfig::new(
//!     "0.0.0.0:0".parse().unwrap(),
//!     None,
//!     false,
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

use super::is_localhost;
use crate::shared::DomainName;

/// Errors that can occur when creating an `HttpTrackerConfig`
///
/// These errors represent domain invariant violations. Each variant provides
/// context about what went wrong and enables the application layer to convert
/// to user-friendly error messages.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum HttpTrackerConfigError {
    /// Dynamic port assignment (port 0) is not supported
    ///
    /// Port 0 tells the OS to assign a random available port, which is not
    /// suitable for deployment configuration where ports must be known.
    #[error("dynamic port (0) is not supported for HTTP tracker bind address '{0}'")]
    DynamicPortNotSupported(SocketAddr),

    /// TLS proxy is enabled but no domain is configured
    ///
    /// When `use_tls_proxy` is true, a domain is required because Caddy needs
    /// the domain name to obtain Let's Encrypt certificates.
    #[error("TLS proxy requires a domain to be configured for HTTP tracker bind address '{0}'")]
    TlsProxyRequiresDomain(SocketAddr),

    /// Localhost address cannot be used with TLS proxy
    ///
    /// Caddy runs in a separate container and cannot reach localhost addresses
    /// in the tracker container. Use 0.0.0.0 or a specific IP instead.
    #[error("localhost '{0}' cannot be used with TLS proxy for HTTP tracker (Caddy runs in separate container)")]
    LocalhostWithTls(SocketAddr),
}

impl HttpTrackerConfigError {
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
                 Fix: Specify an explicit port number (e.g., 7070, 8080, 80).\n\
                 \n\
                 Example: \"bind_address\": \"0.0.0.0:7070\""
            }
            Self::TlsProxyRequiresDomain(_) => {
                "TLS proxy requires a domain name.\n\
                 \n\
                 Why: When use_tls_proxy is enabled, Caddy obtains TLS certificates from\n\
                 Let's Encrypt using the ACME protocol. This requires a valid domain name.\n\
                 \n\
                 Fix (choose one):\n\
                 1. Add a domain: \"domain\": \"tracker.example.com\"\n\
                 2. Disable TLS: \"use_tls_proxy\": false\n\
                 \n\
                 Note: The domain must point to your server's IP address for certificate\n\
                 acquisition to succeed."
            }
            Self::LocalhostWithTls(_) => {
                "Localhost addresses cannot be used with TLS proxy.\n\
                 \n\
                 Why: Caddy runs in a separate Docker container and cannot reach localhost\n\
                 addresses (127.0.0.1 or ::1) in the tracker container. Each container has\n\
                 its own network namespace.\n\
                 \n\
                 Fix (choose one):\n\
                 1. Use a routable address: \"bind_address\": \"0.0.0.0:7070\"\n\
                 2. Disable TLS: \"use_tls_proxy\": false\n\
                 \n\
                 Note: If you need localhost-only access without TLS, you can use SSH\n\
                 tunneling: ssh -L 7070:localhost:7070 user@server"
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
struct HttpTrackerConfigRaw {
    #[serde(deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr")]
    bind_address: SocketAddr,
    #[serde(default)]
    domain: Option<DomainName>,
    use_tls_proxy: bool,
}

/// HTTP tracker bind configuration with domain invariants enforced at construction
///
/// This type guarantees that any instance is valid according to domain rules:
/// - Bind address has a non-zero port
/// - If TLS proxy is enabled, a domain is configured
/// - If TLS proxy is enabled, bind address is not localhost
///
/// # Construction
///
/// Use `HttpTrackerConfig::new()` to create instances with validation:
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::HttpTrackerConfig;
/// use torrust_tracker_deployer_lib::shared::DomainName;
///
/// // Without TLS
/// let config = HttpTrackerConfig::new(
///     "0.0.0.0:7070".parse().unwrap(),
///     None,
///     false,
/// )?;
///
/// // With TLS (requires domain)
/// let tls_config = HttpTrackerConfig::new(
///     "0.0.0.0:7070".parse().unwrap(),
///     Some(DomainName::new("tracker.example.com")?),
///     true,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Invariants
///
/// The following invariants are enforced at construction time:
///
/// 1. **No dynamic ports**: `bind_address.port() != 0`
/// 2. **TLS requires domain**: `use_tls_proxy == true` implies `domain.is_some()`
/// 3. **No localhost with TLS**: `use_tls_proxy == true` implies `!is_localhost(bind_address)`
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    #[serde(serialize_with = "crate::domain::tracker::config::serialize_socket_addr")]
    bind_address: SocketAddr,

    /// Domain name for HTTPS certificate acquisition (optional)
    ///
    /// When present along with `use_tls_proxy: true`, this HTTP tracker will be
    /// accessible via HTTPS through the Caddy reverse proxy using this domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<DomainName>,

    /// Whether to proxy this service through Caddy with TLS termination
    ///
    /// When `true`:
    /// - The service is proxied through Caddy with HTTPS enabled
    /// - `domain` field is required
    /// - Cannot be used with localhost bind addresses (`127.0.0.1`, `::1`)
    /// - Implies the tracker's `on_reverse_proxy` should be `true`
    use_tls_proxy: bool,
}

impl HttpTrackerConfig {
    /// Creates a new HTTP tracker configuration with validation
    ///
    /// This is the primary way to construct an `HttpTrackerConfig`. All domain
    /// invariants are validated before the instance is created.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - Socket address to bind to (e.g., "0.0.0.0:7070")
    /// * `domain` - Optional domain for TLS certificate (required if `use_tls_proxy` is true)
    /// * `use_tls_proxy` - Whether to enable TLS via Caddy reverse proxy
    ///
    /// # Errors
    ///
    /// Returns `HttpTrackerConfigError` if any invariant is violated:
    ///
    /// - `DynamicPortNotSupported` - if port is 0
    /// - `TlsProxyRequiresDomain` - if `use_tls_proxy` is true but `domain` is None
    /// - `LocalhostWithTls` - if `use_tls_proxy` is true and `bind_address` is localhost
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::HttpTrackerConfig;
    /// use torrust_tracker_deployer_lib::shared::DomainName;
    ///
    /// // Basic configuration without TLS
    /// let config = HttpTrackerConfig::new(
    ///     "0.0.0.0:7070".parse().unwrap(),
    ///     None,
    ///     false,
    /// )?;
    ///
    /// // Configuration with TLS (requires domain)
    /// let tls_config = HttpTrackerConfig::new(
    ///     "0.0.0.0:7070".parse().unwrap(),
    ///     Some(DomainName::new("tracker.example.com")?),
    ///     true,
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        bind_address: SocketAddr,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> Result<Self, HttpTrackerConfigError> {
        // Invariant 1: Port 0 (dynamic assignment) is not supported
        if bind_address.port() == 0 {
            return Err(HttpTrackerConfigError::DynamicPortNotSupported(
                bind_address,
            ));
        }

        // Invariant 2: TLS proxy requires a domain
        if use_tls_proxy && domain.is_none() {
            return Err(HttpTrackerConfigError::TlsProxyRequiresDomain(bind_address));
        }

        // Invariant 3: Localhost cannot use TLS (Caddy in separate container)
        if use_tls_proxy && is_localhost(&bind_address) {
            return Err(HttpTrackerConfigError::LocalhostWithTls(bind_address));
        }

        Ok(Self {
            bind_address,
            domain,
            use_tls_proxy,
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

    /// Returns whether TLS proxy is enabled
    #[must_use]
    pub fn use_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }

    // -------------------------------------------------------------------------
    // Convenience methods
    // -------------------------------------------------------------------------

    /// Returns true if this tracker uses the TLS proxy
    ///
    /// Alias for `use_tls_proxy()` for semantic clarity.
    #[must_use]
    pub fn uses_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }

    /// Returns the domain name if TLS proxy is enabled
    ///
    /// Returns `None` if TLS is disabled, even if a domain is configured.
    /// This is useful for determining the effective TLS domain.
    #[must_use]
    pub fn tls_domain(&self) -> Option<&DomainName> {
        if self.use_tls_proxy {
            self.domain.as_ref()
        } else {
            None
        }
    }
}

/// Enables deserialization with validation through the constructor
///
/// This ensures that JSON deserialization also validates the config,
/// maintaining the "always valid" invariant even for loaded data.
impl<'de> Deserialize<'de> for HttpTrackerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = HttpTrackerConfigRaw::deserialize(deserializer)?;
        Self::new(raw.bind_address, raw.domain, raw.use_tls_proxy).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for HttpTrackerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP tracker at {}", self.bind_address)?;
        if let Some(domain) = &self.domain {
            write!(f, " ({})", domain.as_str())?;
        }
        if self.use_tls_proxy {
            write!(f, " [TLS]")?;
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
    fn it_should_create_http_tracker_config_without_tls() {
        let config = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false)
            .expect("valid config should succeed");

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.uses_tls_proxy());
        assert!(config.tls_domain().is_none());
    }

    #[test]
    fn it_should_create_http_tracker_config_with_tls() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), Some(domain), true)
            .expect("valid config should succeed");

        assert!(config.uses_tls_proxy());
        assert_eq!(
            config.tls_domain().map(DomainName::as_str),
            Some("tracker.example.com")
        );
    }

    #[test]
    fn it_should_create_http_tracker_config_with_domain_but_no_tls() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), Some(domain), false)
            .expect("valid config should succeed");

        assert!(!config.uses_tls_proxy());
        // tls_domain returns None when TLS is disabled
        assert!(config.tls_domain().is_none());
        // but domain() still returns the domain
        assert!(config.domain().is_some());
    }

    // =========================================================================
    // Invariant violation tests
    // =========================================================================

    #[test]
    fn it_should_reject_port_zero() {
        let result = HttpTrackerConfig::new("0.0.0.0:0".parse().unwrap(), None, false);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            HttpTrackerConfigError::DynamicPortNotSupported(_)
        ));
        assert!(err.to_string().contains("dynamic port"));
    }

    #[test]
    fn it_should_reject_tls_without_domain() {
        let result = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, true);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            HttpTrackerConfigError::TlsProxyRequiresDomain(_)
        ));
        assert!(err.to_string().contains("TLS proxy requires a domain"));
    }

    #[test]
    fn it_should_reject_localhost_with_tls_ipv4() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let result = HttpTrackerConfig::new("127.0.0.1:7070".parse().unwrap(), Some(domain), true);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HttpTrackerConfigError::LocalhostWithTls(_)));
        assert!(err.to_string().contains("localhost"));
    }

    #[test]
    fn it_should_reject_localhost_with_tls_ipv6() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let result = HttpTrackerConfig::new("[::1]:7070".parse().unwrap(), Some(domain), true);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HttpTrackerConfigError::LocalhostWithTls(_)));
    }

    #[test]
    fn it_should_allow_localhost_without_tls() {
        let result = HttpTrackerConfig::new("127.0.0.1:7070".parse().unwrap(), None, false);

        assert!(result.is_ok());
    }

    // =========================================================================
    // Help text tests
    // =========================================================================

    #[test]
    fn it_should_provide_help_text_for_port_zero_error() {
        let err = HttpTrackerConfigError::DynamicPortNotSupported("0.0.0.0:0".parse().unwrap());

        let help = err.help();
        assert!(help.contains("Dynamic port assignment"));
        assert!(help.contains("Fix:"));
        assert!(help.contains("7070"));
    }

    #[test]
    fn it_should_provide_help_text_for_tls_without_domain() {
        let err = HttpTrackerConfigError::TlsProxyRequiresDomain("0.0.0.0:7070".parse().unwrap());

        let help = err.help();
        assert!(help.contains("TLS proxy requires a domain"));
        assert!(help.contains("Fix"));
    }

    #[test]
    fn it_should_provide_help_text_for_localhost_with_tls() {
        let err = HttpTrackerConfigError::LocalhostWithTls("127.0.0.1:7070".parse().unwrap());

        let help = err.help();
        assert!(help.contains("Localhost addresses cannot be used"));
        assert!(help.contains("Docker container"));
    }

    // =========================================================================
    // Serialization tests
    // =========================================================================

    #[test]
    fn it_should_serialize_http_tracker_config() {
        let config = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:7070");
        assert_eq!(json["use_tls_proxy"], false);
    }

    #[test]
    fn it_should_serialize_http_tracker_config_with_tls() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config =
            HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), Some(domain), true).unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:7070");
        assert_eq!(json["domain"], "tracker.example.com");
        assert_eq!(json["use_tls_proxy"], true);
    }

    // =========================================================================
    // Deserialization tests
    // =========================================================================

    #[test]
    fn it_should_deserialize_http_tracker_config() {
        let json = r#"{"bind_address": "0.0.0.0:7070", "use_tls_proxy": false}"#;
        let config: HttpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(!config.use_tls_proxy());
    }

    #[test]
    fn it_should_deserialize_http_tracker_config_with_tls() {
        let json = r#"{"bind_address": "0.0.0.0:7070", "domain": "tracker.example.com", "use_tls_proxy": true}"#;
        let config: HttpTrackerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(
            config.bind_address(),
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );
        assert!(config.use_tls_proxy());
        assert_eq!(
            config.domain().map(DomainName::as_str),
            Some("tracker.example.com")
        );
    }

    #[test]
    fn it_should_reject_port_zero_during_deserialization() {
        let json = r#"{"bind_address": "0.0.0.0:0", "use_tls_proxy": false}"#;
        let result: Result<HttpTrackerConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("dynamic port"));
    }

    #[test]
    fn it_should_reject_tls_without_domain_during_deserialization() {
        let json = r#"{"bind_address": "0.0.0.0:7070", "use_tls_proxy": true}"#;
        let result: Result<HttpTrackerConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("TLS proxy requires a domain"));
    }

    // =========================================================================
    // Display tests
    // =========================================================================

    #[test]
    fn it_should_display_without_tls() {
        let config = HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap();

        assert_eq!(config.to_string(), "HTTP tracker at 0.0.0.0:7070");
    }

    #[test]
    fn it_should_display_with_tls() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let config =
            HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), Some(domain), true).unwrap();

        assert_eq!(
            config.to_string(),
            "HTTP tracker at 0.0.0.0:7070 (tracker.example.com) [TLS]"
        );
    }

    // =========================================================================
    // Round-trip tests
    // =========================================================================

    #[test]
    fn it_should_round_trip_through_json() {
        let domain = DomainName::new("tracker.example.com").unwrap();
        let original =
            HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), Some(domain), true).unwrap();

        let json = serde_json::to_string(&original).unwrap();
        let restored: HttpTrackerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }
}
