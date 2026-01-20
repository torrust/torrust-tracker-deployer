//! Grafana service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::shared::secrets::Password;

/// Grafana service configuration for Docker Compose
///
/// Contains all configuration needed for the Grafana service in Docker Compose,
/// including admin credentials, TLS settings, and network connections. All logic
/// is pre-computed in Rust to keep the Tera template simple.
#[derive(Serialize, Debug, Clone)]
pub struct GrafanaServiceConfig {
    /// Grafana admin username
    pub admin_user: String,
    /// Grafana admin password
    pub admin_password: Password,
    /// Whether Grafana has TLS enabled (port should not be exposed if true)
    #[serde(default)]
    pub has_tls: bool,
    /// Networks the Grafana service should connect to
    ///
    /// Pre-computed list based on enabled features:
    /// - Always includes `visualization_network` (queries Prometheus)
    /// - Includes `proxy_network` if Caddy TLS proxy is enabled
    pub networks: Vec<String>,
}

impl GrafanaServiceConfig {
    /// Creates a new `GrafanaServiceConfig` with pre-computed networks
    ///
    /// # Arguments
    ///
    /// * `admin_user` - Grafana admin username
    /// * `admin_password` - Grafana admin password
    /// * `has_tls` - Whether Grafana has TLS enabled (via Caddy)
    /// * `has_caddy` - Whether Caddy TLS proxy is enabled (adds `proxy_network`)
    #[must_use]
    pub fn new(
        admin_user: String,
        admin_password: Password,
        has_tls: bool,
        has_caddy: bool,
    ) -> Self {
        let networks = Self::compute_networks(has_caddy);

        Self {
            admin_user,
            admin_password,
            has_tls,
            networks,
        }
    }

    /// Computes the list of networks for the Grafana service
    fn compute_networks(has_caddy: bool) -> Vec<String> {
        let mut networks = vec!["visualization_network".to_string()];

        if has_caddy {
            networks.push("proxy_network".to_string());
        }

        networks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_grafana_config_with_only_visualization_network_when_caddy_disabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), false, false);

        assert_eq!(config.admin_user, "admin");
        assert!(!config.has_tls);
        assert_eq!(config.networks, vec!["visualization_network"]);
    }

    #[test]
    fn it_should_create_grafana_config_with_both_networks_when_caddy_enabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), true, true);

        assert_eq!(config.admin_user, "admin");
        assert!(config.has_tls);
        assert_eq!(
            config.networks,
            vec!["visualization_network", "proxy_network"]
        );
    }
}
