//! Provider Configuration Types (Application Layer)
//!
//! This module contains configuration types for provider-specific configuration.
//! These types are used for deserializing external configuration (JSON files) and
//! contain **raw primitives** (e.g., `String`).
//!
//! After deserialization, use `to_provider_config()` to convert to domain types
//! with validation.
//!
//! # Module Structure
//!
//! Each provider has its own submodule for extensibility:
//! - `lxd` - LXD provider configuration section
//! - `hetzner` - Hetzner provider configuration section
//!
//! # Layer Separation
//!
//! - **These config types** (this module): Raw primitives for JSON parsing
//! - **Domain types** (`domain::provider`): Validated types for business logic
//!
//! # Examples
//!
//! ```rust
//! use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
//!     ProviderSection, LxdProviderSection
//! };
//!
//! // Deserialize from JSON
//! let json = r#"{"provider": "lxd", "profile_name": "torrust-profile"}"#;
//! let section: ProviderSection = serde_json::from_str(json).unwrap();
//!
//! // Convert to domain type with validation
//! let config = section.to_provider_config().unwrap();
//! assert_eq!(config.provider_name(), "lxd");
//! ```

mod hetzner;
mod lxd;

pub use hetzner::HetznerProviderSection;
pub use lxd::LxdProviderSection;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::CreateConfigError;
use crate::domain::provider::{HetznerConfig, LxdConfig, Provider, ProviderConfig};
use crate::domain::ProfileName;

/// Provider-specific configuration section
///
/// Each variant contains the configuration fields specific to that provider
/// using **raw primitives** (`String`) for JSON deserialization.
///
/// This is a tagged enum that deserializes based on the `"provider"` field in JSON.
///
/// # Conversion
///
/// Use `to_provider_config()` to validate and convert to domain types.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
///     ProviderSection, LxdProviderSection
/// };
///
/// let section = ProviderSection::Lxd(LxdProviderSection {
///     profile_name: "torrust-profile-dev".to_string(),
/// });
///
/// let config = section.to_provider_config().unwrap();
/// assert_eq!(config.provider_name(), "lxd");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "provider")]
pub enum ProviderSection {
    /// LXD provider configuration
    #[serde(rename = "lxd")]
    Lxd(LxdProviderSection),

    /// Hetzner provider configuration
    #[serde(rename = "hetzner")]
    Hetzner(HetznerProviderSection),
}

impl ProviderSection {
    /// Returns the provider type (no validation needed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     ProviderSection, LxdProviderSection
    /// };
    /// use torrust_tracker_deployer_lib::domain::provider::Provider;
    ///
    /// let section = ProviderSection::Lxd(LxdProviderSection {
    ///     profile_name: "test".to_string(),
    /// });
    /// assert_eq!(section.provider(), Provider::Lxd);
    /// ```
    #[must_use]
    pub fn provider(&self) -> Provider {
        match self {
            Self::Lxd(_) => Provider::Lxd,
            Self::Hetzner(_) => Provider::Hetzner,
        }
    }

    /// Converts the config to a validated domain `ProviderConfig`.
    ///
    /// This method validates raw string fields and converts them to
    /// domain types with proper validation.
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if validation fails:
    /// - `InvalidProfileName` - if the LXD profile name is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    ///     ProviderSection, LxdProviderSection
    /// };
    ///
    /// // Valid conversion
    /// let section = ProviderSection::Lxd(LxdProviderSection {
    ///     profile_name: "torrust-profile-dev".to_string(),
    /// });
    /// let config = section.to_provider_config().unwrap();
    /// assert_eq!(config.provider_name(), "lxd");
    ///
    /// // Invalid profile name causes error
    /// let invalid = ProviderSection::Lxd(LxdProviderSection {
    ///     profile_name: "".to_string(), // Empty is invalid
    /// });
    /// assert!(invalid.to_provider_config().is_err());
    /// ```
    pub fn to_provider_config(self) -> Result<ProviderConfig, CreateConfigError> {
        match self {
            Self::Lxd(lxd) => {
                let profile_name = ProfileName::new(lxd.profile_name)?;
                Ok(ProviderConfig::Lxd(LxdConfig { profile_name }))
            }
            Self::Hetzner(hetzner) => {
                // Note: Future improvement could add validation for these fields
                Ok(ProviderConfig::Hetzner(HetznerConfig {
                    api_token: hetzner.api_token,
                    server_type: hetzner.server_type,
                    location: hetzner.location,
                    image: hetzner.image,
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_lxd_section() -> ProviderSection {
        ProviderSection::Lxd(LxdProviderSection {
            profile_name: "torrust-profile".to_string(),
        })
    }

    fn create_hetzner_section() -> ProviderSection {
        ProviderSection::Hetzner(HetznerProviderSection {
            api_token: "test-token".to_string(),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        })
    }

    #[test]
    fn it_should_return_lxd_provider_when_section_is_lxd() {
        let section = create_lxd_section();
        assert_eq!(section.provider(), Provider::Lxd);
    }

    #[test]
    fn it_should_return_hetzner_provider_when_section_is_hetzner() {
        let section = create_hetzner_section();
        assert_eq!(section.provider(), Provider::Hetzner);
    }

    #[test]
    fn it_should_deserialize_lxd_section_from_json() {
        let json = r#"{"provider": "lxd", "profile_name": "torrust-profile"}"#;
        let section: ProviderSection = serde_json::from_str(json).unwrap();

        assert_eq!(section.provider(), Provider::Lxd);
        if let ProviderSection::Lxd(lxd) = section {
            assert_eq!(lxd.profile_name, "torrust-profile");
        } else {
            panic!("Expected LXD section");
        }
    }

    #[test]
    fn it_should_deserialize_hetzner_section_from_json() {
        let json = r#"{
            "provider": "hetzner",
            "api_token": "token123",
            "server_type": "cx32",
            "location": "fsn1",
            "image": "ubuntu-24.04"
        }"#;
        let section: ProviderSection = serde_json::from_str(json).unwrap();

        assert_eq!(section.provider(), Provider::Hetzner);
        if let ProviderSection::Hetzner(hetzner) = section {
            assert_eq!(hetzner.api_token, "token123");
            assert_eq!(hetzner.server_type, "cx32");
            assert_eq!(hetzner.location, "fsn1");
            assert_eq!(hetzner.image, "ubuntu-24.04");
        } else {
            panic!("Expected Hetzner section");
        }
    }

    #[test]
    fn it_should_serialize_lxd_section_to_json() {
        let section = create_lxd_section();
        let json = serde_json::to_string(&section).unwrap();

        assert!(json.contains("\"provider\":\"lxd\""));
        assert!(json.contains("\"profile_name\":\"torrust-profile\""));
    }

    #[test]
    fn it_should_serialize_hetzner_section_to_json() {
        let section = create_hetzner_section();
        let json = serde_json::to_string(&section).unwrap();

        assert!(json.contains("\"provider\":\"hetzner\""));
        assert!(json.contains("\"api_token\":\"test-token\""));
        assert!(json.contains("\"server_type\":\"cx22\""));
        assert!(json.contains("\"location\":\"nbg1\""));
        assert!(json.contains("\"image\":\"ubuntu-24.04\""));
    }

    #[test]
    fn it_should_convert_lxd_section_to_domain_config() {
        let section = create_lxd_section();
        let config = section.to_provider_config().unwrap();

        assert_eq!(config.provider(), Provider::Lxd);
        assert_eq!(config.provider_name(), "lxd");
        assert_eq!(
            config.as_lxd().unwrap().profile_name.as_str(),
            "torrust-profile"
        );
    }

    #[test]
    fn it_should_convert_hetzner_section_to_domain_config() {
        let section = create_hetzner_section();
        let config = section.to_provider_config().unwrap();

        assert_eq!(config.provider(), Provider::Hetzner);
        assert_eq!(config.provider_name(), "hetzner");

        let hetzner = config.as_hetzner().unwrap();
        assert_eq!(hetzner.api_token, "test-token");
        assert_eq!(hetzner.server_type, "cx22");
        assert_eq!(hetzner.location, "nbg1");
        assert_eq!(hetzner.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_fail_conversion_when_lxd_profile_name_is_empty() {
        let section = ProviderSection::Lxd(LxdProviderSection {
            profile_name: String::new(), // Empty is invalid
        });
        let result = section.to_provider_config();
        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_conversion_when_lxd_profile_name_starts_with_dash() {
        let section = ProviderSection::Lxd(LxdProviderSection {
            profile_name: "-invalid".to_string(),
        });
        let result = section.to_provider_config();
        assert!(result.is_err());
    }

    #[test]
    fn it_should_fail_conversion_when_lxd_profile_name_ends_with_dash() {
        let section = ProviderSection::Lxd(LxdProviderSection {
            profile_name: "invalid-".to_string(),
        });
        let result = section.to_provider_config();
        assert!(result.is_err());
    }

    #[test]
    fn it_should_be_cloneable() {
        let section = create_lxd_section();
        let cloned = section.clone();
        assert_eq!(section, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait() {
        let section = create_lxd_section();
        let debug = format!("{section:?}");
        assert!(debug.contains("Lxd"));
        assert!(debug.contains("profile_name"));
    }
}
