//! API token secret wrapper type

use secrecy::{ExposeSecret as SecrecyExposeSecret, SecretString};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;

/// Plain API token used in DTOs for serialization/deserialization.
///
/// This is intentionally a `String` type alias to mark places where API tokens
/// are handled in plain text during configuration file I/O at the application layer.
/// Convert to the secure `ApiToken` type at the DTO-to-domain boundary using `ApiToken::from()`.
///
/// # Lifecycle
///
/// ```text
/// User Input (JSON) → PlainApiToken (DTO) → ApiToken (Domain) → PlainApiToken (JSON Output)
/// ```
pub type PlainApiToken = String;

/// A wrapper type for API tokens that prevents accidental exposure.
///
/// Internally uses `SecretString` from the `secrecy` crate for memory zeroing
/// on drop and debug redaction. Adds serialization support by exposing the
/// secret during serialization operations.
#[derive(Clone)]
pub struct ApiToken(SecretString);

impl ApiToken {
    /// Creates a new API token from any type that can be converted into a String.
    ///
    /// # Examples
    /// ```
    /// use torrust_tracker_deployer_lib::shared::secrets::ApiToken;
    ///
    /// let from_string = ApiToken::new(String::from("token"));
    /// let from_str = ApiToken::new("token");
    /// ```
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self(SecretString::from(token.into()))
    }

    /// Exposes the secret API token value.
    ///
    /// This method should be used carefully as it provides access to the sensitive data.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to SecretString's Debug impl which shows "Secret([REDACTED])"
        f.debug_tuple("ApiToken").field(&self.0).finish()
    }
}

impl PartialEq for ApiToken {
    fn eq(&self, other: &Self) -> bool {
        self.expose_secret() == other.expose_secret()
    }
}

impl Eq for ApiToken {}

impl Serialize for ApiToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Deliberately expose secret during serialization for storage
        self.expose_secret().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ApiToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<String> for ApiToken {
    fn from(token: String) -> Self {
        Self::new(token)
    }
}

impl From<&str> for ApiToken {
    fn from(token: &str) -> Self {
        Self::new(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_api_token_from_string() {
        let token = ApiToken::new("test-token".to_string());
        assert_eq!(token.expose_secret(), "test-token");
    }

    #[test]
    fn it_should_create_api_token_from_str_reference() {
        let token = ApiToken::from("test-token");
        assert_eq!(token.expose_secret(), "test-token");
    }

    #[test]
    fn it_should_redact_api_token_in_debug_output() {
        let token = ApiToken::new("secret-token".to_string());
        let debug_output = format!("{token:?}");
        assert!(debug_output.contains("Secret"));
        assert!(!debug_output.contains("secret-token"));
    }

    #[test]
    fn it_should_clone_api_token() {
        let token = ApiToken::new("test-token".to_string());
        let cloned = token.clone();
        assert_eq!(token.expose_secret(), cloned.expose_secret());
    }

    #[test]
    fn it_should_compare_api_tokens_for_equality() {
        let token1 = ApiToken::new("token".to_string());
        let token2 = ApiToken::new("token".to_string());
        let token3 = ApiToken::new("different".to_string());

        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
    }

    #[test]
    fn it_should_serialize_and_deserialize_api_token() {
        let original = ApiToken::new("test-token".to_string());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ApiToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original.expose_secret(), deserialized.expose_secret());
    }

    #[test]
    fn it_should_support_multiple_input_types_for_api_token() {
        let from_string = ApiToken::from("token".to_string());
        let from_str = ApiToken::from("token");
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn it_should_accept_flexible_constructor_inputs_for_api_token() {
        // Test that new() accepts anything convertible to String
        let _from_string_type = ApiToken::new(String::from("token"));
        let _from_str_slice = ApiToken::new("token");
        let _from_owned_string = ApiToken::new("token".to_string());
    }
}
