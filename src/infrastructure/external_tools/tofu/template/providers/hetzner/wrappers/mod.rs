//! `OpenTofu` Hetzner Cloud template wrappers
//!
//! Contains template wrappers for Hetzner Cloud-specific configuration files.
//!
//! - `variables` - templates/tofu/hetzner/variables.tfvars.tera (with runtime variables: `hcloud_api_token`, `instance_name`, etc.)
//!
//! Note: cloud-init wrapper has been moved to `common::wrappers::cloud_init` since
//! the same cloud-init template is used by all providers.

pub mod variables;

pub use variables::{
    VariablesContext, VariablesContextBuilder, VariablesContextError, VariablesTemplate,
};
