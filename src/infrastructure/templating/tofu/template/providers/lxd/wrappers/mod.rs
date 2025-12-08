//! `OpenTofu` LXD template wrappers
//!
//! Contains template wrappers for LXD-specific configuration files.
//!
//! - `variables` - templates/tofu/lxd/variables.tfvars.tera (with runtime variables: `instance_name`)
//!
//! Note: cloud-init wrapper has been moved to `common::wrappers::cloud_init` since
//! the same cloud-init template is used by all providers.

pub mod variables;

pub use variables::{
    VariablesContext, VariablesContextBuilder, VariablesContextError, VariablesTemplate,
};
