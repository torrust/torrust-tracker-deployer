//! `OpenTofu` integration for infrastructure provisioning
//!
//! This module provides `OpenTofu` (formerly Terraform) specific functionality for the
//! deployment system, including command-line wrapper and template rendering.
//!
//! ## Components
//!
//! - `adapter` - `OpenTofu` command-line tool wrapper (`OpenTofuClient`)
//! - `template` - Template renderers and context wrappers for infrastructure configuration files
//!   - `TofuTemplateRenderer` - Handles generation of `OpenTofu` configuration files
//!   - `CloudInitTemplateRenderer` - Specialized collaborator for cloud-init.yml.tera templates

pub mod adapter;
pub mod template;

pub use adapter::client::OpenTofuClient;
pub use template::{CloudInitTemplateRenderer, ProvisionTemplateError, TofuTemplateRenderer};

/// Subdirectory name for OpenTofu-related files within the build directory.
///
/// OpenTofu/Terraform configuration files and state will be managed
/// in `build_dir/{OPENTOFU_SUBFOLDER}/`. Example: "tofu/lxd".
pub const OPENTOFU_SUBFOLDER: &str = "tofu/lxd";
