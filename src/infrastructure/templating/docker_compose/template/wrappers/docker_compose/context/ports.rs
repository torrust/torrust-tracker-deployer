//! Tracker port configuration for Docker Compose

// External crates
use serde::Serialize;

/// Tracker port configuration
///
/// Contains all port information needed for Docker Compose port mappings.
/// Includes TLS status and pre-computed flags for template rendering.
#[derive(Serialize, Debug, Clone)]
pub struct TrackerPorts {
    /// UDP tracker ports (always exposed - UDP doesn't use TLS termination via Caddy)
    pub udp_tracker_ports: Vec<u16>,
    /// HTTP tracker ports without TLS (only these are exposed in Docker Compose)
    ///
    /// Ports with TLS enabled are handled by Caddy and NOT included here.
    pub http_tracker_ports_without_tls: Vec<u16>,
    /// HTTP API port
    pub http_api_port: u16,
    /// Whether the HTTP API has TLS enabled (port should not be exposed if true)
    #[serde(default)]
    pub http_api_has_tls: bool,
    /// Whether the tracker service needs a ports section at all
    ///
    /// Pre-computed flag: true if there are UDP ports, HTTP ports without TLS,
    /// or the API port is exposed (no TLS).
    #[serde(default)]
    pub needs_ports_section: bool,
}

impl TrackerPorts {
    /// Creates a new `TrackerPorts` with pre-computed flags
    ///
    /// # Arguments
    ///
    /// * `udp_tracker_ports` - UDP tracker ports (always exposed)
    /// * `http_tracker_ports_without_tls` - HTTP tracker ports that don't have TLS
    /// * `http_api_port` - The HTTP API port number
    /// * `http_api_has_tls` - Whether the API uses TLS (Caddy handles it)
    #[must_use]
    pub fn new(
        udp_tracker_ports: Vec<u16>,
        http_tracker_ports_without_tls: Vec<u16>,
        http_api_port: u16,
        http_api_has_tls: bool,
    ) -> Self {
        let needs_ports_section = !udp_tracker_ports.is_empty()
            || !http_tracker_ports_without_tls.is_empty()
            || !http_api_has_tls;

        Self {
            udp_tracker_ports,
            http_tracker_ports_without_tls,
            http_api_port,
            http_api_has_tls,
            needs_ports_section,
        }
    }
}
