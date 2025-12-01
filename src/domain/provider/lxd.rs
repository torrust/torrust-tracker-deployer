//! LXD Provider Domain Types
//!
//! This module contains domain types specific to the LXD provider.
//! LXD is used for local development and testing, providing fast VM creation
//! with no cloud costs, ideal for E2E tests and CI environments.

use serde::{Deserialize, Serialize};

use crate::domain::ProfileName;

/// LXD-specific configuration (Domain Type)
///
/// LXD is used for local development and testing. It provides fast VM creation
/// with no cloud costs, making it ideal for E2E tests and CI environments.
///
/// Uses validated domain types (e.g., `ProfileName`).
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::provider::LxdConfig;
/// use torrust_tracker_deployer_lib::domain::ProfileName;
///
/// let config = LxdConfig {
///     profile_name: ProfileName::new("torrust-profile-dev").unwrap(),
/// };
/// assert_eq!(config.profile_name.as_str(), "torrust-profile-dev");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdConfig {
    /// LXD profile name for the instance (validated domain type).
    ///
    /// This profile must exist in LXD and typically configures
    /// networking, storage, and resource limits.
    pub profile_name: ProfileName,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_store_validated_profile_name_when_created() {
        let profile_name = ProfileName::new("test-profile").unwrap();
        let config = LxdConfig {
            profile_name: profile_name.clone(),
        };
        assert_eq!(config.profile_name, profile_name);
    }

    #[test]
    fn it_should_serialize_to_json_when_valid_config_exists() {
        let config = LxdConfig {
            profile_name: ProfileName::new("torrust-profile").unwrap(),
        };
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"profile_name\":\"torrust-profile\""));
    }

    #[test]
    fn it_should_deserialize_from_json_when_valid_json_provided() {
        let json = r#"{"profile_name":"torrust-profile"}"#;
        let config: LxdConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.profile_name.as_str(), "torrust-profile");
    }

    #[test]
    fn it_should_be_cloneable_when_cloned() {
        let config = LxdConfig {
            profile_name: ProfileName::new("test").unwrap(),
        };
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn it_should_implement_debug_trait_when_formatted() {
        let config = LxdConfig {
            profile_name: ProfileName::new("test").unwrap(),
        };
        let debug = format!("{config:?}");
        assert!(debug.contains("LxdConfig"));
        assert!(debug.contains("profile_name"));
    }
}
