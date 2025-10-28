//! SSH credentials configuration value object
//!
//! This module provides the `SshCredentialsConfig` type which represents
//! SSH credentials in the configuration layer (distinct from the adapter layer).
//! It handles string-based paths and usernames that will be converted to domain types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::adapters::ssh::SshCredentials;
use crate::shared::Username;

use super::errors::CreateConfigError;

/// Default SSH username for remote connections
const DEFAULT_SSH_USERNAME: &str = "torrust";

/// Default SSH port for remote connections
const DEFAULT_SSH_PORT: u16 = 22;

/// SSH credentials configuration for remote instance authentication
///
/// This is a configuration-layer value object that uses strings for paths
/// and username. It is distinct from `adapters::ssh::SshCredentials` which
/// uses domain types (`PathBuf`, `Username`).
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::SshCredentialsConfig;
///
/// let config = SshCredentialsConfig {
///     private_key_path: "fixtures/testing_rsa".to_string(),
///     public_key_path: "fixtures/testing_rsa.pub".to_string(),
///     username: "torrust".to_string(),
///     port: 22,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshCredentialsConfig {
    /// Path to the SSH private key file (as string in config)
    pub private_key_path: String,

    /// Path to the SSH public key file (as string in config)
    pub public_key_path: String,

    /// SSH username (as string in config)
    ///
    /// Defaults to "torrust" if not specified in configuration.
    #[serde(default = "default_ssh_username")]
    pub username: String,

    /// SSH port for remote connections
    ///
    /// Defaults to 22 (standard SSH port) if not specified in configuration.
    #[serde(default = "default_ssh_port")]
    pub port: u16,
}

impl SshCredentialsConfig {
    /// Creates a new SSH credentials configuration with explicit values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::SshCredentialsConfig;
    ///
    /// let config = SshCredentialsConfig::new(
    ///     "fixtures/testing_rsa".to_string(),
    ///     "fixtures/testing_rsa.pub".to_string(),
    ///     "ubuntu".to_string(),
    ///     2222,
    /// );
    /// ```
    #[must_use]
    pub fn new(
        private_key_path: String,
        public_key_path: String,
        username: String,
        port: u16,
    ) -> Self {
        Self {
            private_key_path,
            public_key_path,
            username,
            port,
        }
    }

    /// Converts configuration to domain SSH credentials
    ///
    /// This method validates the configuration and converts string-based
    /// configuration values to strongly-typed domain objects.
    ///
    /// # Validation
    ///
    /// - Username must follow Linux username requirements
    /// - Private key file must exist and be accessible
    /// - Public key file must exist and be accessible
    /// - Port must be valid (validated by caller, typically 1-65535)
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if:
    /// - Username is invalid (see `Username` validation rules)
    /// - Private key file does not exist
    /// - Public key file does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::SshCredentialsConfig;
    ///
    /// let config = SshCredentialsConfig::new(
    ///     "fixtures/testing_rsa".to_string(),
    ///     "fixtures/testing_rsa.pub".to_string(),
    ///     "torrust".to_string(),
    ///     22,
    /// );
    ///
    /// let credentials = config.to_ssh_credentials()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_ssh_credentials(self) -> Result<SshCredentials, CreateConfigError> {
        // Convert string username to domain Username type
        let username = Username::new(&self.username)?;

        // Convert string paths to PathBuf
        let private_key_path = PathBuf::from(&self.private_key_path);
        let public_key_path = PathBuf::from(&self.public_key_path);

        // Validate SSH key files exist
        if !private_key_path.exists() {
            return Err(CreateConfigError::PrivateKeyNotFound {
                path: private_key_path,
            });
        }

        if !public_key_path.exists() {
            return Err(CreateConfigError::PublicKeyNotFound {
                path: public_key_path,
            });
        }

        // Create domain credentials object
        Ok(SshCredentials::new(
            private_key_path,
            public_key_path,
            username,
        ))
    }
}

/// Default SSH username for serde deserialization
fn default_ssh_username() -> String {
    DEFAULT_SSH_USERNAME.to_string()
}

/// Default SSH port for serde deserialization
fn default_ssh_port() -> u16 {
    DEFAULT_SSH_PORT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ssh_credentials_config() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        assert_eq!(config.private_key_path, "fixtures/testing_rsa");
        assert_eq!(config.public_key_path, "fixtures/testing_rsa.pub");
        assert_eq!(config.username, "torrust");
        assert_eq!(config.port, 22);
    }

    #[test]
    fn test_deserialize_with_defaults() {
        let json = r#"{
            "private_key_path": "fixtures/testing_rsa",
            "public_key_path": "fixtures/testing_rsa.pub"
        }"#;

        let config: SshCredentialsConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.private_key_path, "fixtures/testing_rsa");
        assert_eq!(config.public_key_path, "fixtures/testing_rsa.pub");
        assert_eq!(config.username, "torrust"); // default
        assert_eq!(config.port, 22); // default
    }

    #[test]
    fn test_deserialize_with_explicit_values() {
        let json = r#"{
            "private_key_path": "path/to/key",
            "public_key_path": "path/to/key.pub",
            "username": "ubuntu",
            "port": 2222
        }"#;

        let config: SshCredentialsConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.private_key_path, "path/to/key");
        assert_eq!(config.public_key_path, "path/to/key.pub");
        assert_eq!(config.username, "ubuntu");
        assert_eq!(config.port, 2222);
    }

    #[test]
    fn test_serialize_ssh_credentials_config() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "deploy".to_string(),
            2222,
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SshCredentialsConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_convert_to_ssh_credentials_success() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result = config.to_ssh_credentials();
        assert!(result.is_ok(), "Expected successful conversion");

        let credentials = result.unwrap();
        assert_eq!(
            credentials.ssh_priv_key_path,
            PathBuf::from("fixtures/testing_rsa")
        );
        assert_eq!(
            credentials.ssh_pub_key_path,
            PathBuf::from("fixtures/testing_rsa.pub")
        );
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
    }

    #[test]
    fn test_convert_to_ssh_credentials_invalid_username() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "123invalid".to_string(), // starts with number - invalid
            22,
        );

        let result = config.to_ssh_credentials();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidUsername(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidUsername error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_ssh_credentials_private_key_not_found() {
        let config = SshCredentialsConfig::new(
            "/nonexistent/private_key".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result = config.to_ssh_credentials();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PrivateKeyNotFound { path } => {
                assert_eq!(path, PathBuf::from("/nonexistent/private_key"));
            }
            other => panic!("Expected PrivateKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_ssh_credentials_public_key_not_found() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "/nonexistent/public_key.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result = config.to_ssh_credentials();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PublicKeyNotFound { path } => {
                assert_eq!(path, PathBuf::from("/nonexistent/public_key.pub"));
            }
            other => panic!("Expected PublicKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_ssh_username(), "torrust");
        assert_eq!(default_ssh_port(), 22);
    }
}
