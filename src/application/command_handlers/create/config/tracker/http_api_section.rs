//! HTTP API configuration DTO (Application Layer)
//!
//! This module contains the DTO for HTTP API configuration. It handles JSON
//! deserialization and delegates validation to the domain layer.
//!
//! ## Pattern
//!
//! This DTO demonstrates the **Anti-Corruption Layer** pattern using `TryFrom`:
//! 1. Accept primitive types from external sources (JSON)
//! 2. Parse/transform primitives (String → `SocketAddr`, String → `DomainName`)
//! 3. Delegate business validation to domain constructor (`HttpApiConfig::new()`)
//! 4. Map domain errors to application errors
//!
//! ## `TryFrom` Pattern
//!
//! We use the standard `TryFrom` trait for DTO→Domain conversion. This provides:
//! - **Discoverability**: Developers can search for `impl TryFrom<HttpApiSection>`
//! - **Consistency**: Standard Rust pattern used across the codebase
//! - **Ergonomics**: Works with `?` operator via `.try_into()`
//!
//! See ADR: `docs/decisions/tryfrom-for-dto-to-domain-conversion.md`
//!
//! ## For Other DTOs
//!
//! Use this file as a reference when refactoring other DTO types to follow
//! the same pattern. See the refactoring plan:
//! `docs/refactors/plans/strengthen-domain-invariant-enforcement.md`

use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HttpApiConfig;
use crate::shared::secrets::PlainApiToken;
use crate::shared::DomainName;

/// HTTP API configuration section (Application DTO)
///
/// This is a Data Transfer Object that uses primitive types (`String`) for
/// JSON deserialization. It converts to the domain type `HttpApiConfig` via
/// the `TryFrom` trait, which delegates validation to the domain layer.
///
/// # Responsibility Split
///
/// - **This DTO**: Parse strings into typed values (`SocketAddr`, `DomainName`)
/// - **Domain type**: Enforce business invariants (port != 0, TLS requires domain, etc.)
///
/// # Usage
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::tracker::HttpApiSection;
/// use torrust_tracker_deployer_lib::domain::tracker::HttpApiConfig;
///
/// let section = HttpApiSection {
///     bind_address: "0.0.0.0:1212".to_string(),
///     admin_token: "MyToken".to_string(),
///     domain: None,
///     use_tls_proxy: None,
/// };
///
/// let config: HttpApiConfig = section.try_into()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # JSON Example
///
/// ```json
/// {
///     "bind_address": "0.0.0.0:1212",
///     "admin_token": "MyAccessToken",
///     "domain": "api.example.com",
///     "use_tls_proxy": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HttpApiSection {
    /// Bind address as string (e.g., "0.0.0.0:1212")
    ///
    /// Parsed to `SocketAddr` during conversion.
    pub bind_address: String,

    /// Admin token as plain string (at DTO boundary)
    ///
    /// Converted to `ApiToken` (secrecy-wrapped) in domain layer.
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

/// Converts `HttpApiSection` (DTO) to `HttpApiConfig` (Domain)
///
/// This implementation:
/// 1. Parses the bind address string to `SocketAddr`
/// 2. Parses the domain string to `DomainName` (if present)
/// 3. Delegates business validation to `HttpApiConfig::new()`
///
/// # Errors
///
/// Returns `CreateConfigError`:
/// - `InvalidBindAddress` - if bind address cannot be parsed as IP:PORT
/// - `InvalidDomain` - if domain string is not a valid domain name
/// - `HttpApiConfigInvalid` - if domain invariants are violated (port 0, TLS without domain, etc.)
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::tracker::HttpApiSection;
/// use torrust_tracker_deployer_lib::domain::tracker::HttpApiConfig;
///
/// let section = HttpApiSection {
///     bind_address: "0.0.0.0:1212".to_string(),
///     admin_token: "MyToken".to_string(),
///     domain: None,
///     use_tls_proxy: None,
/// };
///
/// let config: HttpApiConfig = section.try_into()?;
/// assert_eq!(config.bind_address().port(), 1212);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl TryFrom<HttpApiSection> for HttpApiConfig {
    type Error = CreateConfigError;

    fn try_from(section: HttpApiSection) -> Result<Self, Self::Error> {
        // Step 1: Parse bind address string to SocketAddr
        let bind_address = section.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: section.bind_address.clone(),
                source: e,
            }
        })?;

        // Step 2: Parse domain string to DomainName (if present)
        let domain = match &section.domain {
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

        // Step 3: Delegate business validation to domain constructor
        // The domain layer enforces: port != 0, TLS requires domain, no localhost with TLS
        let config = HttpApiConfig::new(
            bind_address,
            section.admin_token.clone().into(),
            domain,
            section.use_tls_proxy.unwrap_or(false),
        )?; // Uses From<HttpApiConfigError> for CreateConfigError

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Successful conversion tests
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_convert_valid_config_to_http_api_config() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "MyAccessToken".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result: Result<HttpApiConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.bind_address(),
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.admin_token().expose_secret(), "MyAccessToken");
        assert!(!config.use_tls_proxy());
    }

    #[test]
    fn it_should_allow_non_localhost_with_tls_proxy() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some("api.tracker.local".to_string()),
            use_tls_proxy: Some(true),
        };

        let result: Result<HttpApiConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.use_tls_proxy());
        assert!(config.domain().is_some());
    }

    #[test]
    fn it_should_accept_domain_without_tls_proxy() {
        // Domain provided but use_tls_proxy is false - domain is stored but TLS disabled
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some("api.tracker.local".to_string()),
            use_tls_proxy: Some(false),
        };

        let result: Result<HttpApiConfig, _> = section.try_into();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.use_tls_proxy());
        // tls_domain returns None when TLS is disabled
        assert!(config.tls_domain().is_none());
        // But domain() returns the configured domain
        assert!(config.domain().is_some());
    }

    // -------------------------------------------------------------------------
    // DTO-level error tests (parsing failures)
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_fail_for_invalid_bind_address() {
        let section = HttpApiSection {
            bind_address: "invalid-address".to_string(),
            admin_token: "token".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = HttpApiConfig::try_from(section);
        assert!(result.is_err());

        if let Err(CreateConfigError::InvalidBindAddress { address, .. }) = result {
            assert_eq!(address, "invalid-address");
        } else {
            panic!("Expected InvalidBindAddress error");
        }
    }

    #[test]
    fn it_should_fail_for_invalid_domain() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some(String::new()), // Empty domain is invalid
            use_tls_proxy: Some(true),
        };

        let result = HttpApiConfig::try_from(section);
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(CreateConfigError::InvalidDomain { .. })
        ));
    }

    // -------------------------------------------------------------------------
    // Domain-level error tests (delegated to domain layer)
    // -------------------------------------------------------------------------

    #[test]
    fn it_should_reject_port_zero_via_domain_validation() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:0".to_string(),
            admin_token: "token".to_string(),
            domain: None,
            use_tls_proxy: None,
        };

        let result = HttpApiConfig::try_from(section);
        assert!(result.is_err());

        // Error comes from domain layer via From impl
        assert!(matches!(
            result,
            Err(CreateConfigError::HttpApiConfigInvalid(_))
        ));
    }

    #[test]
    fn it_should_reject_tls_proxy_without_domain_via_domain_validation() {
        let section = HttpApiSection {
            bind_address: "0.0.0.0:1212".to_string(),
            admin_token: "token".to_string(),
            domain: None,              // No domain
            use_tls_proxy: Some(true), // But TLS enabled
        };

        let result = HttpApiConfig::try_from(section);
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(CreateConfigError::HttpApiConfigInvalid(_))
        ));
    }

    #[test]
    fn it_should_reject_localhost_with_tls_via_domain_validation() {
        let section = HttpApiSection {
            bind_address: "127.0.0.1:1212".to_string(),
            admin_token: "token".to_string(),
            domain: Some("api.example.com".to_string()),
            use_tls_proxy: Some(true),
        };

        let result = HttpApiConfig::try_from(section);
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(CreateConfigError::HttpApiConfigInvalid(_))
        ));
    }

    // -------------------------------------------------------------------------
    // Serialization tests
    // -------------------------------------------------------------------------

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
    fn it_should_deserialize_with_all_fields() {
        let json = r#"{"bind_address":"0.0.0.0:1212","admin_token":"token","domain":"api.example.com","use_tls_proxy":true}"#;
        let section: HttpApiSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.bind_address, "0.0.0.0:1212");
        assert_eq!(section.domain, Some("api.example.com".to_string()));
        assert_eq!(section.use_tls_proxy, Some(true));
    }
}
