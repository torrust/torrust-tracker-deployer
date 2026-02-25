//! Domain name type with basic validation
//!
//! This module provides a strongly-typed domain name representation that performs
//! basic validation to catch common typos and mistakes. It follows the same pattern
//! as the `Email` type, providing validation during construction while keeping DTO
//! types as primitives for serialization.
//!
//! # Design Decision
//!
//! We intentionally use **minimal validation** rather than strict RFC compliance.
//! The goal is to catch common user mistakes (typos) without rejecting valid but
//! unusual domain names. Caddy and Let's Encrypt are permissive about domain formats,
//! and strict DNS validation would add complexity without significant benefit.
//!
//! Alternative approaches considered:
//! - `addr` crate: RFC-compliant but might reject valid edge cases
//! - `publicsuffix` crate: Validates against public suffix list (overkill)
//! - `idna` crate: Handles internationalized domains (not needed)
//! - Full RFC 1035 DNS validation: Too strict for our use case
//!
//! # Validation Rules
//!
//! Minimal checks to catch typos:
//! - Not empty
//! - No whitespace
//! - Has at least one dot (TLD separator)
//! - No consecutive dots (e.g., `example..com`)
//! - Doesn't start or end with a dot
//!
//! # Architecture (DDD)
//!
//! This type lives in the shared layer rather than domain layer because:
//! - It's a generic building block used across multiple domains
//! - Similar to `Username`, `Email`, and other value objects
//! - The domain layer would use this type for business logic validation
//!
//! # Examples
//!
//! ```
//! use torrust_deployer_types::DomainName;
//!
//! // Valid domain names
//! let domain = DomainName::new("example.com").unwrap();
//! let api = DomainName::new("api.tracker.torrust.org").unwrap();
//! let with_hyphen = DomainName::new("my-service.example.com").unwrap();
//!
//! // Invalid domain names (typos)
//! assert!(DomainName::new("localhost").is_err());  // No TLD
//! assert!(DomainName::new("example..com").is_err()); // Consecutive dots
//! assert!(DomainName::new(" example.com").is_err()); // Leading space
//! ```

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A validated domain name
///
/// This type guarantees that the contained string passes basic domain name
/// validation (see module docs for validation rules). It's immutable and
/// provides access to the underlying string via `as_str()`.
///
/// # Construction
///
/// Use `DomainName::new()` to create a validated instance. The constructor
/// validates the input and returns a `Result` with a detailed error if
/// validation fails.
///
/// # Serialization
///
/// Implements Serde traits with validation on deserialization. Invalid
/// domain names will cause deserialization to fail with a descriptive error.
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct DomainName(String);

impl DomainName {
    /// Creates a new validated domain name
    ///
    /// # Arguments
    ///
    /// * `domain` - A string slice containing the domain name to validate
    ///
    /// # Errors
    ///
    /// Returns `DomainNameError` if the domain name is invalid:
    /// - `EmptyDomain` - Domain is empty
    /// - `InvalidFormat` - Domain has typos or formatting issues
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_deployer_types::DomainName;
    ///
    /// let domain = DomainName::new("tracker.torrust.org").unwrap();
    /// assert_eq!(domain.as_str(), "tracker.torrust.org");
    /// ```
    pub fn new(domain: &str) -> Result<Self, DomainNameError> {
        Self::validate(domain)?;
        Ok(Self(domain.to_string()))
    }

    /// Returns the domain name as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extracts the top-level domain (TLD)
    ///
    /// # Panics
    ///
    /// This method will not panic because `DomainName` is validated at
    /// construction to always contain at least one dot (TLD separator).
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_deployer_types::DomainName;
    ///
    /// let domain = DomainName::new("api.tracker.torrust.org").unwrap();
    /// assert_eq!(domain.tld(), "org");
    /// ```
    #[must_use]
    pub fn tld(&self) -> &str {
        self.0
            .rsplit('.')
            .next()
            .expect("validated domain always has TLD")
    }

    /// Returns all subdomains as a vector (excluding TLD)
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_deployer_types::DomainName;
    ///
    /// let domain = DomainName::new("api.tracker.torrust.org").unwrap();
    /// assert_eq!(domain.subdomains(), vec!["api", "tracker", "torrust"]);
    /// ```
    #[must_use]
    pub fn subdomains(&self) -> Vec<&str> {
        let parts: Vec<&str> = self.0.split('.').collect();
        parts[..parts.len() - 1].to_vec()
    }

    /// Validates a domain name string
    ///
    /// Uses minimal validation to catch common typos without being overly strict.
    /// See module documentation for the rationale behind this approach.
    fn validate(domain: &str) -> Result<(), DomainNameError> {
        // Check for empty domain
        if domain.is_empty() {
            return Err(DomainNameError::EmptyDomain);
        }

        // Check for whitespace (common typo)
        if domain.chars().any(char::is_whitespace) {
            return Err(DomainNameError::InvalidFormat {
                domain: domain.to_string(),
                reason: "domain cannot contain whitespace".to_string(),
            });
        }

        // Must contain at least one dot (for TLD)
        if !domain.contains('.') {
            return Err(DomainNameError::InvalidFormat {
                domain: domain.to_string(),
                reason: "domain must have at least one dot (e.g., 'example.com')".to_string(),
            });
        }

        // Cannot start or end with dot
        if domain.starts_with('.') || domain.ends_with('.') {
            return Err(DomainNameError::InvalidFormat {
                domain: domain.to_string(),
                reason: "domain cannot start or end with a dot".to_string(),
            });
        }

        // Check for consecutive dots (common typo like "example..com")
        if domain.contains("..") {
            return Err(DomainNameError::InvalidFormat {
                domain: domain.to_string(),
                reason: "domain cannot have consecutive dots".to_string(),
            });
        }

        Ok(())
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<DomainName> for String {
    fn from(domain: DomainName) -> Self {
        domain.0
    }
}

impl AsRef<str> for DomainName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Custom Serialize implementation that outputs the inner String
impl Serialize for DomainName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

// Custom Deserialize implementation that validates on parse
impl<'de> Deserialize<'de> for DomainName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DomainName::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Errors that can occur when parsing or validating a domain name
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainNameError {
    /// The domain string is empty
    EmptyDomain,

    /// The domain format is invalid (likely a typo)
    InvalidFormat {
        /// The invalid domain that was provided
        domain: String,
        /// Detailed reason for the failure
        reason: String,
    },
}

impl fmt::Display for DomainNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomain => write!(f, "domain name cannot be empty"),
            Self::InvalidFormat { domain, reason } => {
                write!(f, "invalid domain '{domain}': {reason}")
            }
        }
    }
}

impl std::error::Error for DomainNameError {}

impl DomainNameError {
    /// Returns actionable help text for this error
    ///
    /// Provides users with guidance on how to fix the error, including
    /// valid examples and common mistakes to avoid.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EmptyDomain => {
                "Domain name cannot be empty.\n\
                 \n\
                 Provide a valid domain name like:\n\
                 - example.com\n\
                 - api.tracker.torrust.org\n\
                 - my-service.example.com"
            }
            Self::InvalidFormat { .. } => {
                "Domain name appears to have a typo.\n\
                 \n\
                 Common mistakes:\n\
                 - Missing TLD: 'localhost' → 'localhost.local' or use a real domain\n\
                 - Extra dot: 'example..com' → 'example.com'\n\
                 - Space in name: 'my domain.com' → 'my-domain.com'\n\
                 \n\
                 Valid examples:\n\
                 - example.com\n\
                 - api.tracker.torrust.org\n\
                 - my-service.example.com"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid domain tests

    #[test]
    fn it_should_create_domain_when_format_is_valid() {
        let domain = DomainName::new("example.com").unwrap();
        assert_eq!(domain.as_str(), "example.com");
    }

    #[test]
    fn it_should_accept_domain_with_subdomain() {
        let domain = DomainName::new("api.tracker.torrust.org").unwrap();
        assert_eq!(domain.as_str(), "api.tracker.torrust.org");
    }

    #[test]
    fn it_should_accept_domain_with_hyphen() {
        let domain = DomainName::new("my-service.example.com").unwrap();
        assert_eq!(domain.as_str(), "my-service.example.com");
    }

    #[test]
    fn it_should_accept_domain_with_numbers() {
        let domain = DomainName::new("api2.tracker123.org").unwrap();
        assert_eq!(domain.as_str(), "api2.tracker123.org");
    }

    #[test]
    fn it_should_accept_domain_with_underscore() {
        // Underscores are technically allowed in some DNS contexts (SRV records)
        // We don't reject them since we use minimal validation
        let domain = DomainName::new("my_service.example.com").unwrap();
        assert_eq!(domain.as_str(), "my_service.example.com");
    }

    // Invalid domain tests (typo detection)

    #[test]
    fn it_should_reject_empty_domain() {
        let result = DomainName::new("");
        assert!(matches!(result, Err(DomainNameError::EmptyDomain)));
    }

    #[test]
    fn it_should_reject_domain_without_tld() {
        let result = DomainName::new("localhost");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_with_leading_space() {
        let result = DomainName::new(" example.com");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_with_trailing_space() {
        let result = DomainName::new("example.com ");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_with_space_in_middle() {
        let result = DomainName::new("my domain.com");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_starting_with_dot() {
        let result = DomainName::new(".example.com");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_ending_with_dot() {
        let result = DomainName::new("example.com.");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    #[test]
    fn it_should_reject_domain_with_consecutive_dots() {
        let result = DomainName::new("example..com");
        assert!(matches!(result, Err(DomainNameError::InvalidFormat { .. })));
    }

    // Utility method tests

    #[test]
    fn it_should_extract_tld() {
        let domain = DomainName::new("api.tracker.torrust.org").unwrap();
        assert_eq!(domain.tld(), "org");
    }

    #[test]
    fn it_should_extract_subdomains() {
        let domain = DomainName::new("api.tracker.torrust.org").unwrap();
        assert_eq!(domain.subdomains(), vec!["api", "tracker", "torrust"]);
    }

    #[test]
    fn it_should_display_domain_correctly() {
        let domain = DomainName::new("example.com").unwrap();
        assert_eq!(format!("{domain}"), "example.com");
    }

    // Conversion tests

    #[test]
    fn it_should_convert_to_string() {
        let domain = DomainName::new("example.com").unwrap();
        let s: String = domain.into();
        assert_eq!(s, "example.com");
    }

    #[test]
    fn it_should_convert_from_string() {
        let domain = DomainName::try_from("example.com".to_string()).unwrap();
        assert_eq!(domain.as_str(), "example.com");
    }

    // Serialization tests

    #[test]
    fn it_should_serialize_to_json() {
        let domain = DomainName::new("example.com").unwrap();
        let json = serde_json::to_string(&domain).unwrap();
        assert_eq!(json, "\"example.com\"");
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let domain: DomainName = serde_json::from_str("\"example.com\"").unwrap();
        assert_eq!(domain.as_str(), "example.com");
    }

    #[test]
    fn it_should_fail_deserialization_for_invalid_domain() {
        let result: Result<DomainName, _> = serde_json::from_str("\"localhost\"");
        assert!(result.is_err());
    }

    // Help text tests

    #[test]
    fn it_should_provide_help_for_empty_domain_error() {
        let error = DomainNameError::EmptyDomain;
        assert!(error.help().contains("Domain name cannot be empty"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_format_error() {
        let error = DomainNameError::InvalidFormat {
            domain: "localhost".to_string(),
            reason: "no TLD".to_string(),
        };
        assert!(error.help().contains("Common mistakes"));
    }
}
