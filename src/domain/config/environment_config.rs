//! Environment creation configuration value object
//!
//! This module provides the `EnvironmentCreationConfig` type which represents
//! all configuration needed to create a deployment environment. It handles
//! deserialization from configuration sources and conversion to domain types.

use serde::{Deserialize, Serialize};

use crate::adapters::ssh::SshCredentials;
use crate::domain::EnvironmentName;

use super::errors::CreateConfigError;
use super::ssh_credentials_config::SshCredentialsConfig;

/// Configuration for creating a deployment environment
///
/// This is the top-level configuration object that contains all information
/// needed to create a new deployment environment. It deserializes from JSON
/// configuration and provides type-safe conversion to domain parameters.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::config::{EnvironmentCreationConfig, EnvironmentSection};
///
/// let json = r#"{
///     "environment": {
///         "name": "dev"
///     },
///     "ssh_credentials": {
///         "private_key_path": "fixtures/testing_rsa",
///         "public_key_path": "fixtures/testing_rsa.pub"
///     }
/// }"#;
///
/// let config: EnvironmentCreationConfig = serde_json::from_str(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    /// Environment-specific settings
    pub environment: EnvironmentSection,

    /// SSH credentials configuration
    pub ssh_credentials: SshCredentialsConfig,
}

/// Environment-specific configuration section
///
/// Contains configuration specific to the environment being created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSection {
    /// Name of the environment to create
    ///
    /// Must follow environment naming rules:
    /// - Lowercase letters and numbers only
    /// - Dashes as word separators
    /// - Cannot start or end with separators
    /// - Cannot start with numbers
    pub name: String,
}

impl EnvironmentCreationConfig {
    /// Creates a new environment creation configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig
    /// };
    ///
    /// let config = EnvironmentCreationConfig::new(
    ///     EnvironmentSection {
    ///         name: "dev".to_string(),
    ///     },
    ///     SshCredentialsConfig::new(
    ///         "fixtures/testing_rsa".to_string(),
    ///         "fixtures/testing_rsa.pub".to_string(),
    ///         "torrust".to_string(),
    ///         22,
    ///     ),
    /// );
    /// ```
    #[must_use]
    pub fn new(environment: EnvironmentSection, ssh_credentials: SshCredentialsConfig) -> Self {
        Self {
            environment,
            ssh_credentials,
        }
    }

    /// Converts configuration to domain parameters for `Environment::new()`
    ///
    /// This method validates all configuration values and converts them to
    /// strongly-typed domain objects that can be used to create an Environment.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(EnvironmentName, SshCredentials, u16)` that matches
    /// the signature of `Environment::new()`.
    ///
    /// # Validation
    ///
    /// - Environment name must follow naming rules (see `EnvironmentName`)
    /// - SSH username must follow Linux username requirements (see `Username`)
    /// - SSH key files must exist and be accessible
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if:
    /// - Environment name is invalid
    /// - SSH username is invalid
    /// - SSH private key file does not exist
    /// - SSH public key file does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig
    /// };
    /// use torrust_tracker_deployer_lib::domain::Environment;
    ///
    /// let config = EnvironmentCreationConfig::new(
    ///     EnvironmentSection {
    ///         name: "dev".to_string(),
    ///     },
    ///     SshCredentialsConfig::new(
    ///         "fixtures/testing_rsa".to_string(),
    ///         "fixtures/testing_rsa.pub".to_string(),
    ///         "torrust".to_string(),
    ///         22,
    ///     ),
    /// );
    ///
    /// let (name, credentials, port) = config.to_environment_params()?;
    /// let environment = Environment::new(name, credentials, port);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_environment_params(
        self,
    ) -> Result<(EnvironmentName, SshCredentials, u16), CreateConfigError> {
        // Convert environment name string to domain type
        let environment_name = EnvironmentName::new(&self.environment.name)?;

        // Get SSH port before consuming ssh_credentials
        let ssh_port = self.ssh_credentials.port;

        // Convert SSH credentials config to domain type
        let ssh_credentials = self.ssh_credentials.to_ssh_credentials()?;

        Ok((environment_name, ssh_credentials, ssh_port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_environment_creation_config() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        assert_eq!(config.environment.name, "dev");
        assert_eq!(
            config.ssh_credentials.private_key_path,
            "fixtures/testing_rsa"
        );
        assert_eq!(config.ssh_credentials.username, "torrust");
        assert_eq!(config.ssh_credentials.port, 22);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{
            "environment": {
                "name": "e2e-config"
            },
            "ssh_credentials": {
                "private_key_path": "fixtures/testing_rsa",
                "public_key_path": "fixtures/testing_rsa.pub"
            }
        }"#;

        let config: EnvironmentCreationConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.environment.name, "e2e-config");
        assert_eq!(
            config.ssh_credentials.private_key_path,
            "fixtures/testing_rsa"
        );
        assert_eq!(
            config.ssh_credentials.public_key_path,
            "fixtures/testing_rsa.pub"
        );
        assert_eq!(config.ssh_credentials.username, "torrust"); // default
        assert_eq!(config.ssh_credentials.port, 22); // default
    }

    #[test]
    fn test_deserialize_from_json_with_custom_values() {
        let json = r#"{
            "environment": {
                "name": "production"
            },
            "ssh_credentials": {
                "private_key_path": "keys/prod_key",
                "public_key_path": "keys/prod_key.pub",
                "username": "ubuntu",
                "port": 2222
            }
        }"#;

        let config: EnvironmentCreationConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.environment.name, "production");
        assert_eq!(config.ssh_credentials.private_key_path, "keys/prod_key");
        assert_eq!(config.ssh_credentials.public_key_path, "keys/prod_key.pub");
        assert_eq!(config.ssh_credentials.username, "ubuntu");
        assert_eq!(config.ssh_credentials.port, 2222);
    }

    #[test]
    fn test_serialize_environment_creation_config() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "staging".to_string(),
            },
            SshCredentialsConfig::new(
                "keys/stage_key".to_string(),
                "keys/stage_key.pub".to_string(),
                "deploy".to_string(),
                22,
            ),
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_convert_to_environment_params_success() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let result = config.to_environment_params();
        assert!(result.is_ok(), "Expected successful conversion");

        let (name, credentials, port) = result.unwrap();

        assert_eq!(name.as_str(), "dev");
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
        assert_eq!(port, 22);
    }

    #[test]
    fn test_convert_to_environment_params_invalid_environment_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "Invalid_Name".to_string(), // uppercase - invalid
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidEnvironmentName(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidEnvironmentName error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_environment_params_invalid_username() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "123invalid".to_string(), // starts with number - invalid
                22,
            ),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidUsername(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidUsername error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_environment_params_private_key_not_found() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "/nonexistent/key".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PrivateKeyNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected PrivateKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_environment_params_public_key_not_found() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "/nonexistent/key.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PublicKeyNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected PublicKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn test_integration_with_environment_new() {
        // This test verifies that the converted parameters work with Environment::new()
        use crate::domain::Environment;

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "test-env".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let (name, credentials, port) = config.to_environment_params().unwrap();
        let environment = Environment::new(name, credentials, port);

        assert_eq!(environment.name().as_str(), "test-env");
        assert_eq!(environment.ssh_username().as_str(), "torrust");
        assert_eq!(environment.ssh_port(), 22);
    }

    #[test]
    fn test_round_trip_serialization() {
        let original = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
        );

        let json = serde_json::to_string_pretty(&original).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
