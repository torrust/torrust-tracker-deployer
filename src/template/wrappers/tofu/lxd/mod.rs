//! `OpenTofu` LXD template wrappers
//!
//! Contains template wrappers for LXD-specific configuration files.
//!
//! - `cloud_init` - templates/tofu/lxd/cloud-init.yml.tera (with runtime variables: `ssh_public_key`)

pub mod cloud_init;

pub use cloud_init::{
    CloudInitContext, CloudInitContextBuilder, CloudInitContextError, CloudInitTemplate,
};
