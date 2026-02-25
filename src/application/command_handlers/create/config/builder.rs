//! Fluent builder for [`EnvironmentCreationConfig`].
//!
//! Provides a type-safe, ergonomic way to construct deployment configurations
//! without hand-crafting JSON strings.

use thiserror::Error;

use super::environment_config::{EnvironmentCreationConfig, EnvironmentSection};
use super::provider::{HetznerProviderSection, LxdProviderSection, ProviderSection};
use super::ssh_credentials_config::SshCredentialsConfig;
use super::tracker::{
    DatabaseSection, HealthCheckApiSection, HttpApiSection, HttpTrackerSection, TrackerCoreSection,
    TrackerSection, UdpTrackerSection,
};

/// Default health-check API bind address used when none is set.
const DEFAULT_HEALTH_CHECK_BIND: &str = "127.0.0.1:1313";

/// Errors that can occur when building an [`EnvironmentCreationConfig`].
#[derive(Debug, Error)]
pub enum EnvironmentCreationConfigBuildError {
    /// No environment name was set.
    #[error("missing required field: name — call .name(\"my-env\")")]
    MissingName,

    /// No SSH private key path was set.
    #[error(
        "missing required field: SSH private key path — call .ssh_keys(private_path, public_path)"
    )]
    MissingPrivateKey,

    /// No SSH public key path was set.
    #[error(
        "missing required field: SSH public key path — call .ssh_keys(private_path, public_path)"
    )]
    MissingPublicKey,

    /// No provider was set.
    #[error(
        "missing required field: provider — call .provider_lxd(profile) or .provider_hetzner(…)"
    )]
    MissingProvider,

    /// No database driver was set.
    #[error("missing required field: database — call .sqlite(db_name) or .mysql(…)")]
    MissingDatabase,

    /// No HTTP API was set.
    #[error("missing required field: HTTP API — call .api(bind_address, admin_token)")]
    MissingApi,
}

/// Fluent builder for [`EnvironmentCreationConfig`].
///
/// Construct it via [`EnvironmentCreationConfig::builder`] or
/// [`EnvironmentCreationConfigBuilder::new`].
///
/// # Required fields
///
/// | Method | Sets |
/// |--------|------|
/// | [`name`](Self::name) | environment name |
/// | [`ssh_keys`](Self::ssh_keys) | private & public key paths |
/// | [`provider_lxd`](Self::provider_lxd) / [`provider_hetzner`](Self::provider_hetzner) | VM provider |
/// | [`sqlite`](Self::sqlite) / [`mysql`](Self::mysql) | tracker database |
/// | [`api`](Self::api) | HTTP management API |
///
/// # Optional fields
///
/// | Method | Default |
/// |--------|---------|
/// | [`udp`](Self::udp) | none (call once per listener) |
/// | [`http`](Self::http) | none (call once per listener) |
/// | [`ssh_username`](Self::ssh_username) | `"torrust"` |
/// | [`ssh_port`](Self::ssh_port) | `22` |
/// | [`private`](Self::private) | `false` (public tracker) |
/// | [`health_check`](Self::health_check) | `"127.0.0.1:1313"` |
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
///
/// let config = EnvironmentCreationConfig::builder()
///     .name("my-tracker")
///     .ssh_keys("/path/to/key", "/path/to/key.pub")
///     .provider_lxd("torrust-profile")
///     .sqlite("tracker.db")
///     .udp("0.0.0.0:6969")
///     .http("0.0.0.0:7070")
///     .api("0.0.0.0:1212", "MyToken")
///     .build()
///     .expect("Failed to build configuration");
/// ```
#[derive(Debug, Default)]
pub struct EnvironmentCreationConfigBuilder {
    name: Option<String>,
    ssh_private_key: Option<String>,
    ssh_public_key: Option<String>,
    ssh_username: Option<String>,
    ssh_port: Option<u16>,
    provider: Option<ProviderSection>,
    database: Option<DatabaseSection>,
    private_tracker: bool,
    udp_trackers: Vec<UdpTrackerSection>,
    http_trackers: Vec<HttpTrackerSection>,
    api_bind_address: Option<String>,
    api_admin_token: Option<String>,
    health_check_bind_address: Option<String>,
}

impl EnvironmentCreationConfigBuilder {
    /// Create a new empty builder.
    ///
    /// Prefer [`EnvironmentCreationConfig::builder`] at call sites.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the environment name (required).
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the SSH key pair paths (required).
    #[must_use]
    pub fn ssh_keys(
        mut self,
        private_key_path: impl Into<String>,
        public_key_path: impl Into<String>,
    ) -> Self {
        self.ssh_private_key = Some(private_key_path.into());
        self.ssh_public_key = Some(public_key_path.into());
        self
    }

    /// Override the SSH username (optional, default: `"torrust"`).
    #[must_use]
    pub fn ssh_username(mut self, username: impl Into<String>) -> Self {
        self.ssh_username = Some(username.into());
        self
    }

    /// Override the SSH port (optional, default: `22`).
    #[must_use]
    pub fn ssh_port(mut self, port: u16) -> Self {
        self.ssh_port = Some(port);
        self
    }

    /// Use the LXD provider (required unless `provider_hetzner` is called).
    #[must_use]
    pub fn provider_lxd(mut self, profile_name: impl Into<String>) -> Self {
        self.provider = Some(ProviderSection::Lxd(LxdProviderSection {
            profile_name: profile_name.into(),
        }));
        self
    }

    /// Use the Hetzner provider (required unless `provider_lxd` is called).
    #[must_use]
    pub fn provider_hetzner(
        mut self,
        api_token: impl Into<String>,
        server_type: impl Into<String>,
        location: impl Into<String>,
        image: impl Into<String>,
    ) -> Self {
        self.provider = Some(ProviderSection::Hetzner(HetznerProviderSection {
            api_token: api_token.into(),
            server_type: server_type.into(),
            location: location.into(),
            image: image.into(),
        }));
        self
    }

    /// Use `SQLite` as the tracker database (required unless `mysql` is called).
    #[must_use]
    pub fn sqlite(mut self, database_name: impl Into<String>) -> Self {
        self.database = Some(DatabaseSection::Sqlite {
            database_name: database_name.into(),
        });
        self
    }

    /// Use `MySQL` as the tracker database (required unless `sqlite` is called).
    #[must_use]
    pub fn mysql(
        mut self,
        host: impl Into<String>,
        port: u16,
        database_name: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.database = Some(DatabaseSection::Mysql {
            host: host.into(),
            port,
            database_name: database_name.into(),
            username: username.into(),
            password: password.into(),
        });
        self
    }

    /// Set the tracker privacy mode (optional, default: `false` = public).
    #[must_use]
    pub fn private(mut self, private: bool) -> Self {
        self.private_tracker = private;
        self
    }

    /// Add a UDP tracker listener (optional, repeatable).
    #[must_use]
    pub fn udp(mut self, bind_address: impl Into<String>) -> Self {
        self.udp_trackers.push(UdpTrackerSection {
            bind_address: bind_address.into(),
            domain: None,
        });
        self
    }

    /// Add an HTTP tracker listener (optional, repeatable).
    #[must_use]
    pub fn http(mut self, bind_address: impl Into<String>) -> Self {
        self.http_trackers.push(HttpTrackerSection {
            bind_address: bind_address.into(),
            domain: None,
            use_tls_proxy: None,
        });
        self
    }

    /// Set the HTTP management API bind address and admin token (required).
    #[must_use]
    pub fn api(mut self, bind_address: impl Into<String>, admin_token: impl Into<String>) -> Self {
        self.api_bind_address = Some(bind_address.into());
        self.api_admin_token = Some(admin_token.into());
        self
    }

    /// Override the health-check API bind address (optional, default: `"127.0.0.1:1313"`).
    #[must_use]
    pub fn health_check(mut self, bind_address: impl Into<String>) -> Self {
        self.health_check_bind_address = Some(bind_address.into());
        self
    }

    /// Build the [`EnvironmentCreationConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentCreationConfigBuildError`] when any required field is missing.
    pub fn build(self) -> Result<EnvironmentCreationConfig, EnvironmentCreationConfigBuildError> {
        let name = self
            .name
            .ok_or(EnvironmentCreationConfigBuildError::MissingName)?;
        let private_key_path = self
            .ssh_private_key
            .ok_or(EnvironmentCreationConfigBuildError::MissingPrivateKey)?;
        let public_key_path = self
            .ssh_public_key
            .ok_or(EnvironmentCreationConfigBuildError::MissingPublicKey)?;
        let provider = self
            .provider
            .ok_or(EnvironmentCreationConfigBuildError::MissingProvider)?;
        let database = self
            .database
            .ok_or(EnvironmentCreationConfigBuildError::MissingDatabase)?;
        let api_bind_address = self
            .api_bind_address
            .ok_or(EnvironmentCreationConfigBuildError::MissingApi)?;
        let api_admin_token = self
            .api_admin_token
            .ok_or(EnvironmentCreationConfigBuildError::MissingApi)?;

        let ssh_credentials = SshCredentialsConfig {
            private_key_path,
            public_key_path,
            username: self.ssh_username.unwrap_or_else(|| "torrust".to_string()),
            port: self.ssh_port.unwrap_or(22),
        };

        let tracker = TrackerSection {
            core: TrackerCoreSection {
                database,
                private: self.private_tracker,
            },
            udp_trackers: self.udp_trackers,
            http_trackers: self.http_trackers,
            http_api: HttpApiSection {
                bind_address: api_bind_address,
                admin_token: api_admin_token,
                domain: None,
                use_tls_proxy: None,
            },
            health_check_api: HealthCheckApiSection {
                bind_address: self
                    .health_check_bind_address
                    .unwrap_or_else(|| DEFAULT_HEALTH_CHECK_BIND.to_string()),
                domain: None,
                use_tls_proxy: None,
            },
        };

        Ok(EnvironmentCreationConfig {
            environment: EnvironmentSection {
                name,
                description: None,
                instance_name: None,
            },
            ssh_credentials,
            provider,
            tracker,
            prometheus: None,
            grafana: None,
            https: None,
            backup: None,
        })
    }
}
