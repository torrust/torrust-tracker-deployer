//! Instance name validation and management for LXD VMs and containers
//!
//! This module provides the `InstanceName` type which ensures instance names
//! follow the strict naming requirements imposed by LXD for security and
//! compatibility reasons. The validated names are used for:
//!
//! - **LXD Virtual Machines**: Names for provisioned VMs in production/deployment environments
//! - **Testing Containers**: Names for Docker containers used in end-to-end tests
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

/// Errors that can occur during instance name validation
#[derive(Debug, Error, PartialEq)]
pub enum InstanceNameError {
    #[error("Instance name cannot be empty")]
    Empty,

    #[error("Instance name must be 63 characters or less, got {length} characters")]
    TooLong { length: usize },

    #[error("Instance name must not start with a digit or dash")]
    InvalidFirstCharacter,

    #[error("Instance name must not end with a dash")]
    InvalidLastCharacter,

    #[error("Instance name must contain only ASCII letters, numbers, and dashes")]
    InvalidCharacters,
}

/// A validated instance name following LXD naming requirements.
///
/// This type ensures that names are valid for both LXD virtual machines used in
/// production deployments and Docker containers used in end-to-end testing.
///
/// Valid instance names must fulfill the following requirements:
/// - The name must be between 1 and 63 characters long
/// - The name must contain only letters, numbers and dashes from the ASCII table
/// - The name must not start with a digit or a dash
/// - The name must not end with a dash
///
/// These requirements ensure that the instance name can be used in DNS records,
/// on the file system, in various security profiles and as the host name.
///
/// # Use Cases
///
/// - **LXD Virtual Machines**: Naming VMs provisioned for deployment environments
/// - **Testing Containers**: Naming Docker containers in E2E test scenarios
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::domain::InstanceName;
///
/// // Valid instance names - accepts both &str and String
/// let vm_instance = InstanceName::new("torrust-vm-prod")?;
/// let test_container = InstanceName::new("test-container-01".to_string())?;
/// let dynamic_name = InstanceName::new(format!("app-{}", "staging"))?;
///
/// // Invalid instance names
/// assert!(InstanceName::new("").is_err());
/// assert!(InstanceName::new("test-").is_err());
/// assert!(InstanceName::new("-test").is_err());
/// assert!(InstanceName::new("1test").is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct InstanceName(String);

impl InstanceName {
    /// Creates a new `InstanceName` from a string if it's valid.
    ///
    /// This method validates that the provided name meets the requirements for both
    /// LXD virtual machines and testing containers.
    ///
    /// # Arguments
    ///
    /// * `name` - The instance name to validate (accepts `&str`, `String`, or anything that implements `Into<String>`)
    ///
    /// # Returns
    ///
    /// * `Ok(InstanceName)` - If the name is valid
    /// * `Err(InstanceNameError)` - If the name violates LXD naming requirements
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
    /// use torrust_tracker_deploy::domain::InstanceName;
    ///
    /// // Valid names for VMs and containers - accepts both &str and String
    /// let vm_name = InstanceName::new("torrust-vm-prod")?;
    /// let container_name = InstanceName::new("test-web-01".to_string())?;
    /// let dynamic_name = InstanceName::new(format!("app-{}", "staging"))?;
    ///
    /// // Invalid names
    /// assert!(InstanceName::new("").is_err());
    /// assert!(InstanceName::new("test-").is_err());
    /// assert!(InstanceName::new("-test").is_err());
    /// assert!(InstanceName::new("1test").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Result<Self, InstanceNameError> {
        let name = name.into();
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Returns the instance name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates an instance name according to LXD requirements.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the name is valid
    /// * `Err(InstanceNameError)` - If the name violates any requirement
    fn validate(name: &str) -> Result<(), InstanceNameError> {
        // Check length: must be between 1 and 63 characters
        if name.is_empty() {
            return Err(InstanceNameError::Empty);
        }
        if name.len() > 63 {
            return Err(InstanceNameError::TooLong { length: name.len() });
        }

        // Check characters: only ASCII letters, numbers, and dashes
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(InstanceNameError::InvalidCharacters);
        }

        // Check first character: must not be a digit or dash
        if let Some(first_char) = name.chars().next() {
            if first_char.is_ascii_digit() || first_char == '-' {
                return Err(InstanceNameError::InvalidFirstCharacter);
            }
        }

        // Check last character: must not be a dash
        if name.ends_with('-') {
            return Err(InstanceNameError::InvalidLastCharacter);
        }

        Ok(())
    }
}

impl fmt::Display for InstanceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for InstanceName {
    type Err = InstanceNameError;

    fn from_str(s: &str) -> Result<Self, InstanceNameError> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for InstanceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for InstanceName {
    type Error = InstanceNameError;

    fn try_from(value: String) -> Result<Self, InstanceNameError> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_valid_instance_name() {
        let name = InstanceName::new("test-instance").unwrap();
        assert_eq!(name.as_str(), "test-instance");
    }

    #[test]
    fn it_should_accept_string_slice_and_owned_string() {
        // Test with &str
        let name1 = InstanceName::new("test-instance").unwrap();
        assert_eq!(name1.as_str(), "test-instance");

        // Test with String
        let name2 = InstanceName::new("test-instance".to_string()).unwrap();
        assert_eq!(name2.as_str(), "test-instance");

        // Test with String from format!
        let name3 = InstanceName::new(format!("app-{}", "prod")).unwrap();
        assert_eq!(name3.as_str(), "app-prod");

        // Test that both are equal
        assert_eq!(name1, name2);
    }

    #[test]
    fn it_should_create_instance_name_with_numbers() {
        let name = InstanceName::new("test123").unwrap();
        assert_eq!(name.as_str(), "test123");
    }

    #[test]
    fn it_should_create_instance_name_with_dashes() {
        let name = InstanceName::new("test-instance-name").unwrap();
        assert_eq!(name.as_str(), "test-instance-name");
    }

    #[test]
    fn it_should_create_single_character_name() {
        let name = InstanceName::new("a").unwrap();
        assert_eq!(name.as_str(), "a");
    }

    #[test]
    fn it_should_create_63_character_name() {
        let long_name = "a".repeat(63);
        let name = InstanceName::new(long_name.clone()).unwrap();
        assert_eq!(name.as_str(), long_name);
    }

    #[test]
    fn it_should_reject_empty_name() {
        let result = InstanceName::new("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), InstanceNameError::Empty);
    }

    #[test]
    fn it_should_reject_name_longer_than_63_characters() {
        let long_name = "a".repeat(64);
        let result = InstanceName::new(long_name);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstanceNameError::TooLong { length: 64 }
        );
    }

    #[test]
    fn it_should_reject_name_starting_with_digit() {
        let result = InstanceName::new("1test");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstanceNameError::InvalidFirstCharacter
        );
    }

    #[test]
    fn it_should_reject_name_starting_with_dash() {
        let result = InstanceName::new("-test");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstanceNameError::InvalidFirstCharacter
        );
    }

    #[test]
    fn it_should_reject_name_ending_with_dash() {
        let result = InstanceName::new("test-");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), InstanceNameError::InvalidLastCharacter);
    }

    #[test]
    fn it_should_reject_name_with_invalid_characters() {
        let invalid_chars = vec![
            "test@instance",
            "test.instance",
            "test_instance",
            "test instance",
            "test#instance",
            "test$instance",
            "test%instance",
            "test*instance",
            "test+instance",
            "test=instance",
            "test[instance]",
            "test{instance}",
            "test|instance",
            "test\\instance",
            "test:instance",
            "test;instance",
            "test\"instance",
            "test'instance",
            "test<instance>",
            "test,instance",
            "test?instance",
            "test/instance",
        ];

        for invalid_name in invalid_chars {
            let result = InstanceName::new(invalid_name);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), InstanceNameError::InvalidCharacters);
        }
    }

    #[test]
    fn it_should_reject_name_with_unicode_characters() {
        let result = InstanceName::new("tést-instance");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), InstanceNameError::InvalidCharacters);
    }

    #[test]
    fn it_should_display_instance_name() {
        let name = InstanceName::new("test-instance").unwrap();
        assert_eq!(format!("{name}"), "test-instance");
    }

    #[test]
    fn it_should_parse_valid_name_from_string() {
        let name: InstanceName = "test-instance".parse().unwrap();
        assert_eq!(name.as_str(), "test-instance");
    }

    #[test]
    fn it_should_fail_parsing_invalid_name_from_string() {
        let result: Result<InstanceName, _> = "test-".parse();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), InstanceNameError::InvalidLastCharacter);
    }

    #[test]
    fn it_should_implement_as_ref() {
        let name = InstanceName::new("test-instance").unwrap();
        let as_ref: &str = name.as_ref();
        assert_eq!(as_ref, "test-instance");
    }

    #[test]
    fn it_should_be_cloneable_and_comparable() {
        let name1 = InstanceName::new("test-instance").unwrap();
        let name2 = name1.clone();
        assert_eq!(name1, name2);
    }

    #[test]
    fn it_should_be_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let name1 = InstanceName::new("test-instance").unwrap();
        let name2 = InstanceName::new("test-instance").unwrap();
        let name3 = InstanceName::new("other-instance").unwrap();

        set.insert(name1);
        set.insert(name2); // Should not increase size due to equality
        set.insert(name3);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn it_should_serialize_and_deserialize() {
        let name = InstanceName::new("test-instance").unwrap();

        // Serialize
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"test-instance\"");

        // Deserialize
        let deserialized: InstanceName = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, name);
    }

    #[test]
    fn it_should_fail_deserializing_invalid_name() {
        let invalid_json = "\"test-\"";
        let result: Result<InstanceName, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}
