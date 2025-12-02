//! `OpenTofu` template functionality
//!
//! This module provides template-related functionality for `OpenTofu`,
//! including specialized renderers for different types of configuration files
//! and template wrappers for type-safe context management.

pub mod common;
pub mod providers;

pub use common::renderer::cloud_init::{CloudInitTemplateError, CloudInitTemplateRenderer};
pub use common::renderer::{ProvisionTemplateError, TofuTemplateRenderer};
