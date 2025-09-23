//! `OpenTofu` template functionality
//!
//! This module provides template-related functionality for `OpenTofu`,
//! including specialized renderers for different types of configuration files.

pub mod renderer;

pub use renderer::cloud_init::{CloudInitTemplateError, CloudInitTemplateRenderer};
pub use renderer::{ProvisionTemplateError, TofuTemplateRenderer};
