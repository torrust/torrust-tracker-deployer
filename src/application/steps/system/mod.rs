/*!
 * System Steps
 *
 * This module contains steps that manage system-level configuration and setup.
 * These steps handle OS-level operations, system services, and base system configuration.
 *
 * Current steps:
 * - Cloud-init completion waiting
 * - Automatic security updates configuration
 *
 * Future steps may include:
 * - User account setup and management
 * - Firewall configuration
 * - Log rotation configuration
 * - System service management
 */

pub mod configure_security_updates;
pub mod wait_cloud_init;

pub use configure_security_updates::ConfigureSecurityUpdatesStep;
pub use wait_cloud_init::WaitForCloudInitStep;
