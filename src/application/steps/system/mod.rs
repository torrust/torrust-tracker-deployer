/*!
 * System Steps
 *
 * This module contains steps that manage system-level configuration and setup.
 * These steps handle OS-level operations, system services, and base system configuration.
 *
 * Current steps:
 * - Cloud-init completion waiting
 * - Automatic security updates configuration
 * - UFW firewall configuration
 * - Tracker firewall configuration
 * - Grafana firewall configuration
 *
 * Future steps may include:
 * - User account setup and management
 * - Log rotation configuration
 * - System service management
 */

pub mod configure_firewall;
pub mod configure_grafana_firewall;
pub mod configure_security_updates;
pub mod configure_tracker_firewall;
pub mod wait_cloud_init;

pub use configure_firewall::ConfigureFirewallStep;
pub use configure_grafana_firewall::ConfigureGrafanaFirewallStep;
pub use configure_security_updates::ConfigureSecurityUpdatesStep;
pub use configure_tracker_firewall::ConfigureTrackerFirewallStep;
pub use wait_cloud_init::WaitForCloudInitStep;
