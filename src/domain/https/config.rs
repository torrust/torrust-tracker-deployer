//! HTTPS configuration domain type
//!
//! This module defines the domain-level HTTPS configuration that is stored
//! in the environment and used to configure Caddy TLS termination.
//!
//! ## Domain vs DTO
//!
//! This is the domain type. The DTO version (`HttpsSection`) is in the
//! application layer at `src/application/command_handlers/create/config/https.rs`.
//!
//! The domain type is validated when created from the DTO and carries
//! the configuration through the environment lifecycle.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::shared::Email;

/// Error type for `HttpsConfig` construction failures
///
/// Contains validation errors that can occur when constructing an `HttpsConfig`.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum HttpsConfigError {
    /// The admin email address is invalid
    #[error("Invalid admin email '{email}': {reason}")]
    InvalidEmail {
        /// The invalid email that was provided
        email: String,
        /// The reason why the email is invalid
        reason: String,
    },
}

impl HttpsConfigError {
    /// Returns actionable help text for this error
    ///
    /// Provides detailed guidance on how to fix the configuration issue.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEmail { .. } => {
                "Invalid admin email format.\n\
                 \n\
                 The admin email is used by Let's Encrypt to send:\n\
                 - Certificate expiration warnings\n\
                 - Renewal failure notifications\n\
                 - Important security updates\n\
                 \n\
                 Requirements:\n\
                 - Must be a valid email format (e.g., admin@example.com)\n\
                 - Should be a monitored mailbox for security alerts\n\
                 \n\
                 Fix:\n\
                 Update the admin_email in your HTTPS configuration:\n\
                 \n\
                 \"https\": {\n\
                   \"admin_email\": \"admin@yourdomain.com\",\n\
                   \"use_staging\": false\n\
                 }"
            }
        }
    }
}

/// Domain-level HTTPS configuration for TLS termination
///
/// Contains validated HTTPS settings used for Caddy reverse proxy configuration.
/// This type is created from the application-layer DTO (`HttpsSection`) after
/// validation and stored in the environment.
///
/// # Let's Encrypt Environments
///
/// - **Production** (default): Trusted certificates, rate-limited
/// - **Staging**: Untrusted test certificates, higher rate limits
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::https::HttpsConfig;
///
/// let config = HttpsConfig::new("admin@example.com", false).unwrap();
/// assert_eq!(config.admin_email(), "admin@example.com");
/// assert!(!config.use_staging());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpsConfig {
    /// Admin email for Let's Encrypt notifications
    ///
    /// Receives certificate expiration warnings and renewal failure notifications.
    admin_email: String,

    /// Whether to use Let's Encrypt staging environment
    ///
    /// - `true`: Use staging CA (for testing, certificates not trusted)
    /// - `false`: Use production CA (trusted certificates)
    use_staging: bool,
}

impl HttpsConfig {
    /// Creates a new HTTPS configuration with validated email
    ///
    /// Validates the admin email format at construction time, ensuring
    /// the configuration is always valid.
    ///
    /// # Arguments
    ///
    /// * `admin_email` - Admin email for Let's Encrypt notifications
    /// * `use_staging` - Whether to use staging environment
    ///
    /// # Errors
    ///
    /// Returns `HttpsConfigError::InvalidEmail` if the email format is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::https::HttpsConfig;
    ///
    /// // Production configuration
    /// let config = HttpsConfig::new("admin@example.com", false).unwrap();
    /// assert!(!config.use_staging());
    ///
    /// // Staging configuration (for testing)
    /// let staging = HttpsConfig::new("admin@example.com", true).unwrap();
    /// assert!(staging.use_staging());
    ///
    /// // Invalid email is rejected
    /// let result = HttpsConfig::new("invalid-email", false);
    /// assert!(result.is_err());
    /// ```
    pub fn new(
        admin_email: impl Into<String>,
        use_staging: bool,
    ) -> Result<Self, HttpsConfigError> {
        let email_str = admin_email.into();

        // Validate email format using the shared Email type
        Email::new(&email_str).map_err(|e| HttpsConfigError::InvalidEmail {
            email: email_str.clone(),
            reason: e.to_string(),
        })?;

        Ok(Self {
            admin_email: email_str,
            use_staging,
        })
    }

    /// Creates an HTTPS config from a validated email
    ///
    /// This is the preferred factory method when working with validated
    /// email addresses from the application layer. Since the email is
    /// already validated, this method is infallible.
    ///
    /// # Arguments
    ///
    /// * `email` - Validated email address
    /// * `use_staging` - Whether to use staging environment
    #[must_use]
    pub fn from_validated_email(email: &Email, use_staging: bool) -> Self {
        Self {
            admin_email: email.to_string(),
            use_staging,
        }
    }

    /// Returns the admin email address
    #[must_use]
    pub fn admin_email(&self) -> &str {
        &self.admin_email
    }

    /// Returns whether to use Let's Encrypt staging environment
    #[must_use]
    pub fn use_staging(&self) -> bool {
        self.use_staging
    }
}

impl Default for HttpsConfig {
    /// Creates a default HTTPS configuration
    ///
    /// Uses a placeholder email that should be replaced before deployment.
    fn default() -> Self {
        Self {
            admin_email: "admin@example.com".to_string(),
            use_staging: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_https_config_with_production_ca() {
        let config = HttpsConfig::new("admin@tracker.example.com", false)
            .expect("valid email should succeed");

        assert_eq!(config.admin_email(), "admin@tracker.example.com");
        assert!(!config.use_staging());
    }

    #[test]
    fn it_should_create_https_config_with_staging_ca() {
        let config = HttpsConfig::new("admin@tracker.example.com", true)
            .expect("valid email should succeed");

        assert_eq!(config.admin_email(), "admin@tracker.example.com");
        assert!(config.use_staging());
    }

    #[test]
    fn it_should_reject_invalid_email() {
        let result = HttpsConfig::new("invalid-email", false);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HttpsConfigError::InvalidEmail { .. }));
    }

    #[test]
    fn it_should_reject_email_without_at_symbol() {
        let result = HttpsConfig::new("admin.example.com", false);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_create_default_https_config() {
        let config = HttpsConfig::default();

        assert_eq!(config.admin_email(), "admin@example.com");
        assert!(!config.use_staging());
    }

    #[test]
    fn it_should_serialize_to_json() {
        let config =
            HttpsConfig::new("admin@example.com", true).expect("valid email should succeed");

        let json = serde_json::to_string(&config).expect("serialization should succeed");

        assert!(json.contains("\"admin_email\":\"admin@example.com\""));
        assert!(json.contains("\"use_staging\":true"));
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = r#"{"admin_email":"test@example.com","use_staging":false}"#;

        let config: HttpsConfig =
            serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(config.admin_email(), "test@example.com");
        assert!(!config.use_staging());
    }

    #[test]
    fn it_should_be_cloneable() {
        let config =
            HttpsConfig::new("admin@example.com", true).expect("valid email should succeed");
        let cloned = config.clone();

        assert_eq!(config, cloned);
    }

    #[test]
    fn it_should_create_from_validated_email() {
        let email = Email::new("admin@example.com").expect("valid email");
        let config = HttpsConfig::from_validated_email(&email, false);

        assert_eq!(config.admin_email(), "admin@example.com");
        assert!(!config.use_staging());
    }

    #[test]
    fn it_should_provide_help_for_invalid_email_error() {
        let err = HttpsConfigError::InvalidEmail {
            email: "bad".to_string(),
            reason: "missing @".to_string(),
        };

        let help = err.help();
        assert!(help.contains("Invalid admin email format"));
        assert!(help.contains("Let's Encrypt"));
    }
}
