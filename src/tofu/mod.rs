//! `OpenTofu` integration for infrastructure provisioning
//!
//! This module provides `OpenTofu` (formerly Terraform) specific functionality for the
//! deployment system, primarily focused on template rendering for infrastructure
//! configuration files.
//!
//! ## Key Components
//!
//! - `TofuTemplateRenderer` - Handles generation of `OpenTofu` configuration files
//! - Template processing for infrastructure definitions
//!
//! The module complements the `OpenTofu` command wrapper by providing the template

pub mod template_renderer;

pub use template_renderer::TofuTemplateRenderer;
