/*!
 * System Steps
 *
 * This module contains steps that manage system-level configuration and setup.
 * These steps handle OS-level operations, system services, and base system configuration.
 *
 * Current steps:
 * - Cloud-init completion waiting
 *
 * Future steps may include:
 * - System updates and security patches
 * - User account setup and management
 * - Firewall configuration
 * - Log rotation configuration
 * - System service management
 */

pub mod wait_cloud_init;

pub use wait_cloud_init::WaitForCloudInitStep;
