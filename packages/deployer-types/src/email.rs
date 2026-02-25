//! Email Address Type
//!
//! This module provides a strongly-typed email address wrapper using the
//! `email_address` crate for RFC 5321/5322 compliant validation.
//!
//! # Usage
//!
//! ```rust
//! use torrust_deployer_types::Email;
//!
//! // Valid email
//! let email = Email::new("admin@example.com").unwrap();
//! assert_eq!(email.as_str(), "admin@example.com");
//!
//! // Invalid email returns error
//! let result = Email::new("invalid-email");
//! assert!(result.is_err());
//! ```
//!
//! # Design Notes
//!
//! This type is placed in the `shared` module because:
//! - Email is a fundamental concept used across multiple domains
//! - It provides validation without business-specific logic
//! - Both domain and infrastructure layers may need email validation

use std::fmt;
use std::str::FromStr;

use email_address::EmailAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A validated email address
///
/// This type wraps the `email_address` crate to provide RFC-compliant
/// email validation. It can be used in domain entities where email
/// addresses are required.
///
/// # Validation
///
/// The email address must conform to RFC 5321/5322 standards:
/// - Must have a local part and domain separated by `@`
/// - Local part can contain alphanumeric characters and some special characters
/// - Domain must be a valid domain name or IP address
///
/// # Examples
///
/// ```rust
/// use torrust_deployer_types::Email;
///
/// let email = Email::new("user@example.com").unwrap();
/// println!("Email: {}", email);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Email(String);

impl Email {
    /// Creates a new validated email address
    ///
    /// # Arguments
    ///
    /// * `email` - The email address string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(Email)` - If the email is valid
    /// * `Err(EmailError)` - If the email is invalid
    ///
    /// # Errors
    ///
    /// Returns `EmailError::InvalidFormat` if the email doesn't comply with
    /// RFC 5321/5322 standards.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_deployer_types::Email;
    ///
    /// let valid = Email::new("admin@example.com");
    /// assert!(valid.is_ok());
    ///
    /// let invalid = Email::new("not-an-email");
    /// assert!(invalid.is_err());
    /// ```
    pub fn new(email: &str) -> Result<Self, EmailError> {
        // Use email_address crate for validation
        EmailAddress::from_str(email)
            .map(|_| Self(email.to_string()))
            .map_err(|_| EmailError::InvalidFormat {
                email: email.to_string(),
            })
    }

    /// Returns the email address as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the local part of the email (before the `@`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_deployer_types::Email;
    ///
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.local_part(), "user");
    /// ```
    #[must_use]
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    /// Returns the domain part of the email (after the `@`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_deployer_types::Email;
    ///
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.domain_part(), "example.com");
    /// ```
    #[must_use]
    pub fn domain_part(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = EmailError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> Self {
        email.0
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Errors that can occur when creating an email address
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmailError {
    /// The email format is invalid
    #[error("invalid email format: '{email}'")]
    InvalidFormat {
        /// The invalid email string
        email: String,
    },
}

impl EmailError {
    /// Returns actionable help for resolving the error
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::InvalidFormat { email } => {
                format!(
                    "The email address '{email}' is not valid.\n\n\
                     A valid email address must:\n\
                     - Have a local part (before @) and domain part (after @)\n\
                     - Example: admin@example.com\n\n\
                     Please provide a valid email address for Let's Encrypt certificate notifications."
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_email_when_format_is_valid() {
        let email = Email::new("admin@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "admin@example.com");
    }

    #[test]
    fn it_should_accept_email_with_subdomain() {
        let email = Email::new("user@mail.example.com");
        assert!(email.is_ok());
    }

    #[test]
    fn it_should_accept_email_with_plus_sign() {
        let email = Email::new("user+tag@example.com");
        assert!(email.is_ok());
    }

    #[test]
    fn it_should_accept_email_with_dots_in_local_part() {
        let email = Email::new("first.last@example.com");
        assert!(email.is_ok());
    }

    #[test]
    fn it_should_reject_email_without_at_sign() {
        let result = Email::new("invalid-email");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EmailError::InvalidFormat { .. }
        ));
    }

    #[test]
    fn it_should_reject_email_without_domain() {
        let result = Email::new("user@");
        assert!(result.is_err());
    }

    #[test]
    fn it_should_reject_email_without_local_part() {
        let result = Email::new("@example.com");
        assert!(result.is_err());
    }

    #[test]
    fn it_should_reject_empty_email() {
        let result = Email::new("");
        assert!(result.is_err());
    }

    #[test]
    fn it_should_extract_local_part() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), "user");
    }

    #[test]
    fn it_should_extract_domain_part() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.domain_part(), "example.com");
    }

    #[test]
    fn it_should_display_email_correctly() {
        let email = Email::new("admin@example.com").unwrap();
        assert_eq!(format!("{email}"), "admin@example.com");
    }

    #[test]
    fn it_should_convert_from_string() {
        let result: Result<Email, _> = "user@example.com".to_string().try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_convert_to_string() {
        let email = Email::new("user@example.com").unwrap();
        let string: String = email.into();
        assert_eq!(string, "user@example.com");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let email = Email::new("admin@example.com").unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, "\"admin@example.com\"");
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = "\"admin@example.com\"";
        let email: Email = serde_json::from_str(json).unwrap();
        assert_eq!(email.as_str(), "admin@example.com");
    }

    #[test]
    fn it_should_fail_deserialization_for_invalid_email() {
        let json = "\"invalid-email\"";
        let result: Result<Email, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_provide_help_for_invalid_format_error() {
        let error = EmailError::InvalidFormat {
            email: "bad".to_string(),
        };
        let help = error.help();
        assert!(help.contains("not valid"));
        assert!(help.contains("admin@example.com"));
    }
}
