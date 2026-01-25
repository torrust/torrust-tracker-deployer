//! Docker Compose Service Domain Type
//!
//! This module defines the [`Service`] enum representing services in a
//! Docker Compose deployment.
//!
//! ## Services
//!
//! | Service | Purpose | Description |
//! |---------|---------|-------------|
//! | `Tracker` | Core service | BitTorrent tracker |
//! | `MySQL` | Database | Persistent storage |
//! | `Prometheus` | Metrics | Metrics collection |
//! | `Grafana` | Visualization | Metrics dashboard |
//! | `Caddy` | Proxy | TLS termination |

use std::fmt;

use serde::Serialize;

/// Docker Compose services in the deployment
///
/// This enum provides type-safe service identification, preventing typos
/// and enabling exhaustive matching in domain logic.
///
/// # Serialization
///
/// When serialized, the service outputs its name string (e.g., `"tracker"`),
/// making it directly usable in Tera templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    /// The core `BitTorrent` tracker service
    ///
    /// Always present in every deployment.
    Tracker,

    /// `MySQL` database service
    ///
    /// Provides persistent storage when `SQLite` is not used.
    #[serde(rename = "mysql")]
    MySQL,

    /// Prometheus metrics collection service
    ///
    /// Scrapes metrics from the tracker.
    Prometheus,

    /// Grafana visualization service
    ///
    /// Displays dashboards with Prometheus data.
    Grafana,

    /// Caddy reverse proxy service
    ///
    /// Provides automatic TLS termination with Let's Encrypt.
    Caddy,
}

impl Service {
    /// Returns the service name as used in docker-compose.yml
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::Service;
    ///
    /// assert_eq!(Service::Tracker.name(), "tracker");
    /// assert_eq!(Service::MySQL.name(), "mysql");
    /// assert_eq!(Service::Prometheus.name(), "prometheus");
    /// assert_eq!(Service::Grafana.name(), "grafana");
    /// assert_eq!(Service::Caddy.name(), "caddy");
    /// ```
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Service::Tracker => "tracker",
            Service::MySQL => "mysql",
            Service::Prometheus => "prometheus",
            Service::Grafana => "grafana",
            Service::Caddy => "caddy",
        }
    }

    /// Returns all service variants
    ///
    /// Useful for iteration and validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::Service;
    ///
    /// let all = Service::all();
    /// assert_eq!(all.len(), 5);
    /// ```
    #[must_use]
    pub fn all() -> &'static [Service] {
        &[
            Service::Tracker,
            Service::MySQL,
            Service::Prometheus,
            Service::Grafana,
            Service::Caddy,
        ]
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod service_names {
        use super::*;

        #[test]
        fn it_should_return_correct_service_name_for_tracker() {
            assert_eq!(Service::Tracker.name(), "tracker");
        }

        #[test]
        fn it_should_return_correct_service_name_for_mysql() {
            assert_eq!(Service::MySQL.name(), "mysql");
        }

        #[test]
        fn it_should_return_correct_service_name_for_prometheus() {
            assert_eq!(Service::Prometheus.name(), "prometheus");
        }

        #[test]
        fn it_should_return_correct_service_name_for_grafana() {
            assert_eq!(Service::Grafana.name(), "grafana");
        }

        #[test]
        fn it_should_return_correct_service_name_for_caddy() {
            assert_eq!(Service::Caddy.name(), "caddy");
        }
    }

    mod display {
        use super::*;

        #[test]
        fn it_should_display_service_as_name() {
            assert_eq!(format!("{}", Service::Tracker), "tracker");
            assert_eq!(format!("{}", Service::MySQL), "mysql");
            assert_eq!(format!("{}", Service::Prometheus), "prometheus");
            assert_eq!(format!("{}", Service::Grafana), "grafana");
            assert_eq!(format!("{}", Service::Caddy), "caddy");
        }
    }

    mod all_services {
        use super::*;

        #[test]
        fn it_should_return_all_five_services() {
            let all = Service::all();
            assert_eq!(all.len(), 5);
        }

        #[test]
        fn it_should_contain_tracker_service() {
            assert!(Service::all().contains(&Service::Tracker));
        }

        #[test]
        fn it_should_contain_mysql_service() {
            assert!(Service::all().contains(&Service::MySQL));
        }

        #[test]
        fn it_should_contain_prometheus_service() {
            assert!(Service::all().contains(&Service::Prometheus));
        }

        #[test]
        fn it_should_contain_grafana_service() {
            assert!(Service::all().contains(&Service::Grafana));
        }

        #[test]
        fn it_should_contain_caddy_service() {
            assert!(Service::all().contains(&Service::Caddy));
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn it_should_serialize_to_lowercase_name() {
            let json = serde_json::to_string(&Service::Tracker).unwrap();
            assert_eq!(json, "\"tracker\"");

            let json = serde_json::to_string(&Service::MySQL).unwrap();
            assert_eq!(json, "\"mysql\"");
        }
    }
}
