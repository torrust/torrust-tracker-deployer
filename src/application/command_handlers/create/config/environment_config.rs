//! Environment creation configuration value object
//!
//! This module provides the `EnvironmentCreationConfig` type which represents
//! all configuration needed to create a deployment environment. It handles
//! deserialization from configuration sources and conversion to domain types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::adapters::ssh::SshCredentials;
use crate::domain::grafana::GrafanaConfig;
use crate::domain::https::HttpsConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::{Provider, ProviderConfig};
use crate::domain::tracker::TrackerConfig;
use crate::domain::{EnvironmentName, InstanceName};

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
    /// Converted to domain `ProviderConfig` via `to_environment_params()`.
    pub provider: ProviderSection,

    /// Tracker deployment configuration
    ///
    /// Uses `TrackerSection` for JSON parsing with String primitives.
    /// Converted to domain `TrackerConfig` via `to_environment_params()`.
    pub tracker: TrackerSection,

    /// Prometheus monitoring configuration (optional)
    ///
    /// When present, Prometheus will be deployed to monitor the tracker.
    /// Uses `PrometheusSection` for JSON parsing with String primitives.
    /// Converted to domain `PrometheusConfig` via `to_environment_params()`.
    #[serde(default)]
    pub prometheus: Option<PrometheusSection>,

    /// Grafana dashboard configuration (optional)
    ///
    /// When present, Grafana will be deployed for visualization.
    /// **Requires Prometheus to be configured** - Grafana depends on
    /// Prometheus as its data source.
    ///
    /// Uses `GrafanaSection` for JSON parsing with String primitives.
    /// Converted to domain `GrafanaConfig` via `to_environment_params()`.
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
    /// );
    /// ```
    #[must_use]
    pub fn new(
        environment: EnvironmentSection,
        ssh_credentials: SshCredentialsConfig,
        provider: ProviderSection,
        tracker: TrackerSection,
        prometheus: Option<PrometheusSection>,
        grafana: Option<GrafanaSection>,
        https: Option<HttpsSection>,
    ) -> Self {
        Self {
            environment,
            ssh_credentials,
            provider,
            tracker,
            prometheus,
            grafana,
            https,
        }
    }

    /// Converts configuration to domain parameters for `Environment::new()`
    ///
    /// This method validates all configuration values and converts them to
    /// strongly-typed domain objects that can be used to create an Environment.
    ///
    /// # Returns
    ///
    /// Returns a tuple of domain types.
    ///
    /// # Validation
    ///
    /// - Environment name must follow naming rules (see `EnvironmentName`)
    /// - Instance name (if provided) must follow instance naming rules (see `InstanceName`)
    /// - Provider config must be valid (e.g., valid profile name for LXD)
    /// - SSH username must follow Linux username requirements (see `Username`)
    /// - SSH key files must exist and be accessible
    /// - Grafana requires Prometheus (dependency validation)
    /// - HTTPS configuration must be consistent (section present iff services have TLS)
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
    /// - Grafana is configured but Prometheus is not (dependency violation)
    /// - HTTPS section is defined but no service has TLS configured
    /// - A service has TLS configured but HTTPS section is missing
    /// - HTTPS admin email is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
    ///     ProviderSection, LxdProviderSection
    /// };
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::tracker::TrackerSection;
    /// use torrust_tracker_deployer_lib::domain::Environment;
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
    ///     None, // HTTPS configuration
    /// );
    ///
    /// let result = config.to_environment_params()?;
    ///
    /// // Instance name auto-generated from environment name
    /// assert_eq!(result.1.as_str(), "torrust-tracker-vm-dev");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[allow(clippy::type_complexity)]
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
            Option<PrometheusConfig>,
            Option<GrafanaConfig>,
            Option<HttpsConfig>,
        ),
        CreateConfigError,
    > {
        // Validate HTTPS configuration consistency before any other conversion
        self.validate_https_config()?;

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

        // Convert TrackerSection (DTO) to domain TrackerConfig (validates bind addresses, etc.)
        let tracker_config = self.tracker.to_tracker_config()?;

        // Convert Prometheus and Grafana sections to domain types
        let prometheus_config = self
            .prometheus
            .map(|section| section.to_prometheus_config())
            .transpose()?;

        let grafana_config = self
            .grafana
            .map(|section| section.to_grafana_config())
            .transpose()?;

        // Note: Grafana-Prometheus dependency is now validated at domain level
        // in UserInputs::with_tracker() when the environment is created

        // Convert HTTPS section to domain type (already validated above)
        let https_config = self
            .https
            .map(|section| HttpsConfig::new(section.admin_email, section.use_staging));

        Ok((
            environment_name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker_config,
            prometheus_config,
            grafana_config,
            https_config,
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

    /// Validates HTTPS section configuration details
    ///
    /// Note: Cross-service validation (TLS/HTTPS section consistency) is now
    /// enforced by the domain layer in `UserInputs`. This method only validates
    /// the HTTPS section's internal details (like admin email format).
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidAdminEmail` if the admin email format is invalid.
    pub fn validate_https_config(&self) -> Result<(), CreateConfigError> {
        // Cross-service validation (HTTPS section + TLS consistency) is now
        // enforced by UserInputs in the domain layer. We only validate the
        // HTTPS section's internal details here.
        if let Some(https_section) = &self.https {
            https_section.validate()?;
        }
        Ok(())
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
        );

        let result = config.to_environment_params();
        assert!(result.is_ok(), "Expected successful conversion");

        let (
            name,
            instance_name,
            provider_config,
            credentials,
            port,
            _tracker,
            _prometheus,
            _grafana,
            _https,
        ) = result.unwrap();

        assert_eq!(name.as_str(), "dev");
        assert_eq!(instance_name.as_str(), "torrust-tracker-vm-dev"); // Auto-generated
        assert_eq!(provider_config.provider(), Provider::Lxd);
        assert_eq!(credentials.ssh_username.as_str(), "torrust");
        assert_eq!(port, 22);
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
        );

        let result = config.to_environment_params();
        assert!(result.is_ok(), "Expected successful conversion");

        let (
            name,
            instance_name,
            _provider_config,
            _credentials,
            _port,
            _tracker,
            _prometheus,
            _grafana,
            _https,
        ) = result.unwrap();

        assert_eq!(name.as_str(), "prod");
        assert_eq!(instance_name.as_str(), "my-custom-instance"); // Custom provided
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
        );

        let (
            name,
            _instance_name,
            provider_config,
            credentials,
            port,
            _tracker,
            _prometheus,
            _grafana,
            _https,
        ) = config.to_environment_params().unwrap();
        let environment = Environment::new(
            name.clone(),
            provider_config,
            credentials,
            port,
            chrono::Utc::now(),
        );

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
        );

        assert!(config.has_any_tls_configured());
    }

    #[test]
    fn it_should_pass_validation_when_no_https_section() {
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
        );

        // validate_https_config only checks HTTPS section internal details (admin email)
        // Cross-service TLS/HTTPS validation is now in domain layer (UserInputs)
        assert!(config.validate_https_config().is_ok());
    }

    // Note: Tests for TLS/HTTPS cross-service validation have been moved to domain layer.
    // See UserInputs::with_tracker() tests in src/domain/environment/user_inputs.rs
    // - it_should_reject_tls_services_without_https_section
    // - it_should_reject_https_section_without_tls_services

    #[test]
    fn it_should_pass_validation_when_https_section_has_valid_email() {
        use crate::application::command_handlers::create::config::https::HttpsSection;

        // Note: validate_https_config only validates HTTPS section internal details
        // (like admin email format). Cross-service validation (TLS/HTTPS consistency)
        // is now enforced by UserInputs in the domain layer.
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
            Some(HttpsSection {
                admin_email: "admin@example.com".to_string(),
                use_staging: false,
            }),
        );

        // HTTPS section with valid email should pass this validation
        // (cross-service TLS check is done at domain layer)
        assert!(config.validate_https_config().is_ok());
    }

    #[test]
    fn it_should_pass_validation_when_https_section_with_tls() {
        use crate::application::command_handlers::create::config::https::HttpsSection;
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
            Some(HttpsSection {
                admin_email: "admin@example.com".to_string(),
                use_staging: false,
            }),
        );

        assert!(config.validate_https_config().is_ok());
    }
}
