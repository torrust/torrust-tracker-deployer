//! Provider enum representing available infrastructure providers.
//!
//! This module defines the `Provider` enum which represents the available
//! infrastructure providers for deploying Torrust Tracker environments.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Supported infrastructure providers
///
/// This enum represents the available infrastructure providers for deploying
/// Torrust Tracker environments. It's a domain concept used throughout the
/// codebase to determine provider-specific behavior.
///
/// # Providers
///
/// - **LXD**: Local development and testing provider. Fast VM creation with no
///   cloud costs, ideal for E2E tests and CI environments.
/// - **Hetzner**: Production cloud provider. Cost-effective with good European
///   presence, suitable for production deployments.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::provider::Provider;
///
/// let provider = Provider::Lxd;
/// assert_eq!(provider.as_str(), "lxd");
/// assert_eq!(provider.to_string(), "lxd");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// LXD - Local development and testing
    Lxd,
    /// Hetzner Cloud - Production deployments
    Hetzner,
}

impl Provider {
    /// Returns the provider name as used in directory paths.
    ///
    /// This is used to construct paths like `templates/tofu/{provider}/`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::provider::Provider;
    ///
    /// assert_eq!(Provider::Lxd.as_str(), "lxd");
    /// assert_eq!(Provider::Hetzner.as_str(), "hetzner");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lxd => "lxd",
            Self::Hetzner => "hetzner",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_lowercase_string_when_as_str_called() {
        assert_eq!(Provider::Lxd.as_str(), "lxd");
        assert_eq!(Provider::Hetzner.as_str(), "hetzner");
    }

    #[test]
    fn it_should_return_lowercase_string_when_displayed() {
        assert_eq!(format!("{}", Provider::Lxd), "lxd");
        assert_eq!(format!("{}", Provider::Hetzner), "hetzner");
    }

    #[test]
    fn it_should_serialize_to_lowercase_json_string() {
        let lxd_json = serde_json::to_string(&Provider::Lxd).unwrap();
        let hetzner_json = serde_json::to_string(&Provider::Hetzner).unwrap();

        assert_eq!(lxd_json, "\"lxd\"");
        assert_eq!(hetzner_json, "\"hetzner\"");
    }

    #[test]
    fn it_should_deserialize_from_lowercase_json_string() {
        let lxd: Provider = serde_json::from_str("\"lxd\"").unwrap();
        let hetzner: Provider = serde_json::from_str("\"hetzner\"").unwrap();

        assert_eq!(lxd, Provider::Lxd);
        assert_eq!(hetzner, Provider::Hetzner);
    }

    #[test]
    fn it_should_be_copy_and_clone_when_assigned() {
        let provider = Provider::Lxd;
        let copied = provider; // Copy happens implicitly
        assert_eq!(provider, copied);
    }

    #[test]
    fn it_should_be_hashable_when_inserted_in_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Provider::Lxd);
        set.insert(Provider::Hetzner);

        assert!(set.contains(&Provider::Lxd));
        assert!(set.contains(&Provider::Hetzner));
        assert_eq!(set.len(), 2);
    }
}
