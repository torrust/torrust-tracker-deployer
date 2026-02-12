//! SSH credentials configuration value object
//!
//! This module provides the `SshCredentialsConfig` type which represents
//! SSH credentials in the configuration layer (distinct from the adapter layer).
//! It handles string-based paths and usernames that will be converted to domain types.
//!
//! # Conversion Pattern
//!
//! Uses `TryFrom` for idiomatic Rust conversion from DTO to domain type.
//! See ADR: `docs/decisions/tryfrom-for-dto-to-domain-conversion.md`

use std::convert::TryFrom;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
/// ```no_run
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::SshCredentialsConfig;
///
/// let config = SshCredentialsConfig {
///     private_key_path: "fixtures/testing_rsa".to_string(),
///     public_key_path: "fixtures/testing_rsa.pub".to_string(),
///     username: "torrust".to_string(),
///     port: 22,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    /// ```no_run
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
}

impl TryFrom<SshCredentialsConfig> for SshCredentials {
    type Error = CreateConfigError;

    fn try_from(config: SshCredentialsConfig) -> Result<Self, Self::Error> {
        // Convert string username to domain Username type
        let username = Username::new(&config.username)?;

        // Convert string paths to PathBuf
        let private_key_path = PathBuf::from(&config.private_key_path);
        let public_key_path = PathBuf::from(&config.public_key_path);

        // Validate paths are absolute
        if !private_key_path.is_absolute() {
            return Err(CreateConfigError::RelativePrivateKeyPath {
                path: private_key_path,
            });
        }

        if !public_key_path.is_absolute() {
            return Err(CreateConfigError::RelativePublicKeyPath {
                path: public_key_path,
            });
        }

        // Note: File existence is NOT validated here.
        // SSH keys are external resources that may not exist at config parsing time.
        // They will be validated at runtime when SSH connections are actually attempted.
        // This allows configs to be validated and stored even if keys are on different machines.

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
    fn it_should_create_ssh_credentials_config_when_provided_valid_parameters() {
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
    fn it_should_use_default_values_when_deserializing_partial_config() {
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
    fn it_should_use_explicit_values_when_provided_in_config() {
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
    fn it_should_serialize_to_json_when_converting_ssh_credentials_config() {
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
    fn it_should_convert_to_ssh_credentials_when_config_is_valid() {
        use std::env;

        // Get absolute paths to existing test fixtures
        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = SshCredentialsConfig::new(
            private_key_path.clone(),
            public_key_path.clone(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, _> = config.try_into();
        assert!(result.is_ok(), "Expected successful conversion");

        let credentials = result.unwrap();
        assert_eq!(
            credentials.ssh_priv_key_path,
            PathBuf::from(&private_key_path)
        );
        assert_eq!(
            credentials.ssh_pub_key_path,
            PathBuf::from(&public_key_path)
        );
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
    }

    #[test]
    fn it_should_return_error_when_username_is_invalid() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = SshCredentialsConfig::new(
            private_key_path,
            public_key_path,
            "123invalid".to_string(), // starts with number - invalid
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidUsername(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidUsername error, got: {other:?}"),
        }
    }

    // Note: Tests for file existence removed - file existence is now validated
    // at runtime when SSH connections are attempted, not during config parsing.

    #[test]
    fn it_should_provide_correct_default_values_when_using_default_functions() {
        assert_eq!(default_ssh_username(), "torrust");
        assert_eq!(default_ssh_port(), 22);
    }

    #[test]
    fn it_should_reject_config_when_private_key_path_is_relative() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "/absolute/path/to/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::RelativePrivateKeyPath { path } => {
                assert_eq!(path, PathBuf::from("fixtures/testing_rsa"));
                assert!(!path.is_absolute(), "Path should be relative");
            }
            other => panic!("Expected RelativePrivateKeyPath error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_reject_config_when_public_key_path_is_relative() {
        let config = SshCredentialsConfig::new(
            "/absolute/path/to/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::RelativePublicKeyPath { path } => {
                assert_eq!(path, PathBuf::from("fixtures/testing_rsa.pub"));
                assert!(!path.is_absolute(), "Path should be relative");
            }
            other => panic!("Expected RelativePublicKeyPath error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_reject_config_when_both_key_paths_are_relative() {
        let config = SshCredentialsConfig::new(
            "fixtures/testing_rsa".to_string(),
            "fixtures/testing_rsa.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(result.is_err());

        // Should fail on private key first (checked first in validation)
        match result.unwrap_err() {
            CreateConfigError::RelativePrivateKeyPath { .. } => {
                // Expected - private key is checked first
            }
            other => panic!("Expected RelativePrivateKeyPath error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_accept_config_when_ssh_key_paths_are_absolute() {
        use std::env;

        // Get absolute paths to existing test fixtures
        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Verify test fixtures exist
        assert!(
            PathBuf::from(&private_key_path).exists(),
            "Test fixture not found: {private_key_path}"
        );
        assert!(
            PathBuf::from(&public_key_path).exists(),
            "Test fixture not found: {public_key_path}"
        );

        let config = SshCredentialsConfig::new(
            private_key_path.clone(),
            public_key_path.clone(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(
            result.is_ok(),
            "Expected successful conversion with absolute paths"
        );

        let credentials = result.unwrap();
        assert_eq!(
            credentials.ssh_priv_key_path,
            PathBuf::from(&private_key_path)
        );
        assert_eq!(
            credentials.ssh_pub_key_path,
            PathBuf::from(&public_key_path)
        );
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
    }

    #[test]
    fn it_should_return_clear_error_message_when_relative_path_detected() {
        let config = SshCredentialsConfig::new(
            "relative/path/key".to_string(),
            "/absolute/path/key.pub".to_string(),
            "torrust".to_string(),
            22,
        );

        let result: Result<SshCredentials, CreateConfigError> = config.try_into();
        assert!(result.is_err());

        let error = result.unwrap_err();
        let help_text = error.help();

        // Verify help text contains key information
        assert!(
            help_text.contains("absolute"),
            "Help text should mention absolute paths"
        );
        assert!(
            help_text.contains("realpath"),
            "Help text should mention realpath command"
        );
        assert!(
            help_text.contains("working directories"),
            "Help text should explain why absolute paths are needed"
        );
    }
}
