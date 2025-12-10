//! Environment creation configuration value object
//!
//! This module provides the `EnvironmentCreationConfig` type which represents
//! all configuration needed to create a deployment environment. It handles
//! deserialization from configuration sources and conversion to domain types.

use serde::{Deserialize, Serialize};

use crate::adapters::ssh::SshCredentials;
use crate::domain::provider::{Provider, ProviderConfig};
use crate::domain::tracker::{
    DatabaseConfig, HttpApiConfig, HttpTrackerConfig, TrackerConfig, TrackerCoreConfig,
    UdpTrackerConfig,
};
use crate::domain::{EnvironmentName, InstanceName};

use super::errors::CreateConfigError;
use super::provider::{HetznerProviderSection, LxdProviderSection, ProviderSection};
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
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
///     EnvironmentCreationConfig, EnvironmentSection, ProviderSection, LxdProviderSection
/// };
///
/// let json = r#"{
///     "environment": {
///         "name": "dev"
///     },
///     "ssh_credentials": {
///         "private_key_path": "fixtures/testing_rsa",
///         "public_key_path": "fixtures/testing_rsa.pub"
///     },
///     "provider": {
///         "provider": "lxd",
///         "profile_name": "torrust-profile-dev"
///     },
///     "tracker": {
///         "core": {
///             "database": {
///                 "driver": "sqlite3",
///                 "database_name": "tracker.db"
///             },
///             "private": false
///         },
///         "udp_trackers": [
///             {
///                 "bind_address": "0.0.0.0:6969"
///             }
///         ],
///         "http_trackers": [
///             {
///                 "bind_address": "0.0.0.0:7070"
///             }
///         ],
///         "http_api": {
///             "admin_token": "MyAccessToken"
///         }
///     }
/// }"#;
///
/// let config: EnvironmentCreationConfig = serde_json::from_str(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    /// Environment-specific settings
    pub environment: EnvironmentSection,

    /// SSH credentials configuration
    pub ssh_credentials: SshCredentialsConfig,

    /// Provider-specific configuration (LXD, Hetzner, etc.)
    ///
    /// Uses `ProviderSection` for JSON parsing with raw primitives.
    /// Converted to domain `ProviderConfig` via `to_environment_params()`.
    pub provider: ProviderSection,

    /// Tracker deployment configuration
    pub tracker: TrackerConfig,
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

    /// Optional custom instance name for the VM/container
    ///
    /// If not provided, auto-generated as `torrust-tracker-vm-{env_name}`.
    /// When provided, must follow instance naming rules:
    /// - 1-63 characters
    /// - ASCII letters, numbers, and dashes only
    /// - Cannot start with digit or dash
    /// - Cannot end with dash
    #[serde(default)]
    pub instance_name: Option<String>,
}

impl EnvironmentCreationConfig {
    /// Creates a new environment creation configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    ///     ProviderSection, LxdProviderSection
    /// };
    /// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
    ///
    /// let config = EnvironmentCreationConfig::new(
    ///     EnvironmentSection {
    ///         name: "dev".to_string(),
    ///         instance_name: None,
    ///     },
    ///     SshCredentialsConfig::new(
    ///         "fixtures/testing_rsa".to_string(),
    ///         "fixtures/testing_rsa.pub".to_string(),
    ///         "torrust".to_string(),
    ///         22,
    ///     ),
    ///     ProviderSection::Lxd(LxdProviderSection {
    ///         profile_name: "torrust-profile-dev".to_string(),
    ///     }),
    ///     TrackerConfig::default(),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        environment: EnvironmentSection,
        ssh_credentials: SshCredentialsConfig,
        provider: ProviderSection,
        tracker: TrackerConfig,
    ) -> Self {
        Self {
            environment,
            ssh_credentials,
            provider,
            tracker,
        }
    }

    /// Converts configuration to domain parameters for `Environment::new()`
    ///
    /// This method validates all configuration values and converts them to
    /// strongly-typed domain objects that can be used to create an Environment.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(EnvironmentName, InstanceName, ProviderConfig, SshCredentials, u16)`
    /// that matches the signature of `Environment::new()`.
    ///
    /// # Validation
    ///
    /// - Environment name must follow naming rules (see `EnvironmentName`)
    /// - Instance name (if provided) must follow instance naming rules (see `InstanceName`)
    /// - Provider config must be valid (e.g., valid profile name for LXD)
    /// - SSH username must follow Linux username requirements (see `Username`)
    /// - SSH key files must exist and be accessible
    ///
    /// # Instance Name Auto-Generation
    ///
    /// If `instance_name` is not provided in the configuration, it will be
    /// auto-generated using the format: `torrust-tracker-vm-{env_name}`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if:
    /// - Environment name is invalid
    /// - Instance name is invalid (when provided)
    /// - Provider configuration is invalid (e.g., invalid profile name)
    /// - SSH username is invalid
    /// - SSH private key file does not exist
    /// - SSH public key file does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    ///     ProviderSection, LxdProviderSection
    /// };
    /// use torrust_tracker_deployer_lib::domain::Environment;
    /// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
    ///
    /// let config = EnvironmentCreationConfig::new(
    ///     EnvironmentSection {
    ///         name: "dev".to_string(),
    ///         instance_name: None,
    ///     },
    ///     SshCredentialsConfig::new(
    ///         "fixtures/testing_rsa".to_string(),
    ///         "fixtures/testing_rsa.pub".to_string(),
    ///         "torrust".to_string(),
    ///         22,
    ///     ),
    ///     ProviderSection::Lxd(LxdProviderSection {
    ///         profile_name: "torrust-profile-dev".to_string(),
    ///     }),
    ///     TrackerConfig::default(),
    /// );
    ///
    /// let (name, instance_name, provider_config, credentials, port, tracker) = config.to_environment_params()?;
    ///
    /// // Instance name auto-generated from environment name
    /// assert_eq!(instance_name.as_str(), "torrust-tracker-vm-dev");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_environment_params(
        self,
    ) -> Result<
        (
            EnvironmentName,
            InstanceName,
            ProviderConfig,
            SshCredentials,
            u16,
            TrackerConfig,
        ),
        CreateConfigError,
    > {
        // Convert environment name string to domain type
        let environment_name = EnvironmentName::new(&self.environment.name)?;

        // Instance name: use provided or auto-generate from environment name
        let instance_name = match &self.environment.instance_name {
            Some(name_str) => InstanceName::new(name_str.clone()).map_err(|e| {
                CreateConfigError::InvalidInstanceName {
                    name: name_str.clone(),
                    reason: e.to_string(),
                }
            })?,
            None => Self::generate_instance_name(&environment_name),
        };

        // Convert ProviderSection (DTO) to domain ProviderConfig (validates profile_name, etc.)
        let provider_config = self.provider.to_provider_config()?;

        // Get SSH port before consuming ssh_credentials
        let ssh_port = self.ssh_credentials.port;

        // Convert SSH credentials config to domain type
        let ssh_credentials = self.ssh_credentials.to_ssh_credentials()?;

        // Get tracker config
        let tracker_config = self.tracker;

        Ok((
            environment_name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker_config,
        ))
    }

    /// Generates an instance name from the environment name
    ///
    /// Format: `torrust-tracker-vm-{env_name}`
    ///
    /// # Panics
    ///
    /// This function does not panic. The generated instance name is guaranteed
    /// to be valid for any valid environment name.
    fn generate_instance_name(env_name: &EnvironmentName) -> InstanceName {
        let instance_name_str = format!("torrust-tracker-vm-{}", env_name.as_str());
        InstanceName::new(instance_name_str)
            .expect("Generated instance name should always be valid for valid environment names")
    }

    /// Creates a template instance with placeholder values for a specific provider
    ///
    /// This method generates a configuration template with placeholder values
    /// that users can replace with their actual configuration. The template
    /// structure matches the `EnvironmentCreationConfig` exactly, ensuring
    /// type safety and automatic synchronization with struct changes.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider to generate template for (lxd or hetzner)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
    /// use torrust_tracker_deployer_lib::domain::provider::Provider;
    ///
    /// let template = EnvironmentCreationConfig::template(Provider::Lxd);
    /// assert_eq!(template.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
    /// ```
    #[must_use]
    pub fn template(provider: Provider) -> Self {
        let provider_section = match provider {
            Provider::Lxd => ProviderSection::Lxd(LxdProviderSection {
                profile_name: "REPLACE_WITH_LXD_PROFILE_NAME".to_string(),
            }),
            Provider::Hetzner => ProviderSection::Hetzner(HetznerProviderSection {
                api_token: "REPLACE_WITH_HETZNER_API_TOKEN".to_string(),
                server_type: "cx22".to_string(), // default value - small instance
                location: "nbg1".to_string(),    // default value - Nuremberg
                image: "ubuntu-24.04".to_string(), // default value - Ubuntu 24.04 LTS
            }),
        };

        Self {
            environment: EnvironmentSection {
                name: "REPLACE_WITH_ENVIRONMENT_NAME".to_string(),
                instance_name: None, // Auto-generated if not provided
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH".to_string(),
                public_key_path: "REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH".to_string(),
                username: "torrust".to_string(), // default value
                port: 22,                        // default value
            },
            provider: provider_section,
            tracker: TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite {
                        database_name: "tracker.db".to_string(),
                    },
                    private: false,
                },
                udp_trackers: vec![UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".to_string(),
                }],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:7070".to_string(),
                }],
                http_api: HttpApiConfig {
                    admin_token: "MyAccessToken".to_string(),
                },
            },
        }
    }

    /// Generates a configuration template file at the specified path
    ///
    /// This method creates a JSON configuration file with placeholder values
    /// that users can edit. The file is formatted with pretty-printing for
    /// better readability.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the template file should be created
    /// * `provider` - Provider to generate template for (lxd or hetzner)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Template file created successfully
    /// * `Err(CreateConfigError)` - File creation or serialization failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directory cannot be created
    /// - Template serialization fails (unlikely - indicates a bug)
    /// - File cannot be written due to permissions or I/O errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
    /// use torrust_tracker_deployer_lib::domain::provider::Provider;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// EnvironmentCreationConfig::generate_template_file(
    ///     Path::new("./environment-config.json"),
    ///     Provider::Lxd
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_template_file(
        path: &std::path::Path,
        provider: Provider,
    ) -> Result<(), CreateConfigError> {
        // Create template instance with placeholders
        let template = Self::template(provider);

        // Serialize to pretty-printed JSON
        let json = serde_json::to_string_pretty(&template)
            .map_err(|source| CreateConfigError::TemplateSerializationFailed { source })?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                CreateConfigError::TemplateDirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        // Write template to file
        std::fs::write(path, json).map_err(|source| {
            CreateConfigError::TemplateFileWriteFailed {
                path: path.to_path_buf(),
                source,
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::create::config::provider::LxdProviderSection;
    use crate::domain::provider::Provider;

    /// Helper to create a default LXD provider section for tests
    fn default_lxd_provider(profile_name: &str) -> ProviderSection {
        ProviderSection::Lxd(LxdProviderSection {
            profile_name: profile_name.to_string(),
        })
    }

    #[test]
    fn test_create_environment_creation_config() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
        );

        assert_eq!(config.environment.name, "dev");
        assert_eq!(
            config.ssh_credentials.private_key_path,
            "fixtures/testing_rsa"
        );
        assert_eq!(config.ssh_credentials.username, "torrust");
        assert_eq!(config.ssh_credentials.port, 22);
        assert_eq!(config.provider.provider(), Provider::Lxd);
    }

    #[test]
    fn test_deserialize_from_json_with_lxd_provider() {
        let json = r#"{
            "environment": {
                "name": "e2e-config"
            },
            "ssh_credentials": {
                "private_key_path": "fixtures/testing_rsa",
                "public_key_path": "fixtures/testing_rsa.pub"
            },
            "provider": {
                "provider": "lxd",
                "profile_name": "torrust-profile-e2e-config"
            },
            "tracker": {
                "core": {
                    "database": {
                        "driver": "sqlite3",
                        "database_name": "tracker.db"
                    },
                    "private": false
                },
                "udp_trackers": [
                    {
                        "bind_address": "0.0.0.0:6969"
                    }
                ],
                "http_trackers": [
                    {
                        "bind_address": "0.0.0.0:7070"
                    }
                ],
                "http_api": {
                    "admin_token": "MyAccessToken"
                }
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
        assert_eq!(config.provider.provider(), Provider::Lxd);
    }

    #[test]
    fn test_deserialize_from_json_with_hetzner_provider() {
        let json = r#"{
            "environment": {
                "name": "production",
                "instance_name": "torrust-tracker-demo"
            },
            "ssh_credentials": {
                "private_key_path": "keys/prod_key",
                "public_key_path": "keys/prod_key.pub",
                "username": "ubuntu",
                "port": 2222
            },
            "provider": {
                "provider": "hetzner",
                "api_token": "test-token",
                "server_type": "cx22",
                "location": "nbg1",
                "image": "ubuntu-24.04"
            },
            "tracker": {
                "core": {
                    "database": {
                        "driver": "sqlite3",
                        "database_name": "tracker.db"
                    },
                    "private": false
                },
                "udp_trackers": [
                    {
                        "bind_address": "0.0.0.0:6969"
                    }
                ],
                "http_trackers": [
                    {
                        "bind_address": "0.0.0.0:7070"
                    }
                ],
                "http_api": {
                    "admin_token": "MyAccessToken"
                }
            }
        }"#;

        let config: EnvironmentCreationConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.environment.name, "production");
        assert_eq!(
            config.environment.instance_name,
            Some("torrust-tracker-demo".to_string())
        );
        assert_eq!(config.ssh_credentials.private_key_path, "keys/prod_key");
        assert_eq!(config.ssh_credentials.public_key_path, "keys/prod_key.pub");
        assert_eq!(config.ssh_credentials.username, "ubuntu");
        assert_eq!(config.ssh_credentials.port, 2222);
        assert_eq!(config.provider.provider(), Provider::Hetzner);
    }

    #[test]
    fn test_serialize_environment_creation_config() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "staging".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "keys/stage_key".to_string(),
                "keys/stage_key.pub".to_string(),
                "deploy".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-staging"),
            TrackerConfig::default(),
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_convert_to_environment_params_success_auto_generated_instance_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None, // Auto-generate
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
        );

        let result = config.to_environment_params();
        assert!(result.is_ok(), "Expected successful conversion");

        let (name, instance_name, provider_config, credentials, port, _tracker) = result.unwrap();

        assert_eq!(name.as_str(), "dev");
        assert_eq!(instance_name.as_str(), "torrust-tracker-vm-dev"); // Auto-generated
        assert_eq!(provider_config.provider(), Provider::Lxd);
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
        assert_eq!(port, 22);
    }

    #[test]
    fn test_convert_to_environment_params_success_custom_instance_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "prod".to_string(),
                instance_name: Some("my-custom-instance".to_string()),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-prod"),
            TrackerConfig::default(),
        );

        let result = config.to_environment_params();
        assert!(result.is_ok(), "Expected successful conversion");

        let (name, instance_name, _provider_config, _credentials, _port, _tracker) =
            result.unwrap();

        assert_eq!(name.as_str(), "prod");
        assert_eq!(instance_name.as_str(), "my-custom-instance"); // Custom provided
    }

    #[test]
    fn test_convert_to_environment_params_invalid_environment_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "Invalid_Name".to_string(), // uppercase - invalid
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile"),
            TrackerConfig::default(),
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
    fn test_convert_to_environment_params_invalid_instance_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: Some("invalid-".to_string()), // ends with dash - invalid
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile"),
            TrackerConfig::default(),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidInstanceName { name, reason } => {
                assert_eq!(name, "invalid-");
                assert!(reason.contains("dash"));
            }
            other => panic!("Expected InvalidInstanceName error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_environment_params_invalid_profile_name() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            ProviderSection::Lxd(LxdProviderSection {
                profile_name: "invalid-".to_string(), // ends with dash - invalid
            }),
            TrackerConfig::default(),
        );

        let result = config.to_environment_params();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidProfileName(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidProfileName error, got: {other:?}"),
        }
    }

    #[test]
    fn test_convert_to_environment_params_invalid_username() {
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "123invalid".to_string(), // starts with number - invalid
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
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
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "/nonexistent/key".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
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
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "/nonexistent/key.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
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
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-test-env"),
            TrackerConfig::default(),
        );

        let (name, _instance_name, provider_config, credentials, port, _tracker) =
            config.to_environment_params().unwrap();
        let environment = Environment::new(name.clone(), provider_config, credentials, port);

        assert_eq!(environment.name().as_str(), "test-env");
        assert_eq!(environment.ssh_username().as_str(), "torrust");
        assert_eq!(environment.ssh_port(), 22);
        // The environment auto-generates its own instance name in Environment::new()
        // which may differ from what we pass (it's derived from name internally)
    }

    #[test]
    fn test_round_trip_serialization() {
        let original = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: Some("my-vm".to_string()),
            },
            SshCredentialsConfig::new(
                "fixtures/testing_rsa".to_string(),
                "fixtures/testing_rsa.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerConfig::default(),
        );

        let json = serde_json::to_string_pretty(&original).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_template_has_placeholder_values() {
        let template = EnvironmentCreationConfig::template(Provider::Lxd);

        assert_eq!(template.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
        assert_eq!(template.environment.instance_name, None);
        assert_eq!(
            template.ssh_credentials.private_key_path,
            "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH"
        );
        assert_eq!(
            template.ssh_credentials.public_key_path,
            "REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH"
        );
        assert_eq!(template.ssh_credentials.username, "torrust");
        assert_eq!(template.ssh_credentials.port, 22);
        assert_eq!(template.provider.provider(), Provider::Lxd);
    }

    #[test]
    fn test_template_for_hetzner_has_placeholder_values() {
        let template = EnvironmentCreationConfig::template(Provider::Hetzner);

        assert_eq!(template.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
        assert_eq!(template.environment.instance_name, None);
        assert_eq!(
            template.ssh_credentials.private_key_path,
            "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH"
        );
        assert_eq!(
            template.ssh_credentials.public_key_path,
            "REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH"
        );
        assert_eq!(template.ssh_credentials.username, "torrust");
        assert_eq!(template.ssh_credentials.port, 22);
        assert_eq!(template.provider.provider(), Provider::Hetzner);

        // Verify Hetzner-specific fields
        if let ProviderSection::Hetzner(hetzner) = template.provider {
            assert_eq!(hetzner.api_token, "REPLACE_WITH_HETZNER_API_TOKEN");
            assert_eq!(hetzner.server_type, "cx22");
            assert_eq!(hetzner.location, "nbg1");
        } else {
            panic!("Expected Hetzner provider section");
        }
    }

    #[test]
    fn test_template_serializes_to_valid_json() {
        let template = EnvironmentCreationConfig::template(Provider::Lxd);
        let json = serde_json::to_string_pretty(&template).unwrap();

        // Verify it can be deserialized back
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(template, deserialized);

        // Verify JSON contains provider section
        assert!(json.contains("\"provider\""));
        assert!(json.contains("\"lxd\""));
        assert!(json.contains("REPLACE_WITH_LXD_PROFILE_NAME"));
    }

    #[test]
    fn test_template_structure_matches_config() {
        let template = EnvironmentCreationConfig::template(Provider::Lxd);

        // Verify template has same structure as regular config
        let regular_config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "test".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "path1".to_string(),
                "path2".to_string(),
                "user".to_string(),
                22,
            ),
            default_lxd_provider("test-profile"),
            TrackerConfig::default(),
        );

        // Both should serialize to same structure (different values)
        let template_json = serde_json::to_value(&template).unwrap();
        let config_json = serde_json::to_value(&regular_config).unwrap();

        // Check structure matches
        assert!(template_json.is_object());
        assert!(config_json.is_object());

        let template_obj = template_json.as_object().unwrap();
        let config_obj = config_json.as_object().unwrap();

        assert_eq!(template_obj.keys().len(), config_obj.keys().len());
        assert!(template_obj.contains_key("environment"));
        assert!(template_obj.contains_key("ssh_credentials"));
        assert!(template_obj.contains_key("provider"));
    }

    #[test]
    fn test_generate_template_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("config.json");

        let result =
            EnvironmentCreationConfig::generate_template_file(&template_path, Provider::Lxd);
        assert!(result.is_ok());

        // Verify file exists
        assert!(template_path.exists());

        // Verify content is valid JSON
        let content = std::fs::read_to_string(&template_path).unwrap();
        let parsed: EnvironmentCreationConfig = serde_json::from_str(&content).unwrap();

        // Verify placeholders are present
        assert_eq!(parsed.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
        assert_eq!(
            parsed.ssh_credentials.private_key_path,
            "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH"
        );
        // Verify provider section is present
        assert_eq!(parsed.provider.provider(), Provider::Lxd);
    }

    #[test]
    fn test_generate_template_file_creates_parent_directories() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("configs")
            .join("env")
            .join("test.json");

        let result = EnvironmentCreationConfig::generate_template_file(&nested_path, Provider::Lxd);
        assert!(result.is_ok());

        // Verify nested directories were created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_generate_template_file_overwrites_existing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("config.json");

        // Create initial file
        std::fs::write(&template_path, "old content").unwrap();

        // Generate template should overwrite
        let result =
            EnvironmentCreationConfig::generate_template_file(&template_path, Provider::Lxd);
        assert!(result.is_ok());

        // Verify content was replaced
        let content = std::fs::read_to_string(&template_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn test_generate_template_file_sync() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("config.json");

        let result =
            EnvironmentCreationConfig::generate_template_file(&template_path, Provider::Lxd);
        assert!(result.is_ok());

        // Verify file exists
        assert!(template_path.exists());

        // Verify content is valid JSON
        let content = std::fs::read_to_string(&template_path).unwrap();
        let parsed: EnvironmentCreationConfig = serde_json::from_str(&content).unwrap();

        // Verify placeholders are present
        assert_eq!(parsed.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
        assert_eq!(
            parsed.ssh_credentials.private_key_path,
            "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH"
        );
    }

    #[test]
    fn test_generate_template_file_sync_creates_parent_directories() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("configs")
            .join("env")
            .join("test.json");

        let result = EnvironmentCreationConfig::generate_template_file(&nested_path, Provider::Lxd);
        assert!(result.is_ok());

        // Verify nested directories were created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }
}
