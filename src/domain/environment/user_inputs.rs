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

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::adapters::ssh::SshCredentials;
use crate::domain::environment::EnvironmentName;
use crate::domain::grafana::GrafanaConfig;
use crate::domain::https::HttpsConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::{Provider, ProviderConfig};
use crate::domain::tracker::TrackerConfig;
use crate::domain::InstanceName;

/// Errors for user inputs validation
///
/// These errors represent cross-service invariant violations that can only be
/// detected when considering multiple service configurations together.
#[derive(Debug, Clone, PartialEq)]
pub enum UserInputsError {
    /// Grafana requires Prometheus to be configured as its data source
    GrafanaRequiresPrometheus,

    /// HTTPS section is defined but no service has TLS configured
    HttpsSectionWithoutTlsServices,

    /// At least one service has TLS configured but HTTPS section is missing
    TlsServicesWithoutHttpsSection,
}

impl fmt::Display for UserInputsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrafanaRequiresPrometheus => {
                write!(
                    f,
                    "Grafana requires Prometheus to be configured as its data source"
                )
            }
            Self::HttpsSectionWithoutTlsServices => {
                write!(
                    f,
                    "HTTPS section is defined but no service has TLS configured"
                )
            }
            Self::TlsServicesWithoutHttpsSection => {
                write!(
                    f,
                    "At least one service has TLS configured but HTTPS section is missing"
                )
            }
        }
    }
}

impl std::error::Error for UserInputsError {}

impl UserInputsError {
    /// Provides actionable help text for fixing the error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::GrafanaRequiresPrometheus => {
                "Add a 'prometheus' section to your configuration, or remove the 'grafana' section. \
                Grafana needs Prometheus as its metrics data source."
            }
            Self::HttpsSectionWithoutTlsServices => {
                "Either remove the 'https' section, or set 'use_tls_proxy: true' on at least one \
                service (http_api, http_trackers, or health_check_api)."
            }
            Self::TlsServicesWithoutHttpsSection => {
                "Add an 'https' section with 'admin_email' for Let's Encrypt certificate management. \
                Services with 'use_tls_proxy: true' require Caddy for TLS termination."
            }
        }
    }
}

/// User-provided configuration when creating an environment
///
/// This struct contains all fields that are provided by the user when creating
/// an environment. These fields are immutable throughout the environment lifecycle
/// and represent the user's configuration choices.
///
/// # Cross-Service Invariants
///
/// The following invariants are validated at construction time:
/// - **Grafana requires Prometheus**: If Grafana is enabled, Prometheus must also be enabled
/// - **HTTPS requires TLS services**: If HTTPS section is present, at least one service must have TLS
/// - **TLS requires HTTPS**: If any service has TLS, HTTPS section must be present
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::{InstanceName, EnvironmentName, ProfileName};
/// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig};
/// use torrust_tracker_deployer_lib::domain::environment::user_inputs::UserInputs;
/// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
/// use torrust_tracker_deployer_lib::domain::prometheus::PrometheusConfig;
/// use torrust_tracker_deployer_lib::domain::grafana::GrafanaConfig;
/// use torrust_tracker_deployer_lib::shared::Username;
/// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
/// use std::path::PathBuf;
///
/// let provider_config = ProviderConfig::Lxd(LxdConfig {
///     profile_name: ProfileName::new("torrust-profile-production".to_string())?,
/// });
/// let ssh_credentials = SshCredentials::new(
///     PathBuf::from("keys/prod_rsa"),
///     PathBuf::from("keys/prod_rsa.pub"),
///     Username::new("torrust".to_string())?,
/// );
/// let env_name = EnvironmentName::new("production".to_string())?;
///
/// // Create with defaults (includes Prometheus and Grafana)
/// let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22)?;
///
/// assert_eq!(user_inputs.name().as_str(), "production");
/// assert!(user_inputs.prometheus().is_some());
/// assert!(user_inputs.grafana().is_some());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputs {
    /// The validated environment name
    name: EnvironmentName,

    /// The instance name for this environment (auto-generated from name)
    instance_name: InstanceName,

    /// Provider-specific configuration (e.g., LXD profile, Hetzner settings)
    provider_config: ProviderConfig,

    /// SSH credentials for connecting to instances in this environment
    ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    ssh_port: u16,

    /// Tracker deployment configuration
    tracker: TrackerConfig,

    /// Prometheus metrics collection configuration (optional)
    ///
    /// When present, Prometheus service is enabled in the deployment.
    /// When absent (`None`), Prometheus service is disabled.
    /// Default: `Some(PrometheusConfig::default())` in generated templates.
    prometheus: Option<PrometheusConfig>,

    /// Grafana visualization and dashboard configuration (optional)
    ///
    /// When present, Grafana service is enabled in the deployment.
    /// When absent (`None`), Grafana service is disabled.
    /// Requires Prometheus to be enabled - dependency validated at construction time.
    /// Default: `Some(GrafanaConfig::default())` in generated templates.
    grafana: Option<GrafanaConfig>,

    /// HTTPS/TLS configuration for Caddy reverse proxy (optional)
    ///
    /// When present, Caddy service is deployed as a TLS termination proxy.
    /// When absent (`None`), services are exposed directly over HTTP.
    /// Requires at least one service to have TLS configuration.
    https: Option<HttpsConfig>,
}

impl UserInputs {
    /// Creates a new `UserInputs` with auto-generated instance name and default services
    ///
    /// Creates a `UserInputs` with default tracker configuration, Prometheus, and Grafana
    /// enabled. This is the standard setup for most deployments.
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
    /// - Default tracker configuration
    /// - Prometheus and Grafana enabled (satisfies cross-service invariants)
    ///
    /// # Errors
    ///
    /// This constructor with defaults cannot fail because the default configuration
    /// (Prometheus + Grafana, no HTTPS) always satisfies cross-service invariants.
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
    /// let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22)?;
    ///
    /// assert_eq!(user_inputs.instance_name().as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(user_inputs.provider(), Provider::Lxd);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        name: &EnvironmentName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
    ) -> Result<Self, UserInputsError> {
        // Default configuration: Prometheus + Grafana, no HTTPS
        // This always passes validation (Grafana has Prometheus, no TLS configured)
        Self::with_tracker(
            name,
            provider_config,
            ssh_credentials,
            ssh_port,
            TrackerConfig::default(),
            Some(PrometheusConfig::default()),
            Some(GrafanaConfig::default()),
            None,
        )
    }

    /// Creates a new `UserInputs` with custom tracker and service configuration
    ///
    /// This constructor allows full control over all service configurations.
    /// Cross-service invariants are validated at construction time.
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `provider_config` - Provider-specific configuration
    /// * `ssh_credentials` - SSH credentials for connecting to instances
    /// * `ssh_port` - SSH port for connecting to instances
    /// * `tracker` - Tracker deployment configuration
    /// * `prometheus` - Optional Prometheus configuration
    /// * `grafana` - Optional Grafana configuration (requires Prometheus)
    /// * `https` - Optional HTTPS/TLS configuration (requires TLS services)
    ///
    /// # Errors
    ///
    /// - `GrafanaRequiresPrometheus` if Grafana is configured without Prometheus
    /// - `HttpsSectionWithoutTlsServices` if HTTPS section exists but no service uses TLS
    /// - `TlsServicesWithoutHttpsSection` if a service uses TLS but HTTPS section is missing
    #[allow(clippy::too_many_arguments)]
    pub fn with_tracker(
        name: &EnvironmentName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        tracker: TrackerConfig,
        prometheus: Option<PrometheusConfig>,
        grafana: Option<GrafanaConfig>,
        https: Option<HttpsConfig>,
    ) -> Result<Self, UserInputsError> {
        // Cross-service invariant: Grafana requires Prometheus as data source
        if grafana.is_some() && prometheus.is_none() {
            return Err(UserInputsError::GrafanaRequiresPrometheus);
        }

        // Cross-service invariant: HTTPS section requires at least one TLS service
        let has_tls = tracker.has_any_tls_configured();
        if https.is_some() && !has_tls {
            return Err(UserInputsError::HttpsSectionWithoutTlsServices);
        }

        // Inverse: TLS services require HTTPS section
        if has_tls && https.is_none() {
            return Err(UserInputsError::TlsServicesWithoutHttpsSection);
        }

        let instance_name = Self::generate_instance_name(name);

        Ok(Self {
            name: name.clone(),
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker,
            prometheus,
            grafana,
            https,
        })
    }

    // ========================================================================
    // Getter Methods
    // ========================================================================

    /// Returns the environment name
    #[must_use]
    pub fn name(&self) -> &EnvironmentName {
        &self.name
    }

    /// Returns the instance name
    #[must_use]
    pub fn instance_name(&self) -> &InstanceName {
        &self.instance_name
    }

    /// Returns the SSH credentials
    #[must_use]
    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.ssh_credentials
    }

    /// Returns the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    /// Returns the tracker configuration
    #[must_use]
    pub fn tracker(&self) -> &TrackerConfig {
        &self.tracker
    }

    /// Returns the Prometheus configuration if enabled
    #[must_use]
    pub fn prometheus(&self) -> Option<&PrometheusConfig> {
        self.prometheus.as_ref()
    }

    /// Returns the Grafana configuration if enabled
    #[must_use]
    pub fn grafana(&self) -> Option<&GrafanaConfig> {
        self.grafana.as_ref()
    }

    /// Returns the HTTPS configuration if enabled
    #[must_use]
    pub fn https(&self) -> Option<&HttpsConfig> {
        self.https.as_ref()
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
    /// let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22)?;
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
    use std::path::PathBuf;

    use super::*;
    use crate::domain::provider::LxdConfig;
    use crate::domain::tracker::{
        DatabaseConfig, HealthCheckApiConfig, HttpApiConfig, SqliteConfig, TrackerCoreConfig,
        UdpTrackerConfig,
    };
    use crate::domain::ProfileName;
    use crate::shared::{ApiToken, DomainName, Username};

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

    fn create_test_env_name() -> EnvironmentName {
        EnvironmentName::new("test-env".to_string()).unwrap()
    }

    fn create_tracker_config_with_tls() -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "token".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true, // TLS enabled
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    fn create_tracker_config_without_tls() -> TrackerConfig {
        TrackerConfig::default()
    }

    #[test]
    fn it_should_create_user_inputs_with_lxd_provider() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22).unwrap();

        assert_eq!(user_inputs.name().as_str(), "test-env");
        assert_eq!(
            user_inputs.instance_name().as_str(),
            "torrust-tracker-vm-test-env"
        );
        assert_eq!(user_inputs.provider(), Provider::Lxd);
        assert_eq!(user_inputs.provider_config().provider_name(), "lxd");
        assert_eq!(user_inputs.ssh_port(), 22);
    }

    #[test]
    fn it_should_return_provider_config_for_lxd() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("my-custom-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22).unwrap();

        let lxd_config = user_inputs.provider_config().as_lxd().unwrap();
        assert_eq!(lxd_config.profile_name.as_str(), "my-custom-profile");
    }

    #[test]
    fn it_should_return_provider_config_for_hetzner() {
        use crate::domain::provider::HetznerConfig;

        let env_name = create_test_env_name();
        let provider_config = ProviderConfig::Hetzner(HetznerConfig {
            api_token: ApiToken::from("test-token"),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        });
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22).unwrap();

        assert_eq!(user_inputs.provider(), Provider::Hetzner);
        assert!(user_inputs.provider_config().as_lxd().is_none());

        let hetzner_config = user_inputs.provider_config().as_hetzner().unwrap();
        assert_eq!(hetzner_config.api_token.expose_secret(), "test-token");
        assert_eq!(hetzner_config.server_type, "cx22");
        assert_eq!(hetzner_config.location, "nbg1");
        assert_eq!(hetzner_config.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_auto_generate_instance_name_from_environment_name() {
        let env_name = EnvironmentName::new("production".to_string()).unwrap();
        let provider_config = create_lxd_provider_config("prod-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let user_inputs = UserInputs::new(&env_name, provider_config, ssh_credentials, 22).unwrap();

        assert_eq!(
            user_inputs.instance_name().as_str(),
            "torrust-tracker-vm-production"
        );
    }

    // ========================================================================
    // Cross-Service Invariant Tests
    // ========================================================================

    #[test]
    fn it_should_reject_grafana_without_prometheus() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_without_tls(),
            None,                           // No Prometheus
            Some(GrafanaConfig::default()), // Grafana enabled
            None,
        );

        assert!(
            matches!(result, Err(UserInputsError::GrafanaRequiresPrometheus)),
            "Expected GrafanaRequiresPrometheus error, got {result:?}"
        );
    }

    #[test]
    fn it_should_accept_grafana_with_prometheus() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_without_tls(),
            Some(PrometheusConfig::default()), // Prometheus enabled
            Some(GrafanaConfig::default()),    // Grafana enabled
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_reject_https_section_without_tls_services() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_without_tls(), // No TLS on any service
            Some(PrometheusConfig::default()),
            Some(GrafanaConfig::default()),
            Some(HttpsConfig::new("admin@example.com", false)), // HTTPS section present
        );

        assert!(
            matches!(result, Err(UserInputsError::HttpsSectionWithoutTlsServices)),
            "Expected HttpsSectionWithoutTlsServices error, got {result:?}"
        );
    }

    #[test]
    fn it_should_reject_tls_services_without_https_section() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_with_tls(), // Has TLS on HTTP API
            Some(PrometheusConfig::default()),
            Some(GrafanaConfig::default()),
            None, // No HTTPS section
        );

        assert!(
            matches!(result, Err(UserInputsError::TlsServicesWithoutHttpsSection)),
            "Expected TlsServicesWithoutHttpsSection error, got {result:?}"
        );
    }

    #[test]
    fn it_should_accept_tls_services_with_https_section() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_with_tls(),
            Some(PrometheusConfig::default()),
            Some(GrafanaConfig::default()),
            Some(HttpsConfig::new("admin@example.com", false)),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_accept_no_tls_and_no_https() {
        let env_name = create_test_env_name();
        let provider_config = create_lxd_provider_config("test-profile");
        let ssh_credentials = create_test_ssh_credentials();

        let result = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            create_tracker_config_without_tls(),
            Some(PrometheusConfig::default()),
            Some(GrafanaConfig::default()),
            None, // No HTTPS
        );

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_provide_helpful_error_messages() {
        assert_eq!(
            UserInputsError::GrafanaRequiresPrometheus.to_string(),
            "Grafana requires Prometheus to be configured as its data source"
        );
        assert!(UserInputsError::GrafanaRequiresPrometheus
            .help()
            .contains("prometheus"));

        assert!(UserInputsError::HttpsSectionWithoutTlsServices
            .help()
            .contains("use_tls_proxy"));

        assert!(UserInputsError::TlsServicesWithoutHttpsSection
            .help()
            .contains("https"));
    }
}
