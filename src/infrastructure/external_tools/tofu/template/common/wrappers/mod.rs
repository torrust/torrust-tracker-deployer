//! Common `OpenTofu` template wrappers shared across providers.
//!
//! Contains template wrappers that are used by all providers.
//!
//! - `cloud_init` - templates/tofu/common/cloud-init.yml.tera (with runtime variables: `ssh_public_key`, `username`)
//! - `errors` - Shared error types for template wrappers

pub mod cloud_init;
pub mod errors;

pub use cloud_init::{
    CloudInitContext, CloudInitContextBuilder, CloudInitContextError, CloudInitTemplate,
};
pub use errors::VariablesTemplateError;
