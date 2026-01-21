//! HTTP API configuration
//!
//! This module demonstrates the **DDD validated constructor pattern** where domain
//! types enforce their invariants at construction time, making it impossible to
//! create invalid domain objects.
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
//! use torrust_tracker_deployer_lib::domain::tracker::HttpApiConfig;
//! use torrust_tracker_deployer_lib::shared::{ApiToken, DomainName};
//!
//! // Valid configuration - succeeds
//! let config = HttpApiConfig::new(
//!     "0.0.0.0:1212".parse().unwrap(),
//!     ApiToken::from("token".to_string()),
//!     None,
//!     false,
//! ).expect("valid config");
//!
//! // Invalid: port 0 - fails at construction
//! let result = HttpApiConfig::new(
//!     "0.0.0.0:0".parse().unwrap(),
//!     ApiToken::from("token".to_string()),
//!     None,
//!     false,
//! );
//! assert!(result.is_err());
//! ```
//!
//! ## For Other Domain Types
//!
//! Use this file as a reference when refactoring other domain configuration types
//! to follow the same pattern. See the refactoring plan:
//! `docs/refactors/plans/strengthen-domain-invariant-enforcement.md`

use std::fmt;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::is_localhost;
use crate::shared::{ApiToken, DomainName};

/// Errors that can occur when creating an `HttpApiConfig`
///
/// These errors represent domain invariant violations. Each variant provides
/// context about what went wrong and enables the application layer to convert
/// to user-friendly error messages.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum HttpApiConfigError {
    /// Dynamic port assignment (port 0) is not supported
    ///
    /// Port 0 tells the OS to assign a random available port, which is not
    /// suitable for deployment configuration where ports must be known.
    #[error("dynamic port (0) is not supported for bind address '{0}'")]
    DynamicPortNotSupported(SocketAddr),

    /// TLS proxy is enabled but no domain is configured
    ///
    /// When `use_tls_proxy` is true, a domain is required because Caddy needs
    /// the domain name to obtain Let's Encrypt certificates.
    #[error("TLS proxy requires a domain to be configured for bind address '{0}'")]
    TlsProxyRequiresDomain(SocketAddr),

    /// Localhost address cannot be used with TLS proxy
    ///
    /// Caddy runs in a separate container and cannot reach localhost addresses
    /// in the tracker container. Use 0.0.0.0 or a specific IP instead.
    #[error("localhost '{0}' cannot be used with TLS proxy (Caddy runs in separate container)")]
    LocalhostWithTls(SocketAddr),
}

impl HttpApiConfigError {
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
                 Fix: Specify an explicit port number (e.g., 1212, 8080, 3000).\n\
                 \n\
                 Example: \"bind_address\": \"0.0.0.0:1212\""
            }
            Self::TlsProxyRequiresDomain(_) => {
                "TLS proxy requires a domain name.\n\
                 \n\
                 Why: When use_tls_proxy is enabled, Caddy obtains TLS certificates from\n\
                 Let's Encrypt using the ACME protocol. This requires a valid domain name.\n\
                 \n\
                 Fix (choose one):\n\
                 1. Add a domain: \"domain\": \"api.example.com\"\n\
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
                 1. Use a routable address: \"bind_address\": \"0.0.0.0:1212\"\n\
                 2. Disable TLS: \"use_tls_proxy\": false\n\
                 \n\
                 Note: If you need localhost-only access without TLS, you can use SSH\n\
                 tunneling: ssh -L 1212:localhost:1212 user@server"
            }
        }
    }
}

/// Internal struct for serde deserialization that bypasses validation
///
/// This allows us to deserialize JSON into the raw fields, then validate
/// through the `TryFrom` implementation. This pattern ensures that even
/// deserialized configs are validated.
#[derive(Deserialize)]
struct HttpApiConfigRaw {
    #[serde(deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr")]
    bind_address: SocketAddr,
    admin_token: ApiToken,
    #[serde(default)]
    domain: Option<DomainName>,
    use_tls_proxy: bool,
}

/// HTTP API configuration with domain invariants enforced at construction
///
/// This type guarantees that any instance is valid according to domain rules:
/// - Bind address has a non-zero port
/// - If TLS proxy is enabled, a domain is configured
/// - If TLS proxy is enabled, bind address is not localhost
///
/// # Construction
///
/// Use `HttpApiConfig::new()` to create instances with validation:
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::HttpApiConfig;
/// use torrust_tracker_deployer_lib::shared::ApiToken;
///
/// let config = HttpApiConfig::new(
///     "0.0.0.0:1212".parse().unwrap(),
///     ApiToken::from("MyToken".to_string()),
///     None,
///     false,
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
pub struct HttpApiConfig {
    /// Bind address (e.g., "0.0.0.0:1212")
    #[serde(serialize_with = "crate::domain::tracker::config::serialize_socket_addr")]
    bind_address: SocketAddr,

    /// Admin access token for HTTP API authentication
    admin_token: ApiToken,

    /// Domain name for HTTPS certificate acquisition (optional)
    ///
    /// When present along with `use_tls_proxy: true`, this HTTP API will be
    /// accessible via HTTPS through the Caddy reverse proxy using this domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<DomainName>,

    /// Whether to proxy this service through Caddy with TLS termination
    use_tls_proxy: bool,
}

impl HttpApiConfig {
    /// Creates a new HTTP API configuration with validation
    ///
    /// This is the primary way to construct an `HttpApiConfig`. All domain
    /// invariants are validated before the instance is created.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - Socket address to bind to (e.g., "0.0.0.0:1212")
    /// * `admin_token` - API token for authentication
    /// * `domain` - Optional domain for TLS certificate (required if `use_tls_proxy` is true)
    /// * `use_tls_proxy` - Whether to enable TLS via Caddy reverse proxy
    ///
    /// # Errors
    ///
    /// Returns `HttpApiConfigError` if any invariant is violated:
    ///
    /// - `DynamicPortNotSupported` - if port is 0
    /// - `TlsProxyRequiresDomain` - if `use_tls_proxy` is true but `domain` is None
    /// - `LocalhostWithTls` - if `use_tls_proxy` is true and `bind_address` is localhost
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::HttpApiConfig;
    /// use torrust_tracker_deployer_lib::shared::{ApiToken, DomainName};
    ///
    /// // Basic configuration without TLS
    /// let config = HttpApiConfig::new(
    ///     "0.0.0.0:1212".parse().unwrap(),
    ///     ApiToken::from("MyToken".to_string()),
    ///     None,
    ///     false,
    /// )?;
    ///
    /// // Configuration with TLS (requires domain)
    /// let tls_config = HttpApiConfig::new(
    ///     "0.0.0.0:1212".parse().unwrap(),
    ///     ApiToken::from("MyToken".to_string()),
    ///     Some(DomainName::new("api.example.com")?),
    ///     true,
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        bind_address: SocketAddr,
        admin_token: ApiToken,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> Result<Self, HttpApiConfigError> {
        // Invariant 1: Port 0 (dynamic assignment) is not supported
        if bind_address.port() == 0 {
            return Err(HttpApiConfigError::DynamicPortNotSupported(bind_address));
        }

        // Invariant 2: TLS proxy requires a domain
        if use_tls_proxy && domain.is_none() {
            return Err(HttpApiConfigError::TlsProxyRequiresDomain(bind_address));
        }

        // Invariant 3: Localhost cannot use TLS (Caddy in separate container)
        if use_tls_proxy && is_localhost(&bind_address) {
            return Err(HttpApiConfigError::LocalhostWithTls(bind_address));
        }

        Ok(Self {
            bind_address,
            admin_token,
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

    /// Returns a reference to the admin token
    #[must_use]
    pub fn admin_token(&self) -> &ApiToken {
        &self.admin_token
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

    /// Returns true if this API uses the TLS proxy
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

/// Enables deserialization with validation through `TryFrom`
///
/// This ensures that JSON deserialization also validates the config,
/// maintaining the "always valid" invariant even for loaded data.
impl<'de> Deserialize<'de> for HttpApiConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = HttpApiConfigRaw::deserialize(deserializer)?;
        Self::new(
            raw.bind_address,
            raw.admin_token,
            raw.domain,
            raw.use_tls_proxy,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for HttpApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP API at {}", self.bind_address)?;
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

    // -------------------------------------------------------------------------
    // Construction tests - verify invariant enforcement
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_create_config_when_all_invariants_satisfied() {
        let result = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("test_token".to_string()),
            None,
            false,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.bind_address(), "0.0.0.0:1212".parse().unwrap());
        assert_eq!(config.admin_token().expose_secret(), "test_token");
        assert!(!config.uses_tls_proxy());
        assert!(config.tls_domain().is_none());
    }

    #[test]
    fn it_should_create_config_with_tls_when_domain_provided() {
        let result = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("test_token".to_string()),
            Some(DomainName::new("api.example.com").unwrap()),
            true,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.uses_tls_proxy());
        assert_eq!(
            config.tls_domain().map(DomainName::as_str),
            Some("api.example.com")
        );
    }

    #[test]
    fn it_should_reject_port_zero() {
        let result = HttpApiConfig::new(
            "0.0.0.0:0".parse().unwrap(),
            ApiToken::from("token".to_string()),
            None,
            false,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HttpApiConfigError::DynamicPortNotSupported(_)
        ));
    }

    #[test]
    fn it_should_reject_tls_without_domain() {
        let result = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            None, // No domain
            true, // But TLS enabled
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HttpApiConfigError::TlsProxyRequiresDomain(_)
        ));
    }

    #[test]
    fn it_should_reject_localhost_with_tls() {
        let result = HttpApiConfig::new(
            "127.0.0.1:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            Some(DomainName::new("api.example.com").unwrap()),
            true, // TLS enabled with localhost
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HttpApiConfigError::LocalhostWithTls(_)
        ));
    }

    #[test]
    fn it_should_reject_ipv6_localhost_with_tls() {
        let result = HttpApiConfig::new(
            "[::1]:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            Some(DomainName::new("api.example.com").unwrap()),
            true,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HttpApiConfigError::LocalhostWithTls(_)
        ));
    }

    #[test]
    fn it_should_allow_localhost_without_tls() {
        // Localhost is fine when TLS is disabled
        let result = HttpApiConfig::new(
            "127.0.0.1:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            None,
            false,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_allow_domain_without_tls() {
        // Domain can be set even without TLS (ignored but valid)
        let result = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            Some(DomainName::new("api.example.com").unwrap()),
            false, // TLS disabled
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(!config.uses_tls_proxy());
        // tls_domain returns None when TLS is disabled
        assert!(config.tls_domain().is_none());
        // But domain() still returns the configured domain
        assert!(config.domain().is_some());
    }

    // -------------------------------------------------------------------------
    // Serialization tests
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_serialize_config_to_json() {
        let config = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("token123".to_string()),
            None,
            false,
        )
        .unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["bind_address"], "0.0.0.0:1212");
        assert_eq!(json["admin_token"], "token123");
        assert_eq!(json["use_tls_proxy"], false);
    }

    #[test]
    fn it_should_deserialize_valid_json() {
        let json =
            r#"{"bind_address": "0.0.0.0:1212", "admin_token": "MyToken", "use_tls_proxy": false}"#;
        let result: Result<HttpApiConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.bind_address(), "0.0.0.0:1212".parse().unwrap());
        assert_eq!(config.admin_token().expose_secret(), "MyToken");
    }

    #[test]
    fn it_should_reject_invalid_json_with_port_zero() {
        let json =
            r#"{"bind_address": "0.0.0.0:0", "admin_token": "MyToken", "use_tls_proxy": false}"#;
        let result: Result<HttpApiConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("dynamic port"));
    }

    #[test]
    fn it_should_reject_invalid_json_with_tls_but_no_domain() {
        let json =
            r#"{"bind_address": "0.0.0.0:1212", "admin_token": "MyToken", "use_tls_proxy": true}"#;
        let result: Result<HttpApiConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("TLS proxy requires a domain"));
    }

    // -------------------------------------------------------------------------
    // Error help message tests
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_provide_help_for_dynamic_port_error() {
        let error = HttpApiConfigError::DynamicPortNotSupported("0.0.0.0:0".parse().unwrap());
        let help = error.help();
        assert!(help.contains("Dynamic port assignment"));
        assert!(help.contains("Fix:"));
    }

    #[test]
    fn it_should_provide_help_for_tls_without_domain_error() {
        let error = HttpApiConfigError::TlsProxyRequiresDomain("0.0.0.0:1212".parse().unwrap());
        let help = error.help();
        assert!(help.contains("TLS proxy requires a domain"));
        assert!(help.contains("Fix"));
    }

    #[test]
    fn it_should_provide_help_for_localhost_with_tls_error() {
        let error = HttpApiConfigError::LocalhostWithTls("127.0.0.1:1212".parse().unwrap());
        let help = error.help();
        assert!(help.contains("Localhost addresses cannot be used"));
        assert!(help.contains("Docker container"));
    }

    // -------------------------------------------------------------------------
    // Display tests
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_display_basic_config() {
        let config = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            None,
            false,
        )
        .unwrap();

        assert_eq!(format!("{config}"), "HTTP API at 0.0.0.0:1212");
    }

    #[test]
    fn it_should_display_config_with_tls() {
        let config = HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("token".to_string()),
            Some(DomainName::new("api.example.com").unwrap()),
            true,
        )
        .unwrap();

        assert_eq!(
            format!("{config}"),
            "HTTP API at 0.0.0.0:1212 (api.example.com) [TLS]"
        );
    }
}
