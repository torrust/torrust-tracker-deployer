//! LXD instance name validation and management
//!
//! This module provides the `InstanceName` type which ensures LXD instance names
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

use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use serde::Serialize;

/// A validated LXD instance name following LXD naming requirements.
///
/// Valid instance names must fulfill the following requirements:
/// - The name must be between 1 and 63 characters long
/// - The name must contain only letters, numbers and dashes from the ASCII table
/// - The name must not start with a digit or a dash
/// - The name must not end with a dash
///
/// These requirements ensure that the instance name can be used in DNS records,
/// on the file system, in various security profiles and as the host name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct InstanceName(String);

impl InstanceName {
    /// Creates a new `InstanceName` from a string if it's valid.
    ///
    /// # Arguments
    ///
    /// * `name` - The instance name to validate
    ///
    /// # Returns
    ///
    /// * `Ok(InstanceName)` - If the name is valid
    /// * `Err(anyhow::Error)` - If the name violates LXD naming requirements
    ///
    /// # Errors
    ///
    /// This function will return an error if the name violates any LXD naming requirements:
    /// * Empty name
    /// * Name longer than 63 characters
    /// * Name contains non-ASCII letters, numbers, or dashes
    /// * Name starts with a digit or dash
    /// * Name ends with a dash
    pub fn new(name: String) -> Result<Self> {
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
    /// * `Err(anyhow::Error)` - If the name violates any requirement
    fn validate(name: &str) -> Result<()> {
        // Check length: must be between 1 and 63 characters
        if name.is_empty() {
            return Err(anyhow!("Instance name cannot be empty"));
        }
        if name.len() > 63 {
            return Err(anyhow!(
                "Instance name must be 63 characters or less, got {} characters",
                name.len()
            ));
        }

        // Check characters: only ASCII letters, numbers, and dashes
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(anyhow!(
                "Instance name must contain only ASCII letters, numbers, and dashes"
            ));
        }

        // Check first character: must not be a digit or dash
        if let Some(first_char) = name.chars().next() {
            if first_char.is_ascii_digit() || first_char == '-' {
                return Err(anyhow!("Instance name must not start with a digit or dash"));
            }
        }

        // Check last character: must not be a dash
        if name.ends_with('-') {
            return Err(anyhow!("Instance name must not end with a dash"));
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for InstanceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_valid_instance_name() {
        let name = InstanceName::new("test-instance".to_string()).unwrap();
        assert_eq!(name.as_str(), "test-instance");
    }

    #[test]
    fn it_should_create_instance_name_with_numbers() {
        let name = InstanceName::new("test123".to_string()).unwrap();
        assert_eq!(name.as_str(), "test123");
    }

    #[test]
    fn it_should_create_instance_name_with_dashes() {
        let name = InstanceName::new("test-instance-name".to_string()).unwrap();
        assert_eq!(name.as_str(), "test-instance-name");
    }

    #[test]
    fn it_should_create_single_character_name() {
        let name = InstanceName::new("a".to_string()).unwrap();
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
        let result = InstanceName::new(String::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn it_should_reject_name_longer_than_63_characters() {
        let long_name = "a".repeat(64);
        let result = InstanceName::new(long_name);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("63 characters or less"));
    }

    #[test]
    fn it_should_reject_name_starting_with_digit() {
        let result = InstanceName::new("1test".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not start with a digit or dash"));
    }

    #[test]
    fn it_should_reject_name_starting_with_dash() {
        let result = InstanceName::new("-test".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not start with a digit or dash"));
    }

    #[test]
    fn it_should_reject_name_ending_with_dash() {
        let result = InstanceName::new("test-".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not end with a dash"));
    }

    #[test]
    fn it_should_reject_name_with_invalid_characters() {
        let invalid_chars = vec![
            "test@instance",
            "test.instance",
            "test_instance",
            "test instance",
            "test#instance",
        ];

        for invalid_name in invalid_chars {
            let result = InstanceName::new(invalid_name.to_string());
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("must contain only ASCII letters, numbers, and dashes"));
        }
    }

    #[test]
    fn it_should_reject_name_with_unicode_characters() {
        let result = InstanceName::new("tést-instance".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must contain only ASCII letters, numbers, and dashes"));
    }

    #[test]
    fn it_should_display_instance_name() {
        let name = InstanceName::new("test-instance".to_string()).unwrap();
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
    }

    #[test]
    fn it_should_implement_as_ref() {
        let name = InstanceName::new("test-instance".to_string()).unwrap();
        let as_ref: &str = name.as_ref();
        assert_eq!(as_ref, "test-instance");
    }

    #[test]
    fn it_should_be_cloneable_and_comparable() {
        let name1 = InstanceName::new("test-instance".to_string()).unwrap();
        let name2 = name1.clone();
        assert_eq!(name1, name2);
    }
}
