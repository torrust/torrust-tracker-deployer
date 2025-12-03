//! Provider Configuration Domain Types
//!
//! This module contains the `ProviderConfig` enum that aggregates all
//! provider-specific configurations. Individual provider configurations
//! are defined in their own modules (`lxd`, `hetzner`).
//!
//! These types use validated domain types (like `ProfileName`) and represent
//! the semantic meaning of provider configuration.
//!
//! For config types used in JSON deserialization, see
//! `application::command_handlers::create::config::provider`.
//!
//! # Layer Separation
//!
//! - **Domain types** (this module): `ProviderConfig`, `LxdConfig`, `HetznerConfig`
//!   - Use validated domain types (e.g., `ProfileName`)
//!   - Represent semantic meaning of configuration
//!
//! - **Application config types** (`application::command_handlers::create::config::provider`):
//!   - `ProviderSection`, `LxdProviderSection`, `HetznerProviderSection`
//!   - Use raw primitives (e.g., `String`)
//!   - Handle JSON deserialization and conversion to domain types

use serde::{Deserialize, Serialize};

use super::hetzner::HetznerConfig;
use super::lxd::LxdConfig;
use super::Provider;

/// Provider-specific configuration (Domain Type)
///
/// Each variant contains the configuration fields specific to that provider
/// using **validated domain types** (e.g., `ProfileName` instead of `String`).
///
/// This is a tagged enum that serializes/deserializes based on the `"provider"` field.
///
/// # Note on Layer Placement
///
/// This is a **domain type** with validated fields. For JSON deserialization,
/// use `ProviderSection` in the application layer, then convert to this type.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig, Provider};
/// use torrust_tracker_deployer_lib::domain::ProfileName;
///
/// let lxd_config = ProviderConfig::Lxd(LxdConfig {
///     profile_name: ProfileName::new("torrust-profile").unwrap(),
/// });
///
/// assert_eq!(lxd_config.provider(), Provider::Lxd);
/// assert_eq!(lxd_config.provider_name(), "lxd");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    /// LXD provider configuration
    #[serde(rename = "lxd")]
    Lxd(LxdConfig),

    /// Hetzner provider configuration
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig),
}

impl ProviderConfig {
    /// Returns the provider type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig, Provider};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    ///
    /// let config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new("test").unwrap(),
    /// });
    /// assert_eq!(config.provider(), Provider::Lxd);
    /// ```
    #[must_use]
    pub fn provider(&self) -> Provider {
        match self {
            Self::Lxd(_) => Provider::Lxd,
            Self::Hetzner(_) => Provider::Hetzner,
        }
    }

    /// Returns the provider name as used in directory paths.
    ///
    /// This is a convenience method that delegates to `self.provider().as_str()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    ///
    /// let config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new("test").unwrap(),
    /// });
    /// assert_eq!(config.provider_name(), "lxd");
    /// ```
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.provider().as_str()
    }

    /// Returns a reference to the LXD configuration if this is an LXD provider.
    ///
    /// # Returns
    ///
    /// - `Some(&LxdConfig)` if the provider is LXD
    /// - `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::provider::{ProviderConfig, LxdConfig, HetznerConfig};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    ///
    /// let lxd_config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new("test").unwrap(),
    /// });
    /// assert!(lxd_config.as_lxd().is_some());
    ///
    /// let hetzner_config = ProviderConfig::Hetzner(HetznerConfig {
    ///     api_token: "token".to_string(),
    ///     server_type: "cx22".to_string(),
    ///     location: "nbg1".to_string(),
    ///     image: "ubuntu-24.04".to_string(),
    /// });
    /// assert!(hetzner_config.as_lxd().is_none());
    /// ```
    #[must_use]
    pub fn as_lxd(&self) -> Option<&LxdConfig> {
        match self {
            Self::Lxd(config) => Some(config),
            Self::Hetzner(_) => None,
        }
    }

    /// Returns a reference to the Hetzner configuration if this is a Hetzner provider.
    ///
    /// # Returns
    ///
    /// - `Some(&HetznerConfig)` if the provider is Hetzner
    /// - `None` otherwise
    #[must_use]
    pub fn as_hetzner(&self) -> Option<&HetznerConfig> {
        match self {
            Self::Lxd(_) => None,
            Self::Hetzner(config) => Some(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ProfileName;

    fn create_lxd_config() -> ProviderConfig {
        ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new("torrust-profile").unwrap(),
        })
    }

    fn create_hetzner_config() -> ProviderConfig {
        ProviderConfig::Hetzner(HetznerConfig {
            api_token: "test-token".to_string(),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        })
    }

    #[test]
    fn it_should_return_lxd_provider_when_lxd_config_queried() {
        let config = create_lxd_config();
        assert_eq!(config.provider(), Provider::Lxd);
        assert_eq!(config.provider_name(), "lxd");
    }

    #[test]
    fn it_should_return_hetzner_provider_when_hetzner_config_queried() {
        let config = create_hetzner_config();
        assert_eq!(config.provider(), Provider::Hetzner);
        assert_eq!(config.provider_name(), "hetzner");
    }

    #[test]
    fn it_should_return_some_lxd_config_when_as_lxd_called_on_lxd_variant() {
        let config = create_lxd_config();
        assert!(config.as_lxd().is_some());
        assert!(config.as_hetzner().is_none());
    }

    #[test]
    fn it_should_return_some_hetzner_config_when_as_hetzner_called_on_hetzner_variant() {
        let config = create_hetzner_config();
        assert!(config.as_hetzner().is_some());
        assert!(config.as_lxd().is_none());
    }

    #[test]
    fn it_should_serialize_lxd_config_to_json_with_provider_tag() {
        let config = create_lxd_config();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"provider\":\"lxd\""));
        assert!(json.contains("\"profile_name\":\"torrust-profile\""));
    }

    #[test]
    fn it_should_serialize_hetzner_config_to_json_with_provider_tag() {
        let config = create_hetzner_config();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"provider\":\"hetzner\""));
        assert!(json.contains("\"api_token\":\"test-token\""));
        assert!(json.contains("\"server_type\":\"cx22\""));
        assert!(json.contains("\"location\":\"nbg1\""));
    }

    #[test]
    fn it_should_deserialize_lxd_config_from_json_with_provider_tag() {
        let json = r#"{"provider":"lxd","profile_name":"torrust-profile"}"#;
        let config: ProviderConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.provider(), Provider::Lxd);
        assert_eq!(
            config.as_lxd().unwrap().profile_name.as_str(),
            "torrust-profile"
        );
    }

    #[test]
    fn it_should_deserialize_hetzner_config_from_json_with_provider_tag() {
        let json = r#"{"provider":"hetzner","api_token":"token","server_type":"cx22","location":"nbg1","image":"ubuntu-24.04"}"#;
        let config: ProviderConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.provider(), Provider::Hetzner);
        let hetzner = config.as_hetzner().unwrap();
        assert_eq!(hetzner.api_token, "token");
        assert_eq!(hetzner.server_type, "cx22");
        assert_eq!(hetzner.location, "nbg1");
        assert_eq!(hetzner.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_be_cloneable_when_cloned() {
        let config = create_lxd_config();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait_when_formatted() {
        let config = create_lxd_config();
        let debug = format!("{config:?}");
        assert!(debug.contains("Lxd"));
        assert!(debug.contains("profile_name"));
    }
}
