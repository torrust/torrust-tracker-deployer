/*!
 * Validation Steps
 *
 * This module contains steps that perform testing, validation, and verification operations.
 * These steps ensure that deployments are working correctly and meet requirements.
 *
 * Current steps:
 * - Cloud-init completion validation
 * - Docker installation validation
 * - Docker Compose installation validation
 *
 * Future steps may include:
 * - Tool validation (check command)
 * - Torrust Tracker functionality validation
 * - Deployment validation
 * - Database connectivity validation
 * - HTTP endpoint health checks
 * - Performance baseline validation
 * - Remote service validation
 */

pub mod cloud_init;
pub mod docker;
pub mod docker_compose;

pub use cloud_init::ValidateCloudInitCompletionStep;
pub use docker::ValidateDockerInstallationStep;
pub use docker_compose::ValidateDockerComposeInstallationStep;
