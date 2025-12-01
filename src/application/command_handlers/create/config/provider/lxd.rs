//! LXD Provider Configuration Section (Application Layer)
//!
//! This module contains the configuration section for the LXD provider.
//! Uses raw `String` for JSON deserialization, which is then validated
//! when converting to domain types.

use serde::{Deserialize, Serialize};

/// LXD-specific configuration section
///
/// Uses raw `String` for JSON deserialization. Convert to domain `LxdConfig`
/// via `ProviderSection::to_provider_config()`.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::LxdProviderSection;
///
/// let section = LxdProviderSection {
///     profile_name: "torrust-profile-dev".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdProviderSection {
    /// LXD profile name (raw string - validated on conversion).
    pub profile_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_serialize_to_json() {
        let section = LxdProviderSection {
            profile_name: "test".to_string(),
        };
        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"profile_name\":\"test\""));
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = r#"{"profile_name":"torrust-profile"}"#;
        let section: LxdProviderSection = serde_json::from_str(json).unwrap();
        assert_eq!(section.profile_name, "torrust-profile");
    }

    #[test]
    fn it_should_be_cloneable() {
        let section = LxdProviderSection {
            profile_name: "test".to_string(),
        };
        let cloned = section.clone();
        assert_eq!(section, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait() {
        let section = LxdProviderSection {
            profile_name: "test".to_string(),
        };
        let debug = format!("{section:?}");
        assert!(debug.contains("LxdProviderSection"));
        assert!(debug.contains("profile_name"));
    }
}
