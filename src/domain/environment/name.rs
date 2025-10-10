//! Environment Name Domain Entity
//!
//! This module defines the `EnvironmentName` domain entity which represents a validated
//! environment identifier. Environment names must follow a restricted format to ensure
//! compatibility across different systems and avoid conflicts.
//!
//! ## Format Rules
//!
//! Environment names must:
//! - Contain only lowercase letters (a-z) and numbers (0-9)
//! - Use dashes (-) as word separators (compatible with `InstanceName` requirements)
//! - Be non-empty strings
//! - Not start or end with separators
//! - Not start with numbers (for consistency with `InstanceName`)
//!
//! ## Valid Examples
//!
//! - `dev`
//! - `staging`
//! - `production`
//! - `e2e-config`
//! - `e2e-provision`
//! - `e2e-full`
//! - `test-integration`
//!
//! ## Error Handling
//!
//! Following the project's error handling principles, validation errors provide:
//! - Clear explanation of what went wrong
//! - Specific format requirements
//! - Examples of valid environment names
//! - Actionable guidance for fixing the issue

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use thiserror::Error;

/// Validated environment name following restricted format rules
///
/// Environment names are used to isolate different deployment environments
/// and must follow specific naming conventions for consistency and compatibility.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::EnvironmentName;
///
/// // Valid environment names
/// let dev = EnvironmentName::new("dev".to_string())?;
/// let staging = EnvironmentName::new("staging".to_string())?;
/// let e2e_config = EnvironmentName::new("e2e-config".to_string())?;
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnvironmentName(String);

impl EnvironmentName {
    /// Creates a new `EnvironmentName` from a string with validation.
    ///
    /// # Arguments
    ///
    /// * `name` - Environment name that can be converted to a string
    ///
    /// # Validation Rules
    ///
    /// 1. **Non-empty**: Must contain at least one character
    /// 2. **Lowercase only**: Only lowercase ASCII letters allowed
    /// 3. **Numbers allowed**: ASCII digits 0-9 are permitted
    /// 4. **Dashes only**: Only dashes (-) allowed as separators (no underscores or slashes)
    /// 5. **No leading/trailing separators**: Cannot start or end with dashes
    /// 6. **No consecutive separators**: Cannot contain multiple consecutive dashes
    /// 7. **No leading numbers**: Cannot start with a digit
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::domain::EnvironmentName;
    /// // Valid names - accepts various string types
    /// assert!(EnvironmentName::new("dev").is_ok());
    /// assert!(EnvironmentName::new("e2e-config".to_string()).is_ok());
    /// assert!(EnvironmentName::new(String::from("staging123")).is_ok());
    ///
    /// // Invalid names
    /// assert!(EnvironmentName::new("").is_err());              // Empty
    /// assert!(EnvironmentName::new("Dev").is_err());           // Uppercase
    /// assert!(EnvironmentName::new("dev_test").is_err());      // Underscore
    /// assert!(EnvironmentName::new("123prod").is_err());       // Starts with number
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `EnvironmentNameError` with specific validation failure details
    /// and actionable guidance for fixing the name format.
    pub fn new<S: Into<String>>(name: S) -> Result<Self, EnvironmentNameError> {
        let name = name.into();
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Returns the environment name as a string slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::EnvironmentName;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// assert_eq!(env_name.as_str(), "production");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates an environment name according to the format rules
    ///
    /// This is used internally by `new()` but is also public for testing purposes.
    fn validate(name: &str) -> Result<(), EnvironmentNameError> {
        // Check if empty
        if name.is_empty() {
            return Err(EnvironmentNameError::Empty);
        }

        // Check if starts with number (for consistency with InstanceName)
        if name.chars().next().unwrap().is_ascii_digit() {
            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: "starts with a number (for InstanceName compatibility)".to_string(),
                valid_examples: Self::valid_examples(),
            });
        }

        // Check for invalid characters and collect issues
        let mut has_uppercase = false;
        let mut invalid_chars = Vec::new();

        for ch in name.chars() {
            match ch {
                'A'..='Z' => {
                    has_uppercase = true;
                }
                'a'..='z' | '0'..='9' | '-' => {
                    // Valid characters (lowercase letters, numbers, dashes)
                }
                _ => {
                    invalid_chars.push(ch);
                }
            }
        }

        // Report uppercase letters specifically
        if has_uppercase {
            let uppercase_chars: Vec<char> =
                name.chars().filter(char::is_ascii_uppercase).collect();

            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: format!(
                    "contains uppercase letters: {}",
                    uppercase_chars.iter().collect::<String>()
                ),
                valid_examples: Self::valid_examples(),
            });
        }

        // Report other invalid characters
        if !invalid_chars.is_empty() {
            let unique_invalid: Vec<char> = {
                let mut chars = invalid_chars;
                chars.sort_unstable();
                chars.dedup();
                chars
            };

            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: format!(
                    "contains invalid characters: {}",
                    unique_invalid.iter().collect::<String>()
                ),
                valid_examples: Self::valid_examples(),
            });
        }

        // Check for leading/trailing separators
        if name.starts_with('-') {
            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: "starts with dash".to_string(),
                valid_examples: Self::valid_examples(),
            });
        }

        if name.ends_with('-') {
            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: "ends with dash".to_string(),
                valid_examples: Self::valid_examples(),
            });
        }

        // Check for consecutive separators
        if name.contains("--") {
            return Err(EnvironmentNameError::InvalidFormat {
                attempted_name: name.to_string(),
                reason: "contains consecutive dashes".to_string(),
                valid_examples: Self::valid_examples(),
            });
        }

        Ok(())
    }

    /// Returns a list of valid environment name examples
    fn valid_examples() -> Vec<String> {
        vec![
            "dev".to_string(),
            "staging".to_string(),
            "production".to_string(),
            "e2e-config".to_string(),
            "e2e-provision".to_string(),
            "e2e-full".to_string(),
            "test-integration".to_string(),
            "release-v1-2".to_string(),
        ]
    }
}

impl Display for EnvironmentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for EnvironmentName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Errors that can occur when creating or validating environment names
///
/// These errors follow the project's error handling principles by providing
/// clear, contextual, and actionable error messages.
#[derive(Debug, Error)]
pub enum EnvironmentNameError {
    /// Environment name is empty
    #[error("Environment name cannot be empty.\n\nValid format: lowercase letters, numbers, and dashes only\nExamples: {}", 
        EnvironmentName::valid_examples().join(", "))]
    Empty,

    /// Environment name contains invalid characters or format
    #[error("Environment name '{attempted_name}' is invalid: {reason}.\n\nValid format: lowercase letters, numbers, and dashes only\nExamples: {}", 
        valid_examples.join(", "))]
    InvalidFormat {
        /// The name that was attempted to be created
        attempted_name: String,
        /// Specific reason why the name is invalid
        reason: String,
        /// List of valid example names
        valid_examples: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_valid_environment_names() {
        let valid_names = vec![
            "dev",
            "staging",
            "production",
            "e2e-config",
            "e2e-provision",
            "e2e-full",
            "test-integration",
            "feature-user-auth",
            "release-v1-0",
            "api2", // numbers are allowed, just not at the start
        ];

        for name in valid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_ok(), "Expected '{name}' to be valid");

            let env_name = result.unwrap();
            assert_eq!(env_name.as_str(), name);
            assert_eq!(env_name.to_string(), name);
        }
    }

    #[test]
    fn it_should_reject_empty_names() {
        let result = EnvironmentName::new(String::new());
        assert!(result.is_err());

        match result.unwrap_err() {
            EnvironmentNameError::Empty => {
                // Expected error type
            }
            other @ EnvironmentNameError::InvalidFormat { .. } => {
                panic!("Expected Empty error, got: {other:?}")
            }
        }
    }

    #[test]
    fn it_should_reject_uppercase_letters() {
        let invalid_names = vec!["Dev", "PRODUCTION", "e2e_Config", "Test"];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("uppercase"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn it_should_reject_invalid_characters() {
        let invalid_names = vec![
            "e2e_config", // underscore
            "test.env",   // dot
            "env@prod",   // at symbol
            "test env",   // space
            "env#1",      // hash
            "test/env",   // slash
        ];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("invalid characters"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn it_should_reject_names_starting_with_separators() {
        let invalid_names = vec!["-dev", "-test"];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("starts with dash"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn it_should_reject_names_ending_with_separators() {
        let invalid_names = vec!["dev-", "prod-", "test-"];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("ends with dash"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn it_should_reject_consecutive_separators() {
        let invalid_names = vec!["test--env", "dev--prod"];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("consecutive dashes"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn it_should_be_serializable_to_json() {
        let env_name = EnvironmentName::new("e2e-config".to_string()).unwrap();

        let json = serde_json::to_string(&env_name).unwrap();
        assert_eq!(json, "\"e2e-config\"");

        let deserialized: EnvironmentName = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, env_name);
    }

    #[test]
    fn it_should_implement_display_trait() {
        let env_name = EnvironmentName::new("production".to_string()).unwrap();
        assert_eq!(format!("{env_name}"), "production");
    }

    #[test]
    fn it_should_implement_as_ref_trait() {
        let env_name = EnvironmentName::new("staging".to_string()).unwrap();
        let as_ref: &str = env_name.as_ref();
        assert_eq!(as_ref, "staging");
    }

    #[test]
    fn it_should_provide_helpful_error_messages() {
        let result = EnvironmentName::new("Invalid_Name".to_string());
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid_Name"));
        assert!(
            error_message.contains("invalid characters")
                || error_message.contains("uppercase letters")
        );
        assert!(error_message.contains("lowercase letters, numbers, and dashes only"));
        assert!(error_message.contains("dev")); // Should contain examples
    }

    #[test]
    fn it_should_reject_names_starting_with_numbers() {
        let invalid_names = vec!["1dev", "2test", "3env"];

        for name in invalid_names {
            let result = EnvironmentName::new(name.to_string());
            assert!(result.is_err(), "Expected '{name}' to be invalid");

            match result.unwrap_err() {
                EnvironmentNameError::InvalidFormat {
                    attempted_name,
                    reason,
                    ..
                } => {
                    assert_eq!(attempted_name, name);
                    assert!(reason.contains("starts with a number"));
                }
                other @ EnvironmentNameError::Empty => {
                    panic!("Expected InvalidFormat error, got: {other:?}")
                }
            }
        }
    }
}
