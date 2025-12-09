use serde::Serialize;
use thiserror::Error;

use crate::domain::tracker::TrackerConfig;

/// Errors that can occur when creating an `AnsibleVariablesContext`
#[derive(Debug, Error)]
pub enum AnsibleVariablesContextError {
    /// Invalid SSH port
    #[error("Invalid SSH port: {0}")]
    InvalidSshPort(#[from] crate::infrastructure::templating::ansible::template::wrappers::inventory::context::AnsiblePortError),
}

/// Context for rendering the variables.yml.tera template
///
/// This context contains system configuration variables used across
/// Ansible playbooks (but NOT inventory connection variables).
#[derive(Serialize, Debug, Clone)]
pub struct AnsibleVariablesContext {
    /// SSH port to configure in firewall and other services
    ssh_port: u16,

    /// UDP tracker ports extracted from tracker configuration
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tracker_udp_ports: Vec<u16>,

    /// HTTP tracker ports extracted from tracker configuration
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tracker_http_ports: Vec<u16>,

    /// Tracker HTTP API port
    #[serde(skip_serializing_if = "Option::is_none")]
    tracker_api_port: Option<u16>,
}

impl AnsibleVariablesContext {
    /// Creates a new context with the specified SSH port and optional tracker configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the SSH port is invalid (0 or out of range)
    pub fn new(ssh_port: u16, tracker_config: Option<&TrackerConfig>) -> Result<Self, AnsibleVariablesContextError> {
        // Validate SSH port using existing validation
        crate::infrastructure::templating::ansible::template::wrappers::inventory::context::AnsiblePort::new(ssh_port)?;

        let (tracker_udp_ports, tracker_http_ports, tracker_api_port) =
            Self::extract_tracker_ports(tracker_config);

        Ok(Self {
            ssh_port,
            tracker_udp_ports,
            tracker_http_ports,
            tracker_api_port,
        })
    }

    /// Extract port numbers from tracker configuration
    ///
    /// Returns a tuple of (udp_ports, http_ports, api_port)
    fn extract_tracker_ports(tracker_config: Option<&TrackerConfig>) -> (Vec<u16>, Vec<u16>, Option<u16>) {
        let Some(config) = tracker_config else {
            return (Vec::new(), Vec::new(), None);
        };

        // Extract UDP tracker ports
        let udp_ports: Vec<u16> = config
            .udp_trackers
            .iter()
            .filter_map(|tracker| Self::extract_port(&tracker.bind_address))
            .collect();

        // Extract HTTP tracker ports
        let http_ports: Vec<u16> = config
            .http_trackers
            .iter()
            .filter_map(|tracker| Self::extract_port(&tracker.bind_address))
            .collect();

        // Extract HTTP API port (hardcoded to 1212 for now - can be made configurable later)
        let api_port = Some(1212);

        (udp_ports, http_ports, api_port)
    }

    /// Helper function to extract port from bind_address (e.g., "0.0.0.0:6868" -> 6868)
    fn extract_port(bind_address: &str) -> Option<u16> {
        bind_address.split(':').nth(1)?.parse().ok()
    }

    /// Get the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    /// Get the UDP tracker ports
    #[must_use]
    pub fn tracker_udp_ports(&self) -> &[u16] {
        &self.tracker_udp_ports
    }

    /// Get the HTTP tracker ports
    #[must_use]
    pub fn tracker_http_ports(&self) -> &[u16] {
        &self.tracker_http_ports
    }

    /// Get the tracker API port
    #[must_use]
    pub fn tracker_api_port(&self) -> Option<u16> {
        self.tracker_api_port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_valid_ssh_port() {
        let context = AnsibleVariablesContext::new(22, None).unwrap();
        assert_eq!(context.ssh_port(), 22);
        assert!(context.tracker_udp_ports().is_empty());
        assert!(context.tracker_http_ports().is_empty());
        assert_eq!(context.tracker_api_port(), None);
    }

    #[test]
    fn it_should_create_context_with_custom_ssh_port() {
        let context = AnsibleVariablesContext::new(2222, None).unwrap();
        assert_eq!(context.ssh_port(), 2222);
    }

    #[test]
    fn it_should_create_context_with_high_port() {
        let context = AnsibleVariablesContext::new(65535, None).unwrap();
        assert_eq!(context.ssh_port(), 65535);
    }

    #[test]
    fn it_should_fail_with_port_zero() {
        let result = AnsibleVariablesContext::new(0, None);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid SSH port"));
    }

    #[test]
    fn it_should_implement_clone() {
        let context1 = AnsibleVariablesContext::new(22, None).unwrap();
        let context2 = context1.clone();
        assert_eq!(context1.ssh_port(), context2.ssh_port());
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = AnsibleVariablesContext::new(8022, None).unwrap();
        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("\"ssh_port\":8022"));
    }

    #[test]
    fn it_should_display_error_message_correctly() {
        let error = AnsibleVariablesContext::new(0, None).unwrap_err();
        let error_msg = format!("{error}");
        assert!(error_msg.contains("Invalid SSH port"));
        assert!(error_msg.contains("Invalid port number: 0"));
    }

    #[test]
    fn it_should_extract_tracker_ports_from_config() {
        use crate::domain::tracker::{ DatabaseConfig, HttpApiConfig, HttpTrackerConfig, TrackerCoreConfig, UdpTrackerConfig};

        let tracker_config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6868".to_string(),
                },
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".to_string(),
                },
            ],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiConfig {
                admin_token: "MyAccessToken".to_string(),
            },
        };

        let context = AnsibleVariablesContext::new(22, Some(&tracker_config)).unwrap();

        assert_eq!(context.tracker_udp_ports(), &[6868, 6969]);
        assert_eq!(context.tracker_http_ports(), &[7070]);
        assert_eq!(context.tracker_api_port(), Some(1212));
    }

    #[test]
    fn it_should_handle_empty_tracker_lists() {
        use crate::domain::tracker::{DatabaseConfig, HttpApiConfig, TrackerCoreConfig};

        let tracker_config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: true,
            },
            udp_trackers: vec![],
            http_trackers: vec![],
            http_api: HttpApiConfig {
                admin_token: "Token123".to_string(),
            },
        };

        let context = AnsibleVariablesContext::new(22, Some(&tracker_config)).unwrap();

        assert!(context.tracker_udp_ports().is_empty());
        assert!(context.tracker_http_ports().is_empty());
        assert_eq!(context.tracker_api_port(), Some(1212));
    }

    #[test]
    fn it_should_skip_invalid_bind_addresses() {
        use crate::domain::tracker::{DatabaseConfig, HttpApiConfig, HttpTrackerConfig, TrackerCoreConfig, UdpTrackerConfig};

        let tracker_config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![
                UdpTrackerConfig {
                    bind_address: "invalid".to_string(), // Invalid format
                },
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".to_string(), // Valid
                },
            ],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "no_port_here".to_string(), // Invalid format
            }],
            http_api: HttpApiConfig {
                admin_token: "Token".to_string(),
            },
        };

        let context = AnsibleVariablesContext::new(22, Some(&tracker_config)).unwrap();

        // Only valid port should be extracted
        assert_eq!(context.tracker_udp_ports(), &[6969]);
        assert!(context.tracker_http_ports().is_empty());
    }
}