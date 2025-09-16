/*!
 * Software Steps
 *
 * This module contains steps that manage software installation and configuration.
 * These steps handle installation of third-party software, packages, and tools.
 *
 * Current steps:
 * - Docker installation
 * - Docker Compose installation
 *
 * Future steps may include:
 * - Monitoring agent installation
 * - Package manager operations
 * - Custom software installation
 * - Software version management
 */

pub mod docker;
pub mod docker_compose;

pub use docker::InstallDockerStep;
pub use docker_compose::InstallDockerComposeStep;
