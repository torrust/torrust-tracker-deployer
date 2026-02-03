//! Environment creation configuration value object
//!
//! This module provides the `EnvironmentCreationConfig` type which represents
//! all configuration needed to create a deployment environment. It handles
//! deserialization from configuration sources and conversion to domain types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::provider::Provider;

use super::backup::BackupSection;
use super::errors::CreateConfigError;
use super::grafana::GrafanaSection;
use super::https::HttpsSection;
use super::prometheus::PrometheusSection;
use super::provider::{HetznerProviderSection, LxdProviderSection, ProviderSection};
use super::ssh_credentials_config::SshCredentialsConfig;
use super::tracker::TrackerSection;

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
///             "bind_address": "0.0.0.0:1212",
///             "admin_token": "MyAccessToken"
///         },
///         "health_check_api": {
///             "bind_address": "127.0.0.1:1313"
///         }
///     },
///     "prometheus": {
///         "scrape_interval_in_secs": 15
///     },
///     "grafana": {
///         "admin_user": "admin",
///         "admin_password": "admin"
///     },
///     "backup": {
///         "schedule": "0 3 * * *",
///         "retention_days": 7
///     }
/// }"#;
///
/// let config: EnvironmentCreationConfig = serde_json::from_str(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentCreationConfig {
    /// Environment-specific settings
    pub environment: EnvironmentSection,

    /// SSH credentials configuration
    pub ssh_credentials: SshCredentialsConfig,

    /// Provider-specific configuration (LXD, Hetzner, etc.)
    ///
    /// Uses `ProviderSection` for JSON parsing with raw primitives.
    /// Converted to domain `ProviderConfig` via `TryInto<EnvironmentParams>`.
    pub provider: ProviderSection,

    /// Tracker deployment configuration
    ///
    /// Uses `TrackerSection` for JSON parsing with String primitives.
    /// Converted to domain `TrackerConfig` via `TryInto<EnvironmentParams>`.
    pub tracker: TrackerSection,

    /// Prometheus monitoring configuration (optional)
    ///
    /// When present, Prometheus will be deployed to monitor the tracker.
    /// Uses `PrometheusSection` for JSON parsing with String primitives.
    /// Converted to domain `PrometheusConfig` via `TryInto<EnvironmentParams>`.
    #[serde(default)]
    pub prometheus: Option<PrometheusSection>,

    /// Grafana dashboard configuration (optional)
    ///
    /// When present, Grafana will be deployed for visualization.
    /// **Requires Prometheus to be configured** - Grafana depends on
    /// Prometheus as its data source.
    ///
    /// Uses `GrafanaSection` for JSON parsing with String primitives.
    /// Converted to domain `GrafanaConfig` via `TryInto<EnvironmentParams>`.
    #[serde(default)]
    pub grafana: Option<GrafanaSection>,

    /// HTTPS configuration (optional)
    ///
    /// When present, enables HTTPS for services that have TLS configured.
    /// Contains common settings like admin email for Let's Encrypt.
    ///
    /// **Required if any service has TLS configured** - The `admin_email`
    /// is needed for Let's Encrypt certificate management.
    ///
    /// Uses `HttpsSection` for JSON parsing.
    #[serde(default)]
    pub https: Option<HttpsSection>,

    /// Backup configuration (optional)
    ///
    /// When present, automated backups will be configured for the tracker
    /// database and other persistent data.
    ///
    /// Uses `BackupSection` for JSON parsing with String primitives (cron schedule).
    /// Converted to domain `BackupConfig` via `TryInto<EnvironmentParams>`.
    ///
    /// Default schedule: 3:00 AM daily ("0 3 * * *")
    /// Default retention: 7 days
    #[serde(default)]
    pub backup: Option<BackupSection>,
}

/// Environment-specific configuration section
///
/// Contains configuration specific to the environment being created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    /// ```no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    ///     ProviderSection, LxdProviderSection
    /// };
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::tracker::TrackerSection;
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
    ///     TrackerSection::default(),
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    /// );
    /// ```
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        environment: EnvironmentSection,
        ssh_credentials: SshCredentialsConfig,
        provider: ProviderSection,
        tracker: TrackerSection,
        prometheus: Option<PrometheusSection>,
        grafana: Option<GrafanaSection>,
        https: Option<HttpsSection>,
        backup: Option<BackupSection>,
    ) -> Self {
        Self {
            environment,
            ssh_credentials,
            provider,
            tracker,
            prometheus,
            grafana,
            https,
            backup,
        }
    }

    /// Checks if any service has TLS configured
    ///
    /// Returns `true` if at least one of the following services has TLS:
    /// - Tracker HTTP API
    /// - Any HTTP tracker
    /// - Grafana
    ///
    /// This is used for validation to ensure that when the HTTPS section is
    /// defined, at least one service actually uses it.
    #[must_use]
    pub fn has_any_tls_configured(&self) -> bool {
        // Check HTTP API
        if self.tracker.http_api.use_tls_proxy == Some(true) {
            return true;
        }

        // Check HTTP trackers
        for http_tracker in &self.tracker.http_trackers {
            if http_tracker.use_tls_proxy == Some(true) {
                return true;
            }
        }

        // Check Grafana
        if let Some(ref grafana) = self.grafana {
            if grafana.use_tls_proxy == Some(true) {
                return true;
            }
        }

        false
    }

    // Note: validate_https_config() method has been removed.
    // Email validation now happens in domain layer (HttpsConfig::new())
    // Cross-service validation (TLS/HTTPS consistency) happens in UserInputs::with_tracker()

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
    ///
    /// # Panics
    ///
    /// Panics if default IP addresses fail to parse (should never happen with valid constants).
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
            tracker: TrackerSection {
                core: super::tracker::TrackerCoreSection {
                    database: super::tracker::DatabaseSection::Sqlite {
                        database_name: "tracker.db".to_string(),
                    },
                    private: false,
                },
                udp_trackers: vec![super::tracker::UdpTrackerSection {
                    bind_address: "0.0.0.0:6969".to_string(),
                    domain: None,
                }],
                http_trackers: vec![super::tracker::HttpTrackerSection {
                    bind_address: "0.0.0.0:7070".to_string(),
                    domain: None,
                    use_tls_proxy: None,
                }],
                http_api: super::tracker::HttpApiSection {
                    bind_address: "0.0.0.0:1212".to_string(),
                    admin_token: "MyAccessToken".to_string(),
                    domain: None,
                    use_tls_proxy: None,
                },
                health_check_api: super::tracker::HealthCheckApiSection::default(),
            },
            prometheus: Some(PrometheusSection::default()),
            grafana: Some(GrafanaSection::default()),
            https: None, // Set to HttpsSection with admin_email for HTTPS deployments
            backup: Some(super::backup::BackupSection::default()), // Backups enabled by default with daily 3 AM schedule and 7-day retention
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
    use crate::application::command_handlers::create::config::tracker::TrackerSection;
    use crate::domain::environment::EnvironmentParams;
    use crate::domain::provider::Provider;

    /// Helper to create a default LXD provider section for tests
    fn default_lxd_provider(profile_name: &str) -> ProviderSection {
        ProviderSection::Lxd(LxdProviderSection {
            profile_name: profile_name.to_string(),
        })
    }

    #[test]
    fn it_should_create_environment_creation_config_when_provided_valid_inputs() {
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
            TrackerSection::default(),
            None,
            None,
            None,
            None,
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
    fn it_should_deserialize_from_json_when_using_lxd_provider() {
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
                    "bind_address": "0.0.0.0:1212",
                    "admin_token": "MyAccessToken"
                },
                "health_check_api": {
                    "bind_address": "127.0.0.1:1313"
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
    fn it_should_deserialize_from_json_when_using_hetzner_provider() {
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
                    "bind_address": "0.0.0.0:1212",
                    "admin_token": "MyAccessToken"
                },
                "health_check_api": {
                    "bind_address": "127.0.0.1:1313"
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
    fn it_should_serialize_to_json_when_converting_environment_creation_config() {
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
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn it_should_convert_to_environment_params_when_using_auto_generated_instance_name() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None, // Auto-generate
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_ok(), "Expected successful conversion");

        let params = result.unwrap();

        assert_eq!(params.environment_name.as_str(), "dev");
        assert_eq!(params.instance_name.as_str(), "torrust-tracker-vm-dev"); // Auto-generated
        assert_eq!(params.provider_config.provider(), Provider::Lxd);
        assert_eq!(params.ssh_credentials.ssh_username.as_str(), "torrust");
        assert_eq!(params.ssh_port, 22);
    }

    #[test]
    fn it_should_convert_to_environment_params_when_using_custom_instance_name() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "prod".to_string(),
                instance_name: Some("my-custom-instance".to_string()),
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-prod"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_ok(), "Expected successful conversion");

        let params = result.unwrap();

        assert_eq!(params.environment_name.as_str(), "prod");
        assert_eq!(params.instance_name.as_str(), "my-custom-instance"); // Custom provided
    }

    #[test]
    fn it_should_return_error_when_environment_name_is_invalid() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "Invalid_Name".to_string(), // uppercase - invalid
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidEnvironmentName(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidEnvironmentName error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_when_instance_name_is_invalid() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: Some("invalid-".to_string()), // ends with dash - invalid
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
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
    fn it_should_return_error_when_profile_name_is_invalid() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            ProviderSection::Lxd(LxdProviderSection {
                profile_name: "invalid-".to_string(), // ends with dash - invalid
            }),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidProfileName(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidProfileName error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_when_username_is_invalid() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                private_key_path,
                public_key_path,
                "123invalid".to_string(), // starts with number - invalid
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::InvalidUsername(_) => {
                // Expected error
            }
            other => panic!("Expected InvalidUsername error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_when_private_key_file_not_found() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                "/nonexistent/key".to_string(),
                public_key_path,
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PrivateKeyNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected PrivateKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_when_public_key_file_not_found() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(
                private_key_path,
                "/nonexistent/key.pub".to_string(),
                "torrust".to_string(),
                22,
            ),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateConfigError::PublicKeyNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected PublicKeyNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_integrate_with_environment_new_when_creating_from_config() {
        // This test verifies that the converted parameters work with Environment::new()
        use crate::domain::Environment;
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "test-env".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-test-env"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let params: EnvironmentParams = config.try_into().unwrap();

        // Create environment using the factory pattern with all required parameters
        let working_dir = std::path::Path::new("/tmp/test-env");
        let environment = Environment::create(params, working_dir, chrono::Utc::now()).unwrap();

        assert_eq!(environment.name().as_str(), "test-env");
        assert_eq!(environment.ssh_username().as_str(), "torrust");
        assert_eq!(environment.ssh_port(), 22);
        // The environment auto-generates its own instance name in Environment::new()
        // which may differ from what we pass (it's derived from name internally)
    }

    #[test]
    fn it_should_preserve_data_when_performing_round_trip_serialization() {
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
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let json = serde_json::to_string_pretty(&original).unwrap();
        let deserialized: EnvironmentCreationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn it_should_contain_placeholder_values_when_generating_lxd_template() {
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
    fn it_should_contain_placeholder_values_when_generating_hetzner_template() {
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
    fn it_should_serialize_to_valid_json_when_converting_template() {
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
    fn it_should_match_config_structure_when_validating_template() {
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
            TrackerSection::default(),
            None,
            None,
            None,
            None,
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
    fn it_should_generate_template_file_when_creating_new_template() {
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
    fn it_should_create_parent_directories_when_generating_template_file() {
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
    fn it_should_overwrite_existing_file_when_generating_template() {
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
    fn it_should_generate_template_file_synchronously_when_called() {
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
    fn it_should_create_parent_directories_when_generating_template_file_sync() {
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

    // HTTPS Validation Tests

    #[test]
    fn it_should_return_false_for_has_any_tls_configured_when_no_tls() {
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
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        assert!(!config.has_any_tls_configured());
    }

    #[test]
    fn it_should_return_true_for_has_any_tls_configured_when_http_api_has_tls() {
        use crate::application::command_handlers::create::config::tracker::{
            DatabaseSection, HealthCheckApiSection, HttpApiSection, HttpTrackerSection,
            TrackerCoreSection, TrackerSection, UdpTrackerSection,
        };

        let tracker_section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:6969".to_string(),
                domain: None,
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
                domain: None,
                use_tls_proxy: None,
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
                domain: Some("api.tracker.example.com".to_string()),
                use_tls_proxy: Some(true),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

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
            tracker_section,
            None,
            None,
            None,
            None,
        );

        assert!(config.has_any_tls_configured());
    }

    #[test]
    fn it_should_pass_conversion_when_no_https_section() {
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        // Config with no HTTPS section should convert successfully
        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    }

    // Note: Tests for TLS/HTTPS cross-service validation have been moved to domain layer.
    // See UserInputs::with_tracker() tests in src/domain/environment/user_inputs.rs
    // - it_should_reject_tls_services_without_https_section
    // - it_should_reject_https_section_without_tls_services

    #[test]
    fn it_should_pass_conversion_when_https_section_has_valid_email() {
        use crate::application::command_handlers::create::config::https::HttpsSection;
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Email validation now happens in domain layer (HttpsConfig::new())
        // This test verifies that valid emails pass through TryInto<EnvironmentParams>
        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            Some(HttpsSection {
                admin_email: "admin@example.com".to_string(),
                use_staging: false,
            }),
            None,
        );

        // HTTPS section with valid email should convert successfully
        // (actual cross-service TLS validation happens in domain layer)
        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    }

    #[test]
    fn it_should_reject_invalid_email_in_https_section() {
        use crate::application::command_handlers::create::config::https::HttpsSection;
        use crate::domain::https::HttpsConfigError;
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-dev"),
            TrackerSection::default(),
            None,
            None,
            Some(HttpsSection {
                admin_email: "invalid-email".to_string(), // Invalid email
                use_staging: false,
            }),
            None,
        );

        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::HttpsConfigInvalid(HttpsConfigError::InvalidEmail { .. })
        ));
    }

    #[test]
    fn it_should_pass_conversion_when_https_section_with_tls() {
        use crate::application::command_handlers::create::config::https::HttpsSection;
        use crate::application::command_handlers::create::config::tracker::{
            DatabaseSection, HealthCheckApiSection, HttpApiSection, HttpTrackerSection,
            TrackerCoreSection, TrackerSection, UdpTrackerSection,
        };
        use std::env;

        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let tracker_section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:6969".to_string(),
                domain: None,
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
                domain: Some("tracker.example.com".to_string()),
                use_tls_proxy: Some(true),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
                domain: None,
                use_tls_proxy: None,
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "dev".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            default_lxd_provider("torrust-profile-dev"),
            tracker_section,
            None,
            None,
            Some(HttpsSection {
                admin_email: "admin@example.com".to_string(),
                use_staging: false,
            }),
            None,
        );

        // Note: Email validation now happens in domain layer (HttpsConfig::new())
        // Cross-service TLS/HTTPS validation happens in domain layer (UserInputs)
        // This test verifies the DTO can convert to environment params
        let result: Result<EnvironmentParams, _> = config.try_into();
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    }
}
