/*!
 * System Steps
 *
 * This module contains steps that manage system-level configuration and setup.
 * These steps handle OS-level operations, system services, and base system configuration.
 *
 * Current steps:
 * - Cloud-init completion waiting
 * - Automatic security updates configuration
 * - UFW firewall configuration (SSH access only)
 * - Backup crontab installation
 *
 * Note: Tracker service ports are controlled via Docker port bindings in docker-compose,
 * not through UFW rules. Docker bypasses UFW for published container ports.
 * See ADR: docs/decisions/docker-ufw-firewall-security-strategy.md
 *
 * Future steps may include:
 * - User account setup and management
 * - Log rotation configuration
 * - System service management
 */

pub mod configure_firewall;
pub mod configure_security_updates;
pub mod install_backup_crontab;
pub mod wait_cloud_init;

pub use configure_firewall::ConfigureFirewallStep;
pub use configure_security_updates::ConfigureSecurityUpdatesStep;
pub use install_backup_crontab::InstallBackupCrontabStep;
pub use wait_cloud_init::WaitForCloudInitStep;
