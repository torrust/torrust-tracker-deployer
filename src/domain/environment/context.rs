//! Environment Context Module
//!
//! This module contains the `EnvironmentContext` struct which composes three
//! semantic types to organize state-independent environment data.
//!
//! ## Purpose
//!
//! The `EnvironmentContext` separates immutable environment configuration from
//! the mutable state machine, and further organizes that configuration into
//! three distinct semantic categories:
//!
//! 1. **User Inputs** - Configuration provided by users
//! 2. **Internal Config** - Derived paths for organizing artifacts
//! 3. **Runtime Outputs** - Data generated during deployment
//!
//! ## Benefits
//!
//! - **Reduced pattern matching**: Access common fields without matching on state (83% reduction)
//! - **Clear semantic boundaries**: Types document the purpose of each field
//! - **Developer guidance**: Clear where to add new fields based on their purpose
//! - **Simplified state transitions**: Only the state changes, context remains constant
//! - **Easier extension**: Adding fields is straightforward with clear categorization
//!
//! ## Three-Way Semantic Split
//!
//! ### When to Add Fields
//!
//! - **`UserInputs`**: User needs to configure something at environment creation time
//! - **`InternalConfig`**: Need internal paths or derived configuration
//! - **`RuntimeOutputs`**: Operations produce new data about deployed infrastructure
//!
//! ### Design Rationale
//!
//! By organizing fields into three semantic categories, we make it immediately
//! clear where each piece of information comes from and guide developers on
//! where to add new fields as the application evolves.

use crate::adapters::ssh::SshCredentials;
use crate::domain::environment::{EnvironmentName, InternalConfig, RuntimeOutputs, UserInputs};
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::ProviderConfig;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default value for `created_at` field for backward compatibility
///
/// Returns Unix epoch (1970-01-01 00:00:00 UTC) for environments created
/// before the `created_at` field was added.
fn default_created_at() -> DateTime<Utc> {
    Utc.timestamp_opt(0, 0).unwrap()
}

/// Complete environment context composed of three semantic types
///
/// The context is split into three logical categories:
/// 1. **User Inputs** (`user_inputs`): Configuration provided by users
/// 2. **Internal Config** (`internal_config`): Derived paths for organizing artifacts
/// 3. **Runtime Outputs** (`runtime_outputs`): Data generated during deployment
///
/// This separation makes it clear where each piece of information comes from
/// and helps developers understand where to add new fields.
///
/// # Design Rationale
///
/// By separating state-independent data from the state machine and organizing
/// it into three semantic categories, we:
/// - Eliminate repetitive pattern matching in `AnyEnvironmentState`
/// - Make it clear which data is constant vs. state-dependent
/// - Provide semantic clarity about the purpose of each field
/// - Guide developers where to add new fields based on their purpose
/// - Simplify state transitions (only the state field changes)
/// - Enable easier extension of environment configuration
///
/// # Three Semantic Categories
///
/// - **User Inputs**: Immutable user configuration (name, SSH credentials, port)
/// - **Internal Config**: Derived paths (`build_dir`, `data_dir`)
/// - **Runtime Outputs**: Generated during deployment (`instance_ip`, future metrics)
///
/// # Examples
///
/// `EnvironmentContext` is typically created internally by `Environment::new()`:
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::{Environment, EnvironmentName};
/// use torrust_tracker_deployer_lib::domain::provider::{LxdConfig, ProviderConfig};
/// use torrust_tracker_deployer_lib::domain::ProfileName;
/// use torrust_tracker_deployer_lib::shared::Username;
/// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
/// use std::path::PathBuf;
/// use chrono::{TimeZone, Utc};
///
/// let env_name = EnvironmentName::new("production".to_string())?;
/// let ssh_username = Username::new("torrust".to_string())?;
/// let ssh_credentials = SshCredentials::new(
///     PathBuf::from("keys/prod_rsa"),
///     PathBuf::from("keys/prod_rsa.pub"),
///     ssh_username,
/// );
/// let provider_config = ProviderConfig::Lxd(LxdConfig {
///     profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
/// });
///
/// // Environment::new() creates the EnvironmentContext internally
/// let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
/// let environment = Environment::new(env_name, provider_config, ssh_credentials, 22, created_at);
///
/// // Access the context through the environment
/// let context = environment.context();
/// // Context holds all state-independent data for the environment
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// Timestamp when the environment was created
    ///
    /// This field records the exact moment when the environment was first created
    /// using the `create environment` command. It never changes throughout the
    /// environment lifecycle.
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,

    /// User-provided configuration
    pub user_inputs: UserInputs,

    /// Internal paths and derived configuration
    pub internal_config: InternalConfig,

    /// Runtime outputs from deployment operations
    pub runtime_outputs: RuntimeOutputs,
}

impl EnvironmentContext {
    /// Creates a new `EnvironmentContext` with auto-generated names and paths
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `provider_config` - Provider-specific configuration (LXD, Hetzner, etc.)
    /// * `ssh_credentials` - SSH credentials for connecting to instances
    /// * `ssh_port` - SSH port for connecting to instances
    ///
    /// # Returns
    ///
    /// A new `EnvironmentContext` with:
    /// - Auto-generated instance name: `torrust-tracker-vm-{env_name}`
    /// - Provider configuration with validated settings
    /// - Auto-generated data and build directories
    /// - Empty runtime outputs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::{EnvironmentContext, EnvironmentName};
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
    /// use std::path::PathBuf;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let provider_config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new("torrust-profile-production".to_string())?,
    /// });
    ///
    /// let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    /// let context = EnvironmentContext::new(&env_name, provider_config, ssh_credentials, 22, created_at);
    ///
    /// assert_eq!(context.user_inputs.instance_name.as_str(), "torrust-tracker-vm-production");
    /// let lxd_config = context.user_inputs.provider_config().as_lxd().unwrap();
    /// assert_eq!(lxd_config.profile_name.as_str(), "torrust-profile-production");
    /// assert_eq!(context.internal_config.data_dir, PathBuf::from("./data/production"));
    /// assert_eq!(context.internal_config.build_dir, PathBuf::from("./build/production"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Panics
    ///
    /// This function does not panic. All name generation is guaranteed to succeed
    /// for valid environment names.
    #[must_use]
    pub fn new(
        name: &EnvironmentName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            created_at,
            user_inputs: UserInputs::new(name, provider_config, ssh_credentials, ssh_port),
            internal_config: InternalConfig::new(name),
            runtime_outputs: RuntimeOutputs {
                instance_ip: None,
                provision_method: None,
            },
        }
    }

    /// Creates a new environment context with custom tracker configuration
    ///
    /// This creates absolute paths for data and build directories by using the
    /// provided working directory as the base, and allows specifying custom
    /// tracker, prometheus, and grafana configurations.
    #[must_use]
    #[allow(clippy::too_many_arguments)] // Public API with necessary configuration parameters
    pub fn with_working_dir_and_tracker(
        name: &EnvironmentName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        tracker_config: crate::domain::tracker::TrackerConfig,
        prometheus_config: Option<crate::domain::prometheus::PrometheusConfig>,
        grafana_config: Option<crate::domain::grafana::GrafanaConfig>,
        working_dir: &std::path::Path,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            created_at,
            user_inputs: UserInputs::with_tracker(
                name,
                provider_config,
                ssh_credentials,
                ssh_port,
                tracker_config,
                prometheus_config,
                grafana_config,
            ),
            internal_config: InternalConfig::with_working_dir(name, working_dir),
            runtime_outputs: RuntimeOutputs {
                instance_ip: None,
                provision_method: None,
            },
        }
    }

    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &crate::shared::Username {
        &self.user_inputs.ssh_credentials.ssh_username
    }

    /// Returns the SSH private key path for this environment
    #[must_use]
    pub fn ssh_private_key_path(&self) -> &PathBuf {
        &self.user_inputs.ssh_credentials.ssh_priv_key_path
    }

    /// Returns the SSH public key path for this environment
    #[must_use]
    pub fn ssh_public_key_path(&self) -> &PathBuf {
        &self.user_inputs.ssh_credentials.ssh_pub_key_path
    }

    /// Returns the templates directory for this environment
    ///
    /// Path: `data/{env_name}/templates/`
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.internal_config.templates_dir()
    }

    /// Returns the traces directory for this environment
    ///
    /// Path: `data/{env_name}/traces/`
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.internal_config.traces_dir()
    }

    /// Returns the ansible build directory
    ///
    /// Path: `build/{env_name}/ansible`
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.internal_config.ansible_build_dir()
    }

    /// Returns the tofu build directory for the environment's provider
    ///
    /// Path: `build/{env_name}/tofu/{provider_name}`
    ///
    /// The provider is determined from the environment's provider
    /// configuration (e.g., LXD, Hetzner).
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        let provider = self.user_inputs.provider_config.provider();
        self.internal_config.tofu_build_dir_for_provider(provider)
    }

    /// Returns the ansible templates directory
    ///
    /// Path: `data/{env_name}/templates/ansible`
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.internal_config.ansible_templates_dir()
    }

    /// Returns the tofu templates directory
    ///
    /// Path: `data/{env_name}/templates/tofu`
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.internal_config.tofu_templates_dir()
    }

    /// Returns the environment name
    #[must_use]
    pub fn name(&self) -> &EnvironmentName {
        &self.user_inputs.name
    }

    /// Returns the instance name
    #[must_use]
    pub fn instance_name(&self) -> &crate::domain::InstanceName {
        &self.user_inputs.instance_name
    }

    /// Returns the provider configuration
    #[must_use]
    pub fn provider_config(&self) -> &ProviderConfig {
        &self.user_inputs.provider_config
    }

    /// Returns the SSH credentials
    #[must_use]
    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.user_inputs.ssh_credentials
    }

    /// Returns the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.user_inputs.ssh_port
    }

    /// Returns the database configuration
    #[must_use]
    pub fn database_config(&self) -> &crate::domain::tracker::DatabaseConfig {
        &self.user_inputs.tracker.core.database
    }

    /// Returns the tracker configuration
    #[must_use]
    pub fn tracker_config(&self) -> &crate::domain::tracker::TrackerConfig {
        &self.user_inputs.tracker
    }

    /// Returns the admin token
    #[must_use]
    pub fn admin_token(&self) -> &str {
        self.user_inputs
            .tracker
            .http_api
            .admin_token
            .expose_secret()
    }

    /// Returns the Prometheus configuration if enabled
    #[must_use]
    pub fn prometheus_config(&self) -> Option<&PrometheusConfig> {
        self.user_inputs.prometheus.as_ref()
    }

    /// Returns the Grafana configuration if enabled
    #[must_use]
    pub fn grafana_config(&self) -> Option<&GrafanaConfig> {
        self.user_inputs.grafana.as_ref()
    }

    /// Returns the build directory
    #[must_use]
    pub fn build_dir(&self) -> &PathBuf {
        &self.internal_config.build_dir
    }

    /// Returns the data directory
    #[must_use]
    pub fn data_dir(&self) -> &PathBuf {
        &self.internal_config.data_dir
    }

    /// Returns the instance IP address if available
    #[must_use]
    pub fn instance_ip(&self) -> Option<std::net::IpAddr> {
        self.runtime_outputs.instance_ip
    }

    /// Returns the provision method
    #[must_use]
    pub fn provision_method(&self) -> Option<crate::domain::environment::ProvisionMethod> {
        self.runtime_outputs.provision_method
    }

    /// Returns the creation timestamp
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
