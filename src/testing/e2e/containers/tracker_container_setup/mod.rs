//! Tracker container setup for E2E testing
//!
//! Provides types for managing tracker container configurations and
//! runtime state in E2E tests.

mod container_ports;
mod tracker_ports;

// Export from local modules
pub use container_ports::ContainerPorts;
pub use tracker_ports::TrackerPorts;

// Temporarily re-export remaining types from the original module
pub use super::tracker_ports::E2eConfigEnvironment;
pub use super::tracker_ports::E2eRuntimeEnvironment;
