//! Tracker port configuration for Docker Compose

// External crates
use serde::Serialize;

/// Tracker port configuration
#[derive(Serialize, Debug, Clone)]
pub struct TrackerPorts {
    /// UDP tracker ports
    pub udp_tracker_ports: Vec<u16>,
    /// HTTP tracker ports
    pub http_tracker_ports: Vec<u16>,
    /// HTTP API port
    pub http_api_port: u16,
}
