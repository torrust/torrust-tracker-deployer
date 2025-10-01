//! `OpenTofu` LXD template wrappers
//!
//! Contains template wrappers for LXD-specific configuration files.
//!
//! - `cloud_init` - templates/tofu/lxd/cloud-init.yml.tera (with runtime variables: `ssh_public_key`)
//! - `variables` - templates/tofu/lxd/variables.tfvars.tera (with runtime variables: `instance_name`)

pub mod cloud_init;
pub mod variables;

pub use cloud_init::{
    CloudInitContext, CloudInitContextBuilder, CloudInitContextError, CloudInitTemplate,
};

pub use variables::{
    VariablesContext, VariablesContextBuilder, VariablesContextError, VariablesTemplate,
};
