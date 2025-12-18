//! Hetzner Provider Configuration Section (Application Layer)
//!
//! This module contains the configuration section for the Hetzner provider.
//! Uses raw `String` fields for JSON deserialization, which are then validated
//! when converting to domain types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::shared::PlainApiToken;

/// Hetzner-specific configuration section
///
/// Uses raw `String` fields for JSON deserialization. Convert to domain
/// `HetznerConfig` via `ProviderSection::to_provider_config()`.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::HetznerProviderSection;
///
/// let section = HetznerProviderSection {
///     api_token: "your-api-token".to_string(),
///     server_type: "cx22".to_string(),
///     location: "nbg1".to_string(),
///     image: "ubuntu-24.04".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HetznerProviderSection {
    /// Hetzner API token in plain text format (DTO layer).
    ///
    /// This uses [`PlainApiToken`] to mark it as a transparent secret during
    /// deserialization. Convert to domain `ApiToken` at the DTO-to-domain boundary.
    pub api_token: PlainApiToken,

    /// Hetzner server type (e.g., "cx22", "cx32", "cpx11").
    pub server_type: String,

    /// Hetzner datacenter location (e.g., "fsn1", "nbg1", "hel1").
    pub location: String,

    /// Hetzner server image (e.g., "ubuntu-24.04", "ubuntu-22.04", "debian-12").
    pub image: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_hetzner_section() -> HetznerProviderSection {
        HetznerProviderSection {
            api_token: "token".to_string(),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        }
    }

    #[test]
    fn it_should_serialize_to_json() {
        let section = create_hetzner_section();
        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"api_token\":\"token\""));
        assert!(json.contains("\"server_type\":\"cx22\""));
        assert!(json.contains("\"location\":\"nbg1\""));
        assert!(json.contains("\"image\":\"ubuntu-24.04\""));
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = r#"{"api_token":"token","server_type":"cx22","location":"nbg1","image":"ubuntu-24.04"}"#;
        let section: HetznerProviderSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.api_token, "token");
        assert_eq!(section.server_type, "cx22");
        assert_eq!(section.location, "nbg1");
        assert_eq!(section.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_be_cloneable() {
        let section = create_hetzner_section();
        let cloned = section.clone();
        assert_eq!(section, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait() {
        let section = create_hetzner_section();
        let debug = format!("{section:?}");
        assert!(debug.contains("HetznerProviderSection"));
        assert!(debug.contains("api_token"));
        assert!(debug.contains("server_type"));
        assert!(debug.contains("location"));
        assert!(debug.contains("image"));
    }
}
