//! User Inputs Module
//!
//! This module contains the `UserInputs` struct which holds all user-provided
//! configuration when creating an environment.
//!
//! ## Purpose
//!
//! User inputs represent the immutable configuration choices made by the user
//! when creating an environment. These fields never change throughout the
//! environment's lifecycle.
//!
//! ## Semantic Category
//!
//! **User Inputs** are:
//! - Provided by the user when creating an environment
//! - Immutable throughout environment lifecycle
//! - Examples: name, SSH credentials, port numbers
//!
//! Add new fields here when: User needs to configure something at environment creation time.

use crate::adapters::ssh::SshCredentials;
use crate::domain::environment::EnvironmentName;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::{Provider, ProviderConfig};
use crate::domain::tracker::TrackerConfig;
use crate::domain::InstanceName;
use serde::{Deserialize, Serialize};

/// User-provided configuration when creating an environment
///
/// This struct contains all fields that are provided by the user when creating
/// an environment. These fields are immutable throughout the environment lifecycle
/// and represent the user's configuration choices.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::{InstanceName, EnvironmentName, ProfileName};
/// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig};
/// use torrust_tracker_deployer_lib::domain::environment::user_inputs::UserInputs;
/// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
/// use torrust_tracker_deployer_lib::domain::prometheus::PrometheusConfig;
/// use torrust_tracker_deployer_lib::shared::Username;
/// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
/// use std::path::PathBuf;
///
/// let provider_config = ProviderConfig::Lxd(LxdConfig {
///     profile_name: ProfileName::new("torrust-profile-production".to_string())?,
/// });
///
/// let user_inputs = UserInputs {
///     name: EnvironmentName::new("production".to_string())?,
///     instance_name: InstanceName::new("torrust-tracker-vm-production".to_string())?,
///     provider_config,
///     ssh_credentials: SshCredentials::new(
///         PathBuf::from("keys/prod_rsa"),
///         PathBuf::from("keys/prod_rsa.pub"),
///         Username::new("torrust".to_string())?,
///     ),
///     ssh_port: 22,
///     tracker: TrackerConfig::default(),
///     prometheus: Some(PrometheusConfig::default()),
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputs {
    /// The validated environment name
    pub name: EnvironmentName,

    /// The instance name for this environment (auto-generated from name)
    pub instance_name: InstanceName,

    /// Provider-specific configuration (e.g., LXD profile, Hetzner settings)
    pub provider_config: ProviderConfig,

    /// SSH credentials for connecting to instances in this environment
    pub ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    pub ssh_port: u16,

    /// Tracker deployment configuration
    pub tracker: TrackerConfig,

    /// Prometheus metrics collection configuration (optional)
    ///
    /// When present, Prometheus service is enabled in the deployment.
    /// When absent (`None`), Prometheus service is disabled.
    /// Default: `Some(PrometheusConfig::default())` in generated templates.
    pub prometheus: Option<PrometheusConfig>,
}

impl UserInputs {
    /// Creates a new `UserInputs` with auto-generated instance name
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `provider_config` - Provider-specific configuration
    /// * `ssh_credentials` - SSH credentials for connecting to instances
    /// * `ssh_port` - SSH port for connecting to instances
    ///
    /// # Returns
    ///
    /// A new `UserInputs` with:
    /// - Auto-generated instance name: `torrust-tracker-vm-{env_name}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::{EnvironmentName, UserInputs};
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig, Provider};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
    /// use std::path::PathBuf;
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
    /// let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);
    ///
    /// assert_eq!(user_inputs.instance_name.as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(user_inputs.provider(), Provider::Lxd);
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
    ) -> Self {
        let instance_name = Self::generate_instance_name(name);

        Self {
            name: name.clone(),
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker: TrackerConfig::default(),
            prometheus: Some(PrometheusConfig::default()),
        }
    }

    /// Creates a new `UserInputs` with custom tracker configuration
    ///
    /// This is similar to `new` but allows specifying a custom tracker
    /// configuration instead of using the default.
    #[must_use]
    pub fn with_tracker(
        name: &EnvironmentName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        tracker: TrackerConfig,
    ) -> Self {
        let instance_name = Self::generate_instance_name(name);

        Self {
            name: name.clone(),
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker,
            prometheus: Some(PrometheusConfig::default()),
        }
    }

    // ========================================================================
    // Provider Accessor Methods
    // ========================================================================

    /// Returns the provider type for this environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::{EnvironmentName, UserInputs};
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig, Provider};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("test".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/test_rsa"),
    ///     PathBuf::from("keys/test_rsa.pub"),
    ///     Username::new("torrust".to_string())?,
    /// );
    /// let provider_config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new("test-profile".to_string())?,
    /// });
    ///
    /// let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);
    /// assert_eq!(user_inputs.provider(), Provider::Lxd);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn provider(&self) -> Provider {
        self.provider_config.provider()
    }

    /// Returns a reference to the provider configuration
    ///
    /// Use this to access provider-specific fields. For example:
    /// ```rust,ignore
    /// if let Some(lxd_config) = user_inputs.provider_config().as_lxd() {
    ///     println!("LXD profile: {}", lxd_config.profile_name.as_str());
    /// }
    /// ```
    #[must_use]
    pub fn provider_config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

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
            .expect("Generated instance name should always be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::LxdConfig;
    use crate::domain::ProfileName;
    use crate::shared::Username;
    use std::path::PathBuf;

    fn create_test_ssh_credentials() -> SshCredentials {
        SshCredentials::new(
            PathBuf::from("keys/test_rsa"),
            PathBuf::from("keys/test_rsa.pub"),
            Username::new("testuser".to_string()).unwrap(),
        )
    }

    fn create_lxd_provider_config(profile_name: &str) -> ProviderConfig {
        ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new(profile_name.to_string()).unwrap(),
        })
    }

    #[test]
    fn it_should_create_user_inputs_with_lxd_provider() {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);

        assert_eq!(user_inputs.name.as_str(), "test-env");
        assert_eq!(
            user_inputs.instance_name.as_str(),
            "torrust-tracker-vm-test-env"
        );
        assert_eq!(user_inputs.provider(), Provider::Lxd);
        assert_eq!(user_inputs.provider_config().provider_name(), "lxd");
        assert_eq!(user_inputs.ssh_port, 22);
    }

    #[test]
    fn it_should_return_provider_config_for_lxd() {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let provider_config = create_lxd_provider_config("my-custom-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);

        let lxd_config = user_inputs.provider_config().as_lxd().unwrap();
        assert_eq!(lxd_config.profile_name.as_str(), "my-custom-profile");
    }

    #[test]
    fn it_should_return_provider_config_for_hetzner() {
        use crate::domain::provider::HetznerConfig;

        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let provider_config = ProviderConfig::Hetzner(HetznerConfig {
            api_token: "test-token".to_string(),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        });
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);

        assert_eq!(user_inputs.provider(), Provider::Hetzner);
        assert!(user_inputs.provider_config().as_lxd().is_none());

        let hetzner_config = user_inputs.provider_config().as_hetzner().unwrap();
        assert_eq!(hetzner_config.api_token, "test-token");
        assert_eq!(hetzner_config.server_type, "cx22");
        assert_eq!(hetzner_config.location, "nbg1");
        assert_eq!(hetzner_config.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_auto_generate_instance_name_from_environment_name() {
        let env_name = EnvironmentName::new("production".to_string()).unwrap();
        let provider_config = create_lxd_provider_config("prod-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22);

        assert_eq!(
            user_inputs.instance_name.as_str(),
            "torrust-tracker-vm-production"
        );
    }
}
