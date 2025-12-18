//! Hetzner Provider Domain Types
//!
//! This module contains domain types specific to the Hetzner provider.
//! Hetzner is used for production deployments, providing cost-effective
//! cloud infrastructure with good European presence.

use serde::{Deserialize, Serialize};

use crate::shared::ApiToken;

/// Hetzner-specific configuration (Domain Type)
///
/// Hetzner is used for production deployments. It provides cost-effective
/// cloud infrastructure with good European presence.
///
/// Note: This struct is defined for enum completeness but will be
/// fully implemented in Phase 2 (Add Hetzner Provider task).
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::provider::HetznerConfig;
/// use torrust_tracker_deployer_lib::shared::secrets::ApiToken;
///
/// let config = HetznerConfig {
///     api_token: ApiToken::from("your-api-token"),
///     server_type: "cx22".to_string(),
///     location: "nbg1".to_string(),
///     image: "ubuntu-24.04".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerConfig {
    /// Hetzner API token for authentication.
    ///
    /// This value is kept secure and not exposed in debug output.
    pub api_token: ApiToken,

    /// Hetzner server type (e.g., "cx22", "cx32", "cpx11").
    ///
    /// Determines the VM specifications (CPU, RAM, storage).
    /// Note: Future improvement could use a validated `ServerType` type.
    pub server_type: String,

    /// Hetzner datacenter location (e.g., "fsn1", "nbg1", "hel1").
    ///
    /// Determines where the VM will be physically located.
    /// Note: Future improvement could use a validated `Location` type.
    pub location: String,

    /// Operating system image (e.g., "ubuntu-24.04", "ubuntu-22.04", "debian-12").
    ///
    /// Determines the base operating system for the server.
    /// Note: Future improvement could use a validated `Image` type.
    pub image: String,
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_hetzner_config() -> HetznerConfig {
        HetznerConfig {
            api_token: ApiToken::from("test-token"),
            server_type: "cx22".to_string(),
            location: "nbg1".to_string(),
            image: "ubuntu-24.04".to_string(),
        }
    }

    #[test]
    fn it_should_store_all_fields_when_created() {
        let config = HetznerConfig {
            api_token: ApiToken::from("token123"),
            server_type: "cx32".to_string(),
            location: "fsn1".to_string(),
            image: "ubuntu-22.04".to_string(),
        };
        assert_eq!(config.api_token.expose_secret(), "token123");
        assert_eq!(config.server_type, "cx32");
        assert_eq!(config.location, "fsn1");
        assert_eq!(config.image, "ubuntu-22.04");
    }

    #[test]
    fn it_should_serialize_to_json_when_valid_config_exists() {
        let config = create_hetzner_config();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"api_token\":\"test-token\""));
        assert!(json.contains("\"server_type\":\"cx22\""));
        assert!(json.contains("\"location\":\"nbg1\""));
        assert!(json.contains("\"image\":\"ubuntu-24.04\""));
    }

    #[test]
    fn it_should_deserialize_from_json_when_valid_json_provided() {
        let json = r#"{"api_token":"token","server_type":"cx22","location":"nbg1","image":"ubuntu-24.04"}"#;
        let config: HetznerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.api_token.expose_secret(), "token");
        assert_eq!(config.server_type, "cx22");
        assert_eq!(config.location, "nbg1");
        assert_eq!(config.image, "ubuntu-24.04");
    }

    #[test]
    fn it_should_be_cloneable_when_cloned() {
        let config = create_hetzner_config();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait_when_formatted() {
        let config = create_hetzner_config();
        let debug = format!("{config:?}");
        assert!(debug.contains("HetznerConfig"));
        assert!(debug.contains("api_token"));
        assert!(debug.contains("server_type"));
        assert!(debug.contains("location"));
        assert!(debug.contains("image"));
        // API token should be redacted in debug output
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("test-token"));
    }
}
