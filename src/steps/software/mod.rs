//! Software installation and configuration steps
//!
//! This module contains steps that manage software installation and configuration
//! on deployed infrastructure. These steps handle the installation of third-party
//! software, packages, and tools required for the deployment environment.
//!
//! ## Available Steps
//!
//! - `docker` - Docker engine installation and configuration
//! - `docker_compose` - Docker Compose installation and setup
//!
//! ## Key Features
//!
//! - Automated software installation via Ansible playbooks
//! - Version management and compatibility checking
//! - Service configuration and startup management
//! - Integration with the step-based deployment architecture
//!
//! These steps ensure that the deployed infrastructure has all necessary
//! software components properly installed and configured for application deployment.

pub mod docker;
pub mod docker_compose;

pub use docker::InstallDockerStep;
pub use docker_compose::InstallDockerComposeStep;
