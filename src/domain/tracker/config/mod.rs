//! Tracker configuration domain types
//!
//! This module contains the main tracker configuration and component types
//! used for deploying the Torrust Tracker.

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::{BindingAddress, Protocol};
use crate::domain::tls::TlsConfig;

mod core;
mod health_check_api;
mod http;
mod http_api;
mod udp;

pub use core::{DatabaseConfig, MysqlConfig, SqliteConfig, TrackerCoreConfig};
pub use health_check_api::HealthCheckApiConfig;
pub use http::HttpTrackerConfig;
pub use http_api::HttpApiConfig;
pub use udp::UdpTrackerConfig;

/// Tracker deployment configuration
///
/// This structure mirrors the real tracker configuration but only includes
/// user-configurable fields that are exposed via the environment.json file.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::{
///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
/// };
///
/// let tracker_config = TrackerConfig {
///     core: TrackerCoreConfig {
///         database: DatabaseConfig::Sqlite(SqliteConfig {
///             database_name: "tracker.db".to_string(),
///         }),
///         private: false,
///     },
///     udp_trackers: vec![
///         UdpTrackerConfig { bind_address: "0.0.0.0:6969".parse().unwrap() },
///     ],
///     http_trackers: vec![
///         HttpTrackerConfig { bind_address: "0.0.0.0:7070".parse().unwrap(), tls: None },
///     ],
///     http_api: HttpApiConfig {
///         bind_address: "0.0.0.0:1212".parse().unwrap(),
///         admin_token: "MyAccessToken".to_string().into(),
///         tls: None,
///     },
///     health_check_api: HealthCheckApiConfig {
///         bind_address: "127.0.0.1:1313".parse().unwrap(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerConfig {
    /// Core tracker configuration
    pub core: TrackerCoreConfig,

    /// UDP tracker instances
    pub udp_trackers: Vec<UdpTrackerConfig>,

    /// HTTP tracker instances
    pub http_trackers: Vec<HttpTrackerConfig>,

    /// HTTP API configuration
    pub http_api: HttpApiConfig,

    /// Health Check API configuration
    pub health_check_api: HealthCheckApiConfig,
}

/// Error type for tracker configuration validation failures
#[derive(Debug, Clone, PartialEq)]
pub enum TrackerConfigError {
    /// Multiple services attempting to bind to the same socket address
    DuplicateSocketAddress {
        /// The conflicting socket address
        address: SocketAddr,
        /// The protocol (UDP or TCP)
        protocol: Protocol,
        /// Names of services attempting to bind to this address
        services: Vec<String>,
    },
}

impl fmt::Display for TrackerConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSocketAddress {
                address,
                protocol,
                services,
            } => {
                let services_list = services
                    .iter()
                    .map(|s| format!("'{s}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "Socket address conflict: {services_list} cannot bind to {address} ({protocol})\n\
                    Tip: Assign different port numbers to each service"
                )
            }
        }
    }
}

impl std::error::Error for TrackerConfigError {}

impl TrackerConfigError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::DuplicateSocketAddress {
                address,
                protocol,
                services,
            } => {
                use std::fmt::Write;

                let mut help =
                    String::from("Socket Address Conflict - Detailed Troubleshooting:\n\n");

                help.push_str("Conflicting services:\n");
                for service in services {
                    let _ = writeln!(help, "  - {service}: {address} ({protocol})");
                }
                help.push('\n');

                help.push_str("Why this fails:\n");
                let _ = write!(
                    help,
                    "Two services using the same protocol ({protocol}) cannot bind to the same\n\
                    IP address and port number. The second service will fail with\n\
                    \"Address already in use\" error.\n\n"
                );

                help.push_str("How to fix:\n");
                help.push_str(
                    "1. Assign different port numbers to each service\n\
                    2. Or configure only one service to use this address\n\n",
                );

                help.push_str("Note:\n");
                help.push_str(
                    "Services using different protocols (UDP vs TCP) CAN share the same port.\n\
                    See: docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md\n",
                );

                help
            }
        }
    }
}

impl TrackerConfig {
    /// Validates the tracker configuration for socket address conflicts
    ///
    /// Checks that no two services using the same protocol attempt to bind
    /// to the same socket address (IP + port). Services using different
    /// protocols (UDP vs TCP) can share the same port number.
    ///
    /// # Errors
    ///
    /// Returns `TrackerConfigError::DuplicateSocketAddress` if multiple services
    /// using the same protocol attempt to bind to the same socket address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{
    ///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
    ///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
    /// };
    ///
    /// let config = TrackerConfig {
    ///     core: TrackerCoreConfig {
    ///         database: DatabaseConfig::Sqlite(SqliteConfig {
    ///             database_name: "tracker.db".to_string(),
    ///         }),
    ///         private: false,
    ///     },
    ///     udp_trackers: vec![
    ///         UdpTrackerConfig { bind_address: "0.0.0.0:6969".parse().unwrap() },
    ///     ],
    ///     http_trackers: vec![
    ///         HttpTrackerConfig { bind_address: "0.0.0.0:7070".parse().unwrap(), tls: None },
    ///     ],
    ///     http_api: HttpApiConfig {
    ///         bind_address: "0.0.0.0:1212".parse().unwrap(),
    ///         admin_token: "MyAccessToken".to_string().into(),
    ///         tls: None,
    ///     },
    ///     health_check_api: HealthCheckApiConfig {
    ///         bind_address: "127.0.0.1:1313".parse().unwrap(),
    ///     },
    /// };
    ///
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), TrackerConfigError> {
        let bindings = self.collect_bindings();
        Self::check_for_conflicts(bindings)
    }

    /// Checks for socket address conflicts in the collected bindings
    ///
    /// Examines the binding map to find any addresses that have multiple
    /// services attempting to use them with the same protocol.
    ///
    /// # Errors
    ///
    /// Returns `TrackerConfigError::DuplicateSocketAddress` if any binding
    /// address is shared by multiple services.
    fn check_for_conflicts(
        bindings: HashMap<BindingAddress, Vec<String>>,
    ) -> Result<(), TrackerConfigError> {
        for (binding, services) in bindings {
            if services.len() > 1 {
                return Err(TrackerConfigError::DuplicateSocketAddress {
                    address: *binding.socket(),
                    protocol: binding.protocol(),
                    services,
                });
            }
        }

        Ok(())
    }

    /// Collects all binding addresses with their service names
    ///
    /// Creates a map of binding addresses (socket + protocol) to service names.
    /// This allows identifying which services are attempting to bind to the same
    /// socket address with the same protocol.
    fn collect_bindings(&self) -> HashMap<BindingAddress, Vec<String>> {
        let mut bindings: HashMap<BindingAddress, Vec<String>> = HashMap::new();

        // Add UDP trackers
        Self::register_trackers(
            &mut bindings,
            &self.udp_trackers,
            Protocol::Udp,
            "UDP Tracker",
        );

        // Add HTTP trackers
        Self::register_trackers(
            &mut bindings,
            &self.http_trackers,
            Protocol::Tcp,
            "HTTP Tracker",
        );

        // Add HTTP API
        Self::register_binding(
            &mut bindings,
            self.http_api.bind_address,
            Protocol::Tcp,
            "HTTP API",
        );

        // Add Health Check API
        Self::register_binding(
            &mut bindings,
            self.health_check_api.bind_address,
            Protocol::Tcp,
            "Health Check API",
        );

        bindings
    }

    /// Registers multiple tracker instances in the bindings map
    ///
    /// Creates numbered service names for each tracker instance (e.g., "UDP Tracker #1").
    fn register_trackers<T>(
        bindings: &mut HashMap<BindingAddress, Vec<String>>,
        trackers: &[T],
        protocol: Protocol,
        service_name: &str,
    ) where
        T: HasBindAddress,
    {
        for (i, tracker) in trackers.iter().enumerate() {
            let service_label = format!("{service_name} #{}", i + 1);
            Self::register_binding(bindings, tracker.bind_address(), protocol, &service_label);
        }
    }

    /// Registers a single binding in the bindings map
    ///
    /// Associates the given service name with the socket address and protocol.
    fn register_binding(
        bindings: &mut HashMap<BindingAddress, Vec<String>>,
        address: SocketAddr,
        protocol: Protocol,
        service_name: &str,
    ) {
        let binding = BindingAddress::new(address, protocol);
        bindings
            .entry(binding)
            .or_default()
            .push(service_name.to_string());
    }

    /// Returns the HTTP API TLS domain if configured
    #[must_use]
    pub fn http_api_tls_domain(&self) -> Option<&str> {
        self.http_api.tls.as_ref().map(TlsConfig::domain)
    }

    /// Returns the HTTP API port number
    #[must_use]
    pub fn http_api_port(&self) -> u16 {
        self.http_api.bind_address.port()
    }

    /// Returns HTTP trackers that have TLS configured
    ///
    /// Returns a vector of tuples containing (domain, port) for each
    /// HTTP tracker that has TLS configuration.
    #[must_use]
    pub fn http_trackers_with_tls(&self) -> Vec<(&str, u16)> {
        self.http_trackers
            .iter()
            .filter_map(|tracker| {
                tracker
                    .tls
                    .as_ref()
                    .map(|tls| (tls.domain(), tracker.bind_address.port()))
            })
            .collect()
    }
}

/// Trait for types that have a bind address
///
/// Used for generic tracker registration in validation logic.
trait HasBindAddress {
    /// Returns the socket address this service binds to
    fn bind_address(&self) -> SocketAddr;
}

impl HasBindAddress for UdpTrackerConfig {
    fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }
}

impl HasBindAddress for HttpTrackerConfig {
    fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }
}

impl Default for TrackerConfig {
    /// Returns a default tracker configuration suitable for development and testing
    ///
    /// # Default Values
    ///
    /// - Database: `SQLite` with filename "tracker.db"
    /// - Mode: Public tracker (private = false)
    /// - UDP trackers: One instance on port 6969
    /// - HTTP trackers: One instance on port 7070
    /// - HTTP API: Bind address 0.0.0.0:1212
    /// - Admin token: `MyAccessToken`
    fn default() -> Self {
        Self {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "tracker.db".to_string(),
                }),
                private: false,
            },
            udp_trackers: vec![UdpTrackerConfig {
                bind_address: "0.0.0.0:6969".parse().expect("valid address"),
            }],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().expect("valid address"),
                tls: None,
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().expect("valid address"),
                admin_token: "MyAccessToken".to_string().into(),
                tls: None,
            },
            health_check_api: HealthCheckApiConfig {
                bind_address: "127.0.0.1:1313".parse().expect("valid address"),
            },
        }
    }
}

pub(crate) fn serialize_socket_addr<S>(addr: &SocketAddr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&addr.to_string())
}

pub(crate) fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_tracker_config() {
        let config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "tracker.db".to_string(),
                }),
                private: true,
            },
            udp_trackers: vec![UdpTrackerConfig {
                bind_address: "0.0.0.0:6868".parse().unwrap(),
            }],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().unwrap(),
                tls: None,
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "test_token".to_string().into(),
                tls: None,
            },
            health_check_api: HealthCheckApiConfig {
                bind_address: "127.0.0.1:1313".parse().unwrap(),
            },
        };

        assert_eq!(config.core.database.database_name(), "tracker.db");
        assert!(config.core.private);
        assert_eq!(config.udp_trackers.len(), 1);
        assert_eq!(config.http_trackers.len(), 1);
    }

    #[test]
    fn it_should_serialize_tracker_config() {
        let config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "test.db".to_string(),
                }),
                private: false,
            },
            udp_trackers: vec![],
            http_trackers: vec![],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "token123".to_string().into(),
                tls: None,
            },
            health_check_api: HealthCheckApiConfig {
                bind_address: "127.0.0.1:1313".parse().unwrap(),
            },
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["core"]["private"], false);
        assert_eq!(json["http_api"]["admin_token"], "token123");
    }

    #[test]
    fn it_should_create_default_tracker_config() {
        let config = TrackerConfig::default();

        // Verify default database configuration
        assert_eq!(config.core.database.database_name(), "tracker.db");
        assert_eq!(config.core.database.driver_name(), "sqlite3");

        // Verify public tracker mode
        assert!(!config.core.private);

        // Verify UDP trackers (1 instance)
        assert_eq!(config.udp_trackers.len(), 1);
        assert_eq!(
            config.udp_trackers[0].bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP trackers (1 instance)
        assert_eq!(config.http_trackers.len(), 1);
        assert_eq!(
            config.http_trackers[0].bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP API configuration
        assert_eq!(
            config.http_api.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.http_api.admin_token.expose_secret(), "MyAccessToken");
    }

    mod validation {
        use super::*;

        #[test]
        fn it_should_accept_valid_configuration_with_unique_addresses() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".parse().unwrap(),
                }],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    tls: None,
                }],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            assert!(config.validate().is_ok());
        }

        #[test]
        fn it_should_reject_duplicate_udp_tracker_ports() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![
                    UdpTrackerConfig {
                        bind_address: "0.0.0.0:7070".parse().unwrap(),
                    },
                    UdpTrackerConfig {
                        bind_address: "0.0.0.0:7070".parse().unwrap(),
                    },
                ],
                http_trackers: vec![],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            let result = config.validate();
            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Udp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"UDP Tracker #1".to_string()));
                assert!(services.contains(&"UDP Tracker #2".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_duplicate_http_tracker_ports() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![],
                http_trackers: vec![
                    HttpTrackerConfig {
                        bind_address: "0.0.0.0:7070".parse().unwrap(),
                        tls: None,
                    },
                    HttpTrackerConfig {
                        bind_address: "0.0.0.0:7070".parse().unwrap(),
                        tls: None,
                    },
                ],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            let result = config.validate();
            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_http_tracker_and_api_conflict() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    tls: None,
                }],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            let result = config.validate();
            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:7070".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"HTTP Tracker #1".to_string()));
                assert!(services.contains(&"HTTP API".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_reject_http_tracker_and_health_check_api_conflict() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:1313".parse().unwrap(),
                    tls: None,
                }],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "0.0.0.0:1313".parse().unwrap(),
                },
            };

            let result = config.validate();
            assert!(result.is_err());

            if let Err(TrackerConfigError::DuplicateSocketAddress {
                address,
                protocol,
                services,
            }) = result
            {
                assert_eq!(address, "0.0.0.0:1313".parse::<SocketAddr>().unwrap());
                assert_eq!(protocol, Protocol::Tcp);
                assert_eq!(services.len(), 2);
                assert!(services.contains(&"HTTP Tracker #1".to_string()));
                assert!(services.contains(&"Health Check API".to_string()));
            } else {
                panic!("Expected DuplicateSocketAddress error");
            }
        }

        #[test]
        fn it_should_allow_udp_and_http_on_same_port() {
            // This is valid because UDP and TCP use separate port spaces
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![UdpTrackerConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                }],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    tls: None,
                }],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            assert!(config.validate().is_ok());
        }

        #[test]
        fn it_should_allow_same_port_different_ips() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![],
                http_trackers: vec![
                    HttpTrackerConfig {
                        bind_address: "192.168.1.10:7070".parse().unwrap(),
                        tls: None,
                    },
                    HttpTrackerConfig {
                        bind_address: "192.168.1.20:7070".parse().unwrap(),
                        tls: None,
                    },
                ],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:1212".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            assert!(config.validate().is_ok());
        }

        #[test]
        fn it_should_provide_clear_error_message_with_fix_instructions() {
            let config = TrackerConfig {
                core: TrackerCoreConfig {
                    database: DatabaseConfig::Sqlite(SqliteConfig {
                        database_name: "tracker.db".to_string(),
                    }),
                    private: false,
                },
                udp_trackers: vec![],
                http_trackers: vec![HttpTrackerConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    tls: None,
                }],
                http_api: HttpApiConfig {
                    bind_address: "0.0.0.0:7070".parse().unwrap(),
                    admin_token: "token".to_string().into(),
                    tls: None,
                },
                health_check_api: HealthCheckApiConfig {
                    bind_address: "127.0.0.1:1313".parse().unwrap(),
                },
            };

            let error = config.validate().unwrap_err();
            let error_message = error.to_string();

            // Verify brief error message contains essential information
            assert!(error_message.contains("Socket address conflict"));
            assert!(error_message.contains("'HTTP Tracker #1'"));
            assert!(error_message.contains("'HTTP API'"));
            assert!(error_message.contains("0.0.0.0:7070"));
            assert!(error_message.contains("TCP"));
            assert!(error_message.contains("Tip: Assign different port numbers"));

            // Verify detailed help contains comprehensive troubleshooting
            let help = error.help();
            assert!(help.contains("Socket Address Conflict - Detailed Troubleshooting"));
            assert!(help.contains("Conflicting services:"));
            assert!(help.contains("HTTP Tracker #1"));
            assert!(help.contains("HTTP API"));
            assert!(help.contains("Why this fails:"));
            assert!(help.contains("How to fix:"));
            assert!(help.contains("docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md"));
        }
    }
}
