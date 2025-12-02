//! `OpenTofu` integration for infrastructure provisioning
//!
//! This module provides `OpenTofu` (formerly Terraform) specific functionality for the
//! deployment system, including template rendering for infrastructure configuration files.
//!
//! ## Components
//!
//! - `template` - Template renderers and context wrappers for infrastructure configuration files
//!   - `TofuTemplateRenderer` - Handles generation of `OpenTofu` configuration files
//!   - `CloudInitTemplateRenderer` - Specialized collaborator for cloud-init.yml.tera templates
//!
//! Note: The `OpenTofu` adapter (`OpenTofuClient`) has been moved to `crate::adapters::tofu`

pub mod template;

pub use template::{CloudInitTemplateRenderer, TofuTemplateRenderer, TofuTemplateRendererError};

/// Subdirectory name for OpenTofu-related files within the build directory.
///
/// OpenTofu/Terraform configuration files and state will be managed
/// in `build_dir/{OPENTOFU_SUBFOLDER}/`. Example: "tofu/lxd".
pub const OPENTOFU_SUBFOLDER: &str = "tofu/lxd";
