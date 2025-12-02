//! # Hetzner Cloud `OpenTofu` Variables Templates
//!
//! Template wrappers for rendering `variables.tfvars.tera` with Hetzner Cloud-specific configuration.
//!
//! This module provides the `VariablesTemplate` and `VariablesContext` for validating and rendering `OpenTofu`
//! variable files with runtime context injection, specifically for parameterizing
//! Hetzner Cloud infrastructure provisioning.

pub mod context;
mod variables_template;

pub use crate::infrastructure::external_tools::tofu::template::common::wrappers::VariablesTemplateError;
pub use context::{VariablesContext, VariablesContextBuilder, VariablesContextError};
pub use variables_template::VariablesTemplate;
