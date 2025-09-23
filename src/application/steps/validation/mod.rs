//! Validation and verification steps
//!
//! This module contains steps that perform testing, validation, and verification
//! operations to ensure that deployments are working correctly and meet requirements.
//! These steps validate various aspects of the deployed infrastructure and services.
//!
//! ## Available Steps
//!
//! - `cloud_init` - Cloud-init completion validation
//! - `docker` - Docker installation and functionality validation  
//! - `docker_compose` - Docker Compose installation and functionality validation
//!
//! ## Key Features
//!
//! - Comprehensive deployment validation and testing
//! - Service health and functionality verification
//! - Integration with remote action validation system
//! - Detailed error reporting for validation failures
//!
//! These steps ensure that the complete deployment workflow has resulted in
//! a functional environment ready for application deployment and operation.

pub mod cloud_init;
pub mod docker;
pub mod docker_compose;

pub use cloud_init::ValidateCloudInitCompletionStep;
pub use docker::ValidateDockerInstallationStep;
pub use docker_compose::ValidateDockerComposeInstallationStep;
