//! Docker Compose template functionality
//!
//! This module provides template-related functionality for Docker Compose,
//! including the template renderer for static file management.
//!
//! ## Components
//!
//! - `renderer` - Template renderer for Docker Compose configuration files
//!
//! Note: Unlike Ansible, Docker Compose currently only uses static templates
//! (no Tera variable substitution). If dynamic templates are needed in the
//! future, a `wrappers` submodule can be added similar to Ansible.

pub mod renderer;

pub use renderer::{DockerComposeTemplateError, DockerComposeTemplateRenderer};
