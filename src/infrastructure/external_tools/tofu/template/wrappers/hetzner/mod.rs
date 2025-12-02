//! `OpenTofu` Hetzner Cloud template wrappers
//!
//! Contains template wrappers for Hetzner Cloud-specific configuration files.
//!
//! - `cloud_init` - templates/tofu/hetzner/cloud-init.yml.tera (with runtime variables: `ssh_public_key`, `username`)
//! - `variables` - templates/tofu/hetzner/variables.tfvars.tera (with runtime variables: `hcloud_api_token`, `instance_name`, etc.)

pub mod cloud_init;
pub mod variables;

pub use cloud_init::{
    CloudInitContext, CloudInitContextBuilder, CloudInitContextError, CloudInitTemplate,
};

pub use variables::{
    VariablesContext, VariablesContextBuilder, VariablesContextError, VariablesTemplate,
};
