//! Template and configuration rendering steps
//!
//! This module contains steps that handle template and configuration generation
//! for the deployment system. These steps prepare configuration files and
//! templates with dynamic content for deployment.
//!
//! ## Available Steps
//!
//! - `ansible_templates` - Ansible template rendering with runtime variables
//! - `opentofu_templates` - `OpenTofu` template rendering for infrastructure
//!
//! ## Key Features
//!
//! - Dynamic template rendering with deployment-specific variables
//! - Support for multiple template engines and formats
//! - Integration with the step-based deployment architecture
//! - Comprehensive error handling for template processing
//!
//! These steps are essential for generating configuration files that contain
//! runtime information like IP addresses, SSH keys, and deployment settings.

pub mod ansible_templates;
pub mod opentofu_templates;

pub use ansible_templates::{RenderAnsibleTemplatesError, RenderAnsibleTemplatesStep};
pub use opentofu_templates::RenderOpenTofuTemplatesStep;
