//! Docker Compose template functionality
//!
//! This module provides template-related functionality for Docker Compose,
//! including the template renderer and wrappers for dynamic templates.
//!
//! ## Components
//!
//! - `renderer` - Template renderer for Docker Compose configuration files
//! - `wrappers` - Template wrappers for .tera files that need variable substitution

pub mod renderer;
pub mod wrappers;

pub use renderer::{DockerComposeProjectGenerator, DockerComposeProjectGeneratorError};
