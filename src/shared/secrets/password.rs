//! Password secret wrapper type

use secrecy::{ExposeSecret as SecrecyExposeSecret, SecretString};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;

/// Plain password used in DTOs for serialization/deserialization.
///
/// This is intentionally a `String` type alias to mark places where passwords
/// are handled in plain text during configuration file I/O at the application layer.
/// Convert to the secure `Password` type at the DTO-to-domain boundary using `Password::from()`.
///
/// # Lifecycle
///
/// ```text
/// User Input (JSON) → PlainPassword (DTO) → Password (Domain) → PlainPassword (JSON Output)
/// ```
pub type PlainPassword = String;

/// A wrapper type for passwords that prevents accidental exposure.
///
/// Internally uses `SecretString` from the `secrecy` crate for memory zeroing
/// on drop and debug redaction. Adds serialization support by exposing the
/// secret during serialization operations.
#[derive(Clone)]
pub struct Password(SecretString);

impl Password {
    /// Creates a new password from any type that can be converted into a String.
    ///
    /// # Examples
    /// ```
    /// use torrust_tracker_deployer_lib::shared::secrets::Password;
    ///
    /// let from_string = Password::new(String::from("password"));
    /// let from_str = Password::new("password");
    /// ```
    #[must_use]
    pub fn new(password: impl Into<String>) -> Self {
        Self(SecretString::from(password.into()))
    }

    /// Exposes the secret password value.
    ///
    /// This method should be used carefully as it provides access to the sensitive data.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to SecretString's Debug impl which shows "Secret([REDACTED])"
        f.debug_tuple("Password").field(&self.0).finish()
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.expose_secret() == other.expose_secret()
    }
}

impl Eq for Password {}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Deliberately expose secret during serialization for storage
        self.expose_secret().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<String> for Password {
    fn from(password: String) -> Self {
        Self::new(password)
    }
}

impl From<&str> for Password {
    fn from(password: &str) -> Self {
        Self::new(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_password_from_string() {
        let password = Password::new("test-password".to_string());
        assert_eq!(password.expose_secret(), "test-password");
    }

    #[test]
    fn it_should_create_password_from_str_reference() {
        let password = Password::from("test-password");
        assert_eq!(password.expose_secret(), "test-password");
    }

    #[test]
    fn it_should_redact_password_in_debug_output() {
        let password = Password::new("secret-pass".to_string());
        let debug_output = format!("{password:?}");
        assert!(debug_output.contains("Secret"));
        assert!(!debug_output.contains("secret-pass"));
    }

    #[test]
    fn it_should_clone_password() {
        let password = Password::new("test-password".to_string());
        let cloned = password.clone();
        assert_eq!(password.expose_secret(), cloned.expose_secret());
    }

    #[test]
    fn it_should_compare_passwords_for_equality() {
        let password1 = Password::new("pass".to_string());
        let password2 = Password::new("pass".to_string());
        let password3 = Password::new("different".to_string());

        assert_eq!(password1, password2);
        assert_ne!(password1, password3);
    }

    #[test]
    fn it_should_serialize_and_deserialize_password() {
        let original = Password::new("test-password".to_string());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Password = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original.expose_secret(), deserialized.expose_secret());
    }

    #[test]
    fn it_should_support_multiple_input_types_for_password() {
        let from_string = Password::from("password".to_string());
        let from_str = Password::from("password");
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn it_should_accept_flexible_constructor_inputs_for_password() {
        // Test that new() accepts anything convertible to String
        let _from_string_type = Password::new(String::from("password"));
        let _from_str_slice = Password::new("password");
        let _from_owned_string = Password::new("password".to_string());
    }
}
