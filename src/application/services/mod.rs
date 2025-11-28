//! Application Services
//!
//! This module contains shared application services that can be used by multiple
//! command handlers. Services encapsulate reusable business logic that doesn't
//! belong to a specific command handler.
//!
//! ## Services
//!
//! - `AnsibleTemplateService` - Renders Ansible templates with runtime configuration

mod ansible_template_service;

pub use ansible_template_service::{AnsibleTemplateService, AnsibleTemplateServiceError};
