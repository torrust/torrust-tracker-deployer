//! Docker Compose Network Domain Type
//!
//! This module defines the [`Network`] enum representing Docker Compose networks
//! used for service isolation. Each network serves a specific security purpose
//! in the deployment topology.
//!
//! ## Network Purposes
//!
//! | Network | Purpose | Connected Services |
//! |---------|---------|-------------------|
//! | `Database` | Isolates database access | Tracker ↔ MySQL |
//! | `Metrics` | Metrics scraping | Tracker ↔ Prometheus |
//! | `Visualization` | Dashboard queries | Prometheus ↔ Grafana |
//! | `Proxy` | TLS termination | Caddy ↔ backend services |
//!
//! ## Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::domain::topology::Network;
//!
//! let network = Network::Metrics;
//! assert_eq!(network.name(), "metrics_network");
//! assert_eq!(network.driver(), "bridge");
//! ```

use std::fmt;

use serde::Serialize;

/// Docker Compose networks used for service isolation
///
/// Each network serves a specific security purpose:
/// - `Database`: Isolates database access to only the tracker
/// - `Metrics`: Allows Prometheus to scrape tracker metrics
/// - `Visualization`: Allows Grafana to query Prometheus
/// - `Proxy`: Allows Caddy to reverse proxy to backend services
///
/// # Serialization
///
/// When serialized, the network outputs its name string (e.g., `"metrics_network"`),
/// making it directly usable in Tera templates:
///
/// ```yaml
/// networks:
///   - {{ network }}  # Renders as "metrics_network"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    /// Network for database access (Tracker ↔ MySQL)
    ///
    /// Only the tracker and MySQL are connected to this network,
    /// ensuring database isolation from other services.
    Database,

    /// Network for metrics scraping (Tracker ↔ Prometheus)
    ///
    /// Allows Prometheus to scrape metrics from the tracker
    /// while keeping Prometheus isolated from other traffic.
    Metrics,

    /// Network for visualization queries (Prometheus ↔ Grafana)
    ///
    /// Allows Grafana to query Prometheus for dashboard data.
    Visualization,

    /// Network for TLS proxy (Caddy ↔ backend services)
    ///
    /// Allows Caddy to reverse proxy to services that need
    /// TLS termination (tracker API, Grafana, etc.).
    Proxy,
}

impl Network {
    /// Returns the network name as used in docker-compose.yml
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::Network;
    ///
    /// assert_eq!(Network::Database.name(), "database_network");
    /// assert_eq!(Network::Metrics.name(), "metrics_network");
    /// assert_eq!(Network::Visualization.name(), "visualization_network");
    /// assert_eq!(Network::Proxy.name(), "proxy_network");
    /// ```
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Network::Database => "database_network",
            Network::Metrics => "metrics_network",
            Network::Visualization => "visualization_network",
            Network::Proxy => "proxy_network",
        }
    }

    /// Returns the network driver
    ///
    /// Currently all networks use the `bridge` driver.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::Network;
    ///
    /// assert_eq!(Network::Database.driver(), "bridge");
    /// ```
    #[must_use]
    pub fn driver(&self) -> &'static str {
        "bridge"
    }

    /// Returns all network variants
    ///
    /// Useful for iteration when generating the global networks section
    /// in docker-compose.yml.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::Network;
    ///
    /// let all = Network::all();
    /// assert_eq!(all.len(), 4);
    /// ```
    #[must_use]
    pub fn all() -> &'static [Network] {
        &[
            Network::Database,
            Network::Metrics,
            Network::Visualization,
            Network::Proxy,
        ]
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Custom serialization to output the network name string
///
/// This allows the Network enum to be used directly in Tera templates
/// without needing wrapper types or custom serialization logic.
impl Serialize for Network {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Network name tests
    // ==========================================================================

    #[test]
    fn it_should_return_correct_network_name_for_database() {
        assert_eq!(Network::Database.name(), "database_network");
    }

    #[test]
    fn it_should_return_correct_network_name_for_metrics() {
        assert_eq!(Network::Metrics.name(), "metrics_network");
    }

    #[test]
    fn it_should_return_correct_network_name_for_visualization() {
        assert_eq!(Network::Visualization.name(), "visualization_network");
    }

    #[test]
    fn it_should_return_correct_network_name_for_proxy() {
        assert_eq!(Network::Proxy.name(), "proxy_network");
    }

    // ==========================================================================
    // Driver tests
    // ==========================================================================

    #[test]
    fn it_should_return_bridge_driver_for_database_network() {
        assert_eq!(Network::Database.driver(), "bridge");
    }

    #[test]
    fn it_should_return_bridge_driver_for_metrics_network() {
        assert_eq!(Network::Metrics.driver(), "bridge");
    }

    #[test]
    fn it_should_return_bridge_driver_for_visualization_network() {
        assert_eq!(Network::Visualization.driver(), "bridge");
    }

    #[test]
    fn it_should_return_bridge_driver_for_proxy_network() {
        assert_eq!(Network::Proxy.driver(), "bridge");
    }

    // ==========================================================================
    // Display tests
    // ==========================================================================

    #[test]
    fn it_should_display_network_as_name() {
        assert_eq!(format!("{}", Network::Database), "database_network");
        assert_eq!(format!("{}", Network::Metrics), "metrics_network");
        assert_eq!(
            format!("{}", Network::Visualization),
            "visualization_network"
        );
        assert_eq!(format!("{}", Network::Proxy), "proxy_network");
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn it_should_serialize_network_to_name_string() {
        let json = serde_json::to_string(&Network::Metrics).unwrap();
        assert_eq!(json, "\"metrics_network\"");
    }

    #[test]
    fn it_should_serialize_all_networks_correctly() {
        assert_eq!(
            serde_json::to_string(&Network::Database).unwrap(),
            "\"database_network\""
        );
        assert_eq!(
            serde_json::to_string(&Network::Metrics).unwrap(),
            "\"metrics_network\""
        );
        assert_eq!(
            serde_json::to_string(&Network::Visualization).unwrap(),
            "\"visualization_network\""
        );
        assert_eq!(
            serde_json::to_string(&Network::Proxy).unwrap(),
            "\"proxy_network\""
        );
    }

    // ==========================================================================
    // All networks tests
    // ==========================================================================

    #[test]
    fn it_should_return_all_four_networks() {
        let all = Network::all();

        assert_eq!(all.len(), 4);
        assert!(all.contains(&Network::Database));
        assert!(all.contains(&Network::Metrics));
        assert!(all.contains(&Network::Visualization));
        assert!(all.contains(&Network::Proxy));
    }

    // ==========================================================================
    // Equality tests
    // ==========================================================================

    #[test]
    fn it_should_compare_networks_for_equality() {
        assert_eq!(Network::Database, Network::Database);
        assert_ne!(Network::Database, Network::Metrics);
    }

    // ==========================================================================
    // Clone and Copy tests
    // ==========================================================================

    #[test]
    fn it_should_be_copyable() {
        let network = Network::Metrics;
        let copied = network; // Copy
        assert_eq!(network, copied);
    }

    #[test]
    fn it_should_be_clonable() {
        let network = Network::Metrics;
        #[allow(clippy::clone_on_copy)]
        let cloned = network.clone();
        assert_eq!(network, cloned);
    }

    // ==========================================================================
    // Hash tests (for use in HashSet/HashMap)
    // ==========================================================================

    #[test]
    fn it_should_be_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Network::Database);
        set.insert(Network::Metrics);
        set.insert(Network::Database); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Network::Database));
        assert!(set.contains(&Network::Metrics));
    }
}
