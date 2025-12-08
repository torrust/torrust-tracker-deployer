//! `OpenTofu` integration for infrastructure provisioning
//!
//! This module provides `OpenTofu` (formerly Terraform) specific functionality for the
//! deployment system, including template rendering for infrastructure configuration files.
//!
//! ## Components
//!
//! - `template` - Template renderers and context wrappers for infrastructure configuration files
//!   - `TofuProjectGenerator` - Handles generation of `OpenTofu` configuration files
//!   - `CloudInitRenderer` - Specialized collaborator for cloud-init.yml.tera templates
//!
//! Note: The `OpenTofu` adapter (`OpenTofuClient`) has been moved to `crate::adapters::tofu`

pub mod template;

pub use template::{CloudInitRenderer, TofuProjectGenerator, TofuProjectGeneratorError};
