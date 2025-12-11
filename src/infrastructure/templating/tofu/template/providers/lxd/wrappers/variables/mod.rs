//! # `OpenTofu` Variables Templates
//!
//! Template wrappers for rendering `variables.tfvars.tera` with dynamic instance naming.
//!
//! This module provides the `VariablesTemplate` and `VariablesContext` for validating and rendering `OpenTofu`
//! variable files with runtime context injection, specifically for parameterizing
//! instance names in LXD infrastructure provisioning.

pub mod context;
mod variables_template;

pub use crate::infrastructure::templating::tofu::template::common::wrappers::VariablesTemplateError;
pub use context::{VariablesContext, VariablesContextBuilder, VariablesContextError};
pub use variables_template::VariablesTemplate;
