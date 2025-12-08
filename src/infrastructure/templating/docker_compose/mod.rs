//! Docker Compose integration for application deployment
//!
//! This module provides Docker Compose-specific functionality for the deployment system,
//! including template rendering for Docker Compose configuration files.
//!
//! ## Components
//!
//! - `template` - Template rendering functionality for Docker Compose files
//!
//! Note: Unlike Ansible and Tofu, Docker Compose currently only uses static templates
//! (no Tera variable substitution). If dynamic templates are needed in the future,
//! the template module can be extended similar to Ansible.

pub mod template;

pub use template::{DockerComposeProjectGenerator, DockerComposeProjectGeneratorError};

/// Subdirectory name for Docker Compose-related files within the build directory.
///
/// Docker Compose files will be rendered to `build_dir/{DOCKER_COMPOSE_SUBFOLDER}/`.
pub const DOCKER_COMPOSE_SUBFOLDER: &str = "docker-compose";
