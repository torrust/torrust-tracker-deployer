//! `OpenTofu` template functionality
//!
//! This module provides template-related functionality for `OpenTofu`,
//! including specialized renderers for different types of configuration files
//! and template wrappers for type-safe context management.

pub mod renderer;
pub mod wrappers;

pub use renderer::cloud_init::{CloudInitTemplateError, CloudInitTemplateRenderer};
pub use renderer::{ProvisionTemplateError, TofuTemplateRenderer};
