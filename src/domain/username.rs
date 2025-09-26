//! Linux system username validation and management
//!
//! This module provides the `Username` type which ensures usernames follow
//! Linux system username requirements for security, compatibility, and
//! system integration.
//!
//! ## Naming Requirements
//!
//! - Length: 1-32 characters (standard Linux limit)
//! - First character: Must be a letter (a-z, A-Z) or underscore (_)
//! - Subsequent characters: lowercase letters, digits, underscores, and hyphens
//! - Cannot contain spaces or special characters
//! - Case-sensitive (though lowercase is conventional)
//!
//! These restrictions ensure compatibility with Linux system calls, SSH authentication,
//! file ownership, and process management across different distributions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during username validation
#[derive(Debug, Error, PartialEq)]
pub enum UsernameError {
    #[error("Username cannot be empty")]
    Empty,

    #[error("Username must be 32 characters or less, got {length} characters")]
    TooLong { length: usize },

    #[error("Username must start with a letter (a-z, A-Z) or underscore (_)")]
    InvalidFirstCharacter,

    #[error("Username must contain only letters, digits, underscores, and hyphens")]
    InvalidCharacters,
}

/// A validated Linux system username following Linux naming requirements.
///
/// Valid usernames must fulfill the following requirements:
/// - The username must be between 1 and 32 characters long
/// - The username must start with a letter (a-z, A-Z) or underscore (_)
/// - The username can contain lowercase letters, digits, underscores, and hyphens
/// - The username cannot contain spaces or other special characters
///
/// These requirements ensure that the username can be used for:
/// - SSH authentication
/// - File and directory ownership
/// - Process ownership and management
/// - System user identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Creates a new `Username` from a string if it's valid.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to validate (accepts `&str`, `String`, or anything that implements `Into<String>`)
    ///
    /// # Returns
    ///
    /// * `Ok(Username)` - If the username is valid
    /// * `Err(UsernameError)` - If the username violates Linux naming requirements
    ///
    /// # Errors
    ///
    /// This function will return an error if the username violates any Linux naming requirements:
    /// * Empty username
    /// * Username longer than 32 characters
    /// * Username contains invalid characters
    /// * Username starts with a digit or invalid character
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::Username;
    ///
    /// // Valid usernames - accepts both &str and String
    /// let user1 = Username::new("torrust")?;
    /// let user2 = Username::new("user_123".to_string())?;
    /// let user3 = Username::new("deploy-user")?;
    /// let user4 = Username::new("_service")?;
    ///
    /// // Invalid usernames
    /// assert!(Username::new("").is_err());
    /// assert!(Username::new("user@domain").is_err());
    /// assert!(Username::new("123user").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<S: Into<String>>(username: S) -> Result<Self, UsernameError> {
        let username = username.into();
        Self::validate(&username)?;
        Ok(Self(username))
    }

    /// Returns the username as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates a username according to Linux system requirements.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the username is valid
    /// * `Err(UsernameError)` - If the username violates any requirement
    fn validate(username: &str) -> Result<(), UsernameError> {
        // Check length: must be between 1 and 32 characters
        if username.is_empty() {
            return Err(UsernameError::Empty);
        }
        if username.len() > 32 {
            return Err(UsernameError::TooLong {
                length: username.len(),
            });
        }

        // Check first character: must be a letter or underscore
        if let Some(first_char) = username.chars().next() {
            if !first_char.is_ascii_alphabetic() && first_char != '_' {
                return Err(UsernameError::InvalidFirstCharacter);
            }
        }

        // Check all characters: letters, digits, underscores, and hyphens only
        if !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(UsernameError::InvalidCharacters);
        }

        Ok(())
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Username {
    type Err = UsernameError;

    fn from_str(s: &str) -> Result<Self, UsernameError> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_valid_username() {
        let username = Username::new("torrust").unwrap();
        assert_eq!(username.as_str(), "torrust");
    }

    #[test]
    fn it_should_accept_string_slice_and_owned_string() {
        // Test with &str
        let username1 = Username::new("torrust").unwrap();
        assert_eq!(username1.as_str(), "torrust");

        // Test with String
        let username2 = Username::new("torrust".to_string()).unwrap();
        assert_eq!(username2.as_str(), "torrust");

        // Test with String from format!
        let username3 = Username::new(format!("user_{}", 123)).unwrap();
        assert_eq!(username3.as_str(), "user_123");

        // Test that both are equal
        assert_eq!(username1, username2);
    }

    #[test]
    fn it_should_create_username_with_numbers() {
        let username = Username::new("user123").unwrap();
        assert_eq!(username.as_str(), "user123");
    }

    #[test]
    fn it_should_create_username_with_underscores() {
        let username = Username::new("deploy_user").unwrap();
        assert_eq!(username.as_str(), "deploy_user");
    }

    #[test]
    fn it_should_create_username_with_hyphens() {
        let username = Username::new("deploy-user").unwrap();
        assert_eq!(username.as_str(), "deploy-user");
    }

    #[test]
    fn it_should_create_username_starting_with_underscore() {
        let username = Username::new("_service").unwrap();
        assert_eq!(username.as_str(), "_service");
    }

    #[test]
    fn it_should_create_single_character_username() {
        let username = Username::new("a").unwrap();
        assert_eq!(username.as_str(), "a");
    }

    #[test]
    fn it_should_create_32_character_username() {
        let long_username = "a".repeat(32);
        let username = Username::new(long_username.clone()).unwrap();
        assert_eq!(username.as_str(), long_username);
    }

    #[test]
    fn it_should_reject_empty_username() {
        let result = Username::new("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn it_should_reject_username_longer_than_32_characters() {
        let long_username = "a".repeat(33);
        let result = Username::new(long_username);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("32 characters or less"));
    }

    #[test]
    fn it_should_reject_username_starting_with_digit() {
        let result = Username::new("123user");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with a letter"));
    }

    #[test]
    fn it_should_reject_username_starting_with_special_character() {
        let invalid_starters = vec!["@user", "#user", "$user", "%user", "-user"];

        for invalid_username in invalid_starters {
            let result = Username::new(invalid_username);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("must start with a letter"));
        }
    }

    #[test]
    fn it_should_reject_username_with_invalid_characters() {
        let invalid_chars = vec![
            "user@domain",
            "user.name",
            "user name",
            "user#123",
            "user$test",
            "user%test",
            "user*test",
            "user+test",
            "user=test",
            "user[test]",
            "user{test}",
            "user|test",
            "user\\test",
            "user:test",
            "user;test",
            "user\"test",
            "user'test",
            "user<test>",
            "user,test",
            "user?test",
            "user/test",
        ];

        for invalid_username in invalid_chars {
            let result = Username::new(invalid_username);
            assert!(
                result.is_err(),
                "Username '{invalid_username}' should be invalid"
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("must contain only letters, digits, underscores, and hyphens"));
        }
    }

    #[test]
    fn it_should_accept_mixed_case_usernames() {
        let mixed_case_usernames = vec!["User", "USER", "TorrustUser", "Deploy_USER"];

        for username in mixed_case_usernames {
            let result = Username::new(username);
            assert!(result.is_ok(), "Username '{username}' should be valid");
        }
    }

    #[test]
    fn it_should_display_username() {
        let username = Username::new("torrust").unwrap();
        assert_eq!(format!("{username}"), "torrust");
    }

    #[test]
    fn it_should_parse_valid_username_from_string() {
        let username: Username = "torrust".parse().unwrap();
        assert_eq!(username.as_str(), "torrust");
    }

    #[test]
    fn it_should_fail_parsing_invalid_username_from_string() {
        let result: Result<Username, UsernameError> = "invalid@username".parse();
        assert!(result.is_err());
    }

    #[test]
    fn it_should_implement_as_ref() {
        let username = Username::new("torrust").unwrap();
        let username_ref: &str = username.as_ref();
        assert_eq!(username_ref, "torrust");
    }

    #[test]
    fn it_should_be_cloneable_and_comparable() {
        let username1 = Username::new("torrust").unwrap();
        let username2 = username1.clone();
        assert_eq!(username1, username2);
    }

    #[test]
    fn it_should_be_serializable_and_deserializable() {
        let username = Username::new("torrust").unwrap();

        // Test serialization
        let json = serde_json::to_string(&username).unwrap();
        assert_eq!(json, "\"torrust\"");

        // Test deserialization
        let deserialized: Username = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, username);
    }

    #[test]
    fn it_should_work_with_common_linux_usernames() {
        let common_usernames = vec![
            "admin",
            "ubuntu",
            "debian",
            "centos",
            "fedora",
            "user",
            "guest",
            "deploy",
            "www-data",
            "nginx",
            "apache",
            "mysql",
            "postgres",
            "redis",
            "mongodb",
            "elastic",
            "docker",
            "jenkins",
            "git",
            "ftp",
            "mail",
            "service_account",
            "app-user",
            "torrust",
        ];

        for username in common_usernames {
            let result = Username::new(username);
            assert!(result.is_ok(), "Username '{username}' should be valid");
        }
    }

    #[test]
    fn it_should_return_specific_error_for_empty_username() {
        let result = Username::new("");
        assert!(matches!(result.unwrap_err(), UsernameError::Empty));
    }

    #[test]
    fn it_should_return_specific_error_for_too_long_username() {
        let long_username = "a".repeat(33);
        let result = Username::new(long_username);
        assert!(matches!(
            result.unwrap_err(),
            UsernameError::TooLong { length: 33 }
        ));
    }

    #[test]
    fn it_should_return_specific_error_for_invalid_first_character() {
        let result = Username::new("123user");
        assert!(matches!(
            result.unwrap_err(),
            UsernameError::InvalidFirstCharacter
        ));
    }

    #[test]
    fn it_should_return_specific_error_for_invalid_characters() {
        let result = Username::new("user@domain");
        assert!(matches!(
            result.unwrap_err(),
            UsernameError::InvalidCharacters
        ));
    }

    #[test]
    fn it_should_display_error_messages_correctly() {
        let empty_error = UsernameError::Empty;
        assert_eq!(empty_error.to_string(), "Username cannot be empty");

        let too_long_error = UsernameError::TooLong { length: 40 };
        assert_eq!(
            too_long_error.to_string(),
            "Username must be 32 characters or less, got 40 characters"
        );

        let invalid_first_char_error = UsernameError::InvalidFirstCharacter;
        assert_eq!(
            invalid_first_char_error.to_string(),
            "Username must start with a letter (a-z, A-Z) or underscore (_)"
        );

        let invalid_chars_error = UsernameError::InvalidCharacters;
        assert_eq!(
            invalid_chars_error.to_string(),
            "Username must contain only letters, digits, underscores, and hyphens"
        );
    }

    #[test]
    fn it_should_support_error_equality_comparison() {
        let error1 = UsernameError::Empty;
        let error2 = UsernameError::Empty;
        let error3 = UsernameError::InvalidFirstCharacter;

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
