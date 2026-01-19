//! HTTPS Configuration DTOs (Application Layer)
//!
//! This module contains DTO types for HTTPS/TLS configuration used in
//! environment creation. These types enable automatic HTTPS setup with
//! Caddy as a TLS termination proxy.
//!
//! ## Architecture
//!
//! The HTTPS configuration follows a **service-based approach** where:
//! - Common HTTPS settings (admin email, staging flag) are at the top level
//! - Each service (tracker API, HTTP trackers, Grafana) has optional TLS config
//!
//! See [ADR: Service-Based TLS Configuration](../../../../docs/decisions/) for rationale.
//!
//! ## DTO vs Domain Types
//!
//! These types are Data Transfer Objects that use primitive types (`String`) for
//! JSON deserialization. Validation converts to domain types (e.g., `Email`) which
//! provide RFC-compliant validation via external crates like `email_address`.
//!
//! This separation allows:
//! - Clean JSON serialization/deserialization at DTO boundaries
//! - Rich domain validation via strongly-typed domain types
//! - No domain type coupling to serialization concerns
//!
//! ## Security
//!
//! The `admin_email` field is considered semi-sensitive as it's used in
//! Let's Encrypt certificate requests and may be visible in certificate
//! transparency logs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::errors::CreateConfigError;
use crate::shared::Email;

/// Common HTTPS configuration (top-level)
///
/// Contains configuration shared across all TLS-enabled services.
/// This section is required if any service has TLS enabled.
///
/// # Let's Encrypt Environments
///
/// - **Production** (default): Uses `https://acme-v02.api.letsencrypt.org/directory`
///   - Rate limits: 50 certs/week per domain, 5 duplicates/week
///   - Certificates are trusted by all browsers
///
/// - **Staging** (`use_staging: true`): Uses `https://acme-staging-v02.api.letsencrypt.org/directory`
///   - Much higher rate limits for testing
///   - Certificates show browser warnings (not trusted)
///   - Use only for testing the HTTPS flow
///
/// # Examples
///
/// Production configuration:
/// ```json
/// {
///     "https": {
///         "admin_email": "admin@example.com"
///     }
/// }
/// ```
///
/// Staging configuration (for testing):
/// ```json
/// {
///     "https": {
///         "admin_email": "admin@example.com",
///         "use_staging": true
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HttpsSection {
    /// Admin email for Let's Encrypt certificate notifications
    ///
    /// This email will receive:
    /// - Certificate expiration warnings (30 days before expiry)
    /// - Certificate renewal failure notifications
    /// - Important Let's Encrypt service announcements
    ///
    /// **Note**: This email may be publicly visible in certificate transparency logs.
    pub admin_email: String,

    /// Use Let's Encrypt staging environment for testing
    ///
    /// When `true`:
    /// - Uses staging CA: `https://acme-staging-v02.api.letsencrypt.org/directory`
    /// - Certificates will show browser warnings (not trusted by browsers)
    /// - Higher rate limits allow extensive testing
    ///
    /// When `false` or omitted (default):
    /// - Uses production CA: `https://acme-v02.api.letsencrypt.org/directory`
    /// - Certificates are trusted by all browsers
    /// - Subject to rate limits (50 certs/week, 5 duplicates/week)
    #[serde(default)]
    pub use_staging: bool,
}

impl HttpsSection {
    /// Creates a new HTTPS configuration section
    #[must_use]
    pub fn new(admin_email: String, use_staging: bool) -> Self {
        Self {
            admin_email,
            use_staging,
        }
    }

    /// Validates the HTTPS configuration
    ///
    /// Uses the domain-level `Email` type for RFC-compliant validation via
    /// the `email_address` crate.
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidAdminEmail` if the email format is invalid.
    pub fn validate(&self) -> Result<(), CreateConfigError> {
        // Validate email using the domain type for RFC-compliant validation
        Email::new(&self.admin_email).map_err(|e| CreateConfigError::InvalidAdminEmail {
            email: self.admin_email.clone(),
            reason: e.to_string(),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod https_section_tests {
        use super::*;

        #[test]
        fn it_should_create_https_section_with_defaults() {
            let section = HttpsSection::new("admin@example.com".to_string(), false);
            assert_eq!(section.admin_email, "admin@example.com");
            assert!(!section.use_staging);
        }

        #[test]
        fn it_should_create_https_section_with_staging() {
            let section = HttpsSection::new("admin@example.com".to_string(), true);
            assert!(section.use_staging);
        }

        #[test]
        fn it_should_validate_valid_email() {
            let section = HttpsSection::new("admin@example.com".to_string(), false);
            assert!(section.validate().is_ok());
        }

        #[test]
        fn it_should_reject_email_without_at_symbol() {
            let section = HttpsSection::new("invalid-email".to_string(), false);
            let result = section.validate();
            assert!(result.is_err());
            if let Err(CreateConfigError::InvalidAdminEmail { email, .. }) = result {
                assert_eq!(email, "invalid-email");
            } else {
                panic!("Expected InvalidAdminEmail error");
            }
        }

        #[test]
        fn it_should_reject_email_with_empty_local_part() {
            let section = HttpsSection::new("@example.com".to_string(), false);
            assert!(section.validate().is_err());
        }

        #[test]
        fn it_should_reject_email_with_empty_domain_part() {
            let section = HttpsSection::new("admin@".to_string(), false);
            assert!(section.validate().is_err());
        }

        #[test]
        fn it_should_deserialize_from_json() {
            let json = r#"{"admin_email": "test@example.com", "use_staging": true}"#;
            let section: HttpsSection = serde_json::from_str(json).unwrap();
            assert_eq!(section.admin_email, "test@example.com");
            assert!(section.use_staging);
        }

        #[test]
        fn it_should_deserialize_with_default_use_staging() {
            let json = r#"{"admin_email": "test@example.com"}"#;
            let section: HttpsSection = serde_json::from_str(json).unwrap();
            assert!(!section.use_staging);
        }
    }

    /// Tests for email validation in HTTPS context
    ///
    /// Note: Comprehensive email format validation tests are in `src/shared/email.rs`.
    /// These tests verify the integration of the `Email` type with `HttpsSection`.
    mod email_validation_integration_tests {
        use super::*;

        #[test]
        fn it_should_accept_rfc_compliant_email() {
            let section = HttpsSection::new("user@example.com".to_string(), false);
            assert!(section.validate().is_ok());
        }

        #[test]
        fn it_should_reject_rfc_non_compliant_email() {
            let section = HttpsSection::new("invalid-email".to_string(), false);
            let result = section.validate();
            assert!(matches!(
                result,
                Err(CreateConfigError::InvalidAdminEmail { .. })
            ));
        }
    }
}
