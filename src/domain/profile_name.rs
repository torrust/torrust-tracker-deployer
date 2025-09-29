//! LXD profile name validation and management
//!
//! This module provides the `ProfileName` type which ensures LXD profile names
//! follow the strict naming requirements imposed by LXD for security and
//! compatibility reasons.
//!
//! ## Naming Requirements
//!
//! - Length: 1-63 characters
//! - Characters: ASCII letters, numbers, and dashes only
//! - Cannot start with digit or dash
//! - Cannot end with dash
//!
//! These restrictions ensure compatibility with DNS records, file systems,
//! security profiles, and host names across different environments.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during profile name validation
#[derive(Debug, Error, PartialEq)]
pub enum ProfileNameError {
    #[error("Profile name cannot be empty")]
    Empty,

    #[error("Profile name must be 63 characters or less, got {length} characters")]
    TooLong { length: usize },

    #[error("Profile name must not start with a digit or dash")]
    InvalidFirstCharacter,

    #[error("Profile name must not end with a dash")]
    InvalidLastCharacter,

    #[error("Profile name must contain only ASCII letters, numbers, and dashes")]
    InvalidCharacters,
}

/// A validated LXD profile name following LXD naming requirements.
///
/// Valid profile names must fulfill the following requirements:
/// - The name must be between 1 and 63 characters long
/// - The name must contain only letters, numbers and dashes from the ASCII table
/// - The name must not start with a digit or a dash
/// - The name must not end with a dash
///
/// These requirements ensure that the profile name can be used in DNS records,
/// on the file system, in various security profiles and as the host name.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::domain::ProfileName;
///
/// // Valid profile names - accepts both &str and String
/// let profile1 = ProfileName::new("test-profile")?;
/// let profile2 = ProfileName::new("web-server-profile".to_string())?;
/// let profile3 = ProfileName::new(format!("app-{}-profile", "prod"))?;
///
/// // Invalid profile names
/// assert!(ProfileName::new("").is_err());
/// assert!(ProfileName::new("test-").is_err());
/// assert!(ProfileName::new("-test").is_err());
/// assert!(ProfileName::new("1test").is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ProfileName(String);

impl ProfileName {
    /// Creates a new `ProfileName` from a string if it's valid.
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name to validate (accepts `&str`, `String`, or anything that implements `Into<String>`)
    ///
    /// # Returns
    ///
    /// * `Ok(ProfileName)` - If the name is valid
    /// * `Err(ProfileNameError)` - If the name violates LXD naming requirements
    ///
    /// # Errors
    ///
    /// This function will return an error if the name violates any LXD naming requirements:
    /// * Empty name
    /// * Name longer than 63 characters
    /// * Name contains non-ASCII letters, numbers, or dashes
    /// * Name starts with a digit or dash
    /// * Name ends with a dash
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::ProfileName;
    ///
    /// // Valid names - accepts both &str and String
    /// let name1 = ProfileName::new("test-profile")?;
    /// let name2 = ProfileName::new("web-01-profile".to_string())?;
    /// let name3 = ProfileName::new(format!("app-{}-profile", "prod"))?;
    ///
    /// // Invalid names
    /// assert!(ProfileName::new("").is_err());
    /// assert!(ProfileName::new("test-").is_err());
    /// assert!(ProfileName::new("-test").is_err());
    /// assert!(ProfileName::new("1test").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Result<Self, ProfileNameError> {
        let name = name.into();
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Returns the profile name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates a profile name according to LXD requirements.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the name is valid
    /// * `Err(ProfileNameError)` - If the name violates any requirement
    fn validate(name: &str) -> Result<(), ProfileNameError> {
        // Check length: must be between 1 and 63 characters
        if name.is_empty() {
            return Err(ProfileNameError::Empty);
        }
        if name.len() > 63 {
            return Err(ProfileNameError::TooLong { length: name.len() });
        }

        // Check characters: only ASCII letters, numbers, and dashes
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ProfileNameError::InvalidCharacters);
        }

        // Check first character: must not be a digit or dash
        if let Some(first_char) = name.chars().next() {
            if first_char.is_ascii_digit() || first_char == '-' {
                return Err(ProfileNameError::InvalidFirstCharacter);
            }
        }

        // Check last character: must not be a dash
        if let Some(last_char) = name.chars().last() {
            if last_char == '-' {
                return Err(ProfileNameError::InvalidLastCharacter);
            }
        }

        Ok(())
    }
}

impl fmt::Display for ProfileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ProfileName {
    type Err = ProfileNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for ProfileName {
    type Error = ProfileNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ProfileName> for String {
    fn from(profile_name: ProfileName) -> Self {
        profile_name.0
    }
}

impl AsRef<str> for ProfileName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_valid_profile_name() {
        let profile = ProfileName::new("torrust-profile").unwrap();
        assert_eq!(profile.as_str(), "torrust-profile");
    }

    #[test]
    fn it_should_accept_string_input() {
        let profile_str = "test-profile".to_string();
        let profile = ProfileName::new(profile_str).unwrap();
        assert_eq!(profile.as_str(), "test-profile");
    }

    #[test]
    fn it_should_accept_formatted_string() {
        let env = "production";
        let profile = ProfileName::new(format!("torrust-profile-{env}")).unwrap();
        assert_eq!(profile.as_str(), "torrust-profile-production");
    }

    #[test]
    fn it_should_create_profile_name_with_numbers() {
        let profile = ProfileName::new("profile123").unwrap();
        assert_eq!(profile.as_str(), "profile123");
    }

    #[test]
    fn it_should_create_profile_name_with_dashes() {
        let profile = ProfileName::new("test-profile-name").unwrap();
        assert_eq!(profile.as_str(), "test-profile-name");
    }

    #[test]
    fn it_should_reject_empty_profile_name() {
        let result = ProfileName::new("");
        assert!(matches!(result, Err(ProfileNameError::Empty)));
    }

    #[test]
    fn it_should_reject_profile_name_starting_with_digit() {
        let result = ProfileName::new("1profile");
        assert!(matches!(
            result,
            Err(ProfileNameError::InvalidFirstCharacter)
        ));
    }

    #[test]
    fn it_should_reject_profile_name_starting_with_dash() {
        let result = ProfileName::new("-profile");
        assert!(matches!(
            result,
            Err(ProfileNameError::InvalidFirstCharacter)
        ));
    }

    #[test]
    fn it_should_reject_profile_name_ending_with_dash() {
        let result = ProfileName::new("profile-");
        assert!(matches!(
            result,
            Err(ProfileNameError::InvalidLastCharacter)
        ));
    }

    #[test]
    fn it_should_reject_profile_name_with_invalid_characters() {
        let result = ProfileName::new("test@profile");
        assert!(matches!(result, Err(ProfileNameError::InvalidCharacters)));

        let result = ProfileName::new("test_profile");
        assert!(matches!(result, Err(ProfileNameError::InvalidCharacters)));

        let result = ProfileName::new("test profile");
        assert!(matches!(result, Err(ProfileNameError::InvalidCharacters)));
    }

    #[test]
    fn it_should_reject_profile_name_too_long() {
        let long_name = "a".repeat(64);
        let result = ProfileName::new(long_name);
        assert!(matches!(
            result,
            Err(ProfileNameError::TooLong { length: 64 })
        ));
    }

    #[test]
    fn it_should_accept_max_length_profile_name() {
        let max_length_name = "a".repeat(63);
        let result = ProfileName::new(max_length_name);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_implement_from_str() {
        let profile: ProfileName = "test-profile".parse().unwrap();
        assert_eq!(profile.as_str(), "test-profile");

        let invalid: Result<ProfileName, _> = "".parse();
        assert!(invalid.is_err());
    }

    #[test]
    fn it_should_implement_try_from_string() {
        let profile = ProfileName::try_from("test-profile".to_string()).unwrap();
        assert_eq!(profile.as_str(), "test-profile");

        let invalid = ProfileName::try_from(String::new());
        assert!(invalid.is_err());
    }

    #[test]
    fn it_should_convert_to_string() {
        let profile = ProfileName::new("test-profile").unwrap();
        let string: String = profile.into();
        assert_eq!(string, "test-profile");
    }

    #[test]
    fn it_should_implement_as_ref() {
        let profile = ProfileName::new("test-profile").unwrap();
        let s: &str = profile.as_ref();
        assert_eq!(s, "test-profile");
    }

    #[test]
    fn it_should_display_profile_name() {
        let profile = ProfileName::new("test-profile").unwrap();
        assert_eq!(format!("{profile}"), "test-profile");
    }

    #[test]
    fn it_should_be_cloneable() {
        let profile = ProfileName::new("test-profile").unwrap();
        let cloned = profile.clone();
        assert_eq!(profile, cloned);
    }

    #[test]
    fn it_should_be_hashable() {
        use std::collections::HashMap;

        let profile = ProfileName::new("test-profile").unwrap();
        let mut map = HashMap::new();
        map.insert(profile, "value");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn it_should_serialize_and_deserialize() {
        let profile = ProfileName::new("test-profile").unwrap();
        let json = serde_json::to_string(&profile).unwrap();
        assert_eq!(json, "\"test-profile\"");

        let deserialized: ProfileName = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, profile);
    }

    #[test]
    fn it_should_fail_deserialization_for_invalid_names() {
        let invalid_json = "\"\""; // Empty string
        let result: Result<ProfileName, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        let invalid_json = "\"-invalid\""; // Starts with dash
        let result: Result<ProfileName, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}
