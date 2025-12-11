//! External Tools Integration - Application-Specific Configuration
//!
//! This module contains application-specific configuration and template rendering
//! for external CLI tools used in the deployment infrastructure.
//!
//! ## Architecture Note
//!
//! **Generic CLI Adapters** (thin wrappers for external tools) have been moved to
//! `crate::adapters`. This module now contains only **application-specific logic**
//! such as template rendering and context management.
//!
//! - **`crate::adapters`**: Generic CLI tool wrappers (reusable across projects)
//! - **This module**: Application-specific tool configuration and templates
//!
//! ## Components
//!
//! - `ansible` - Ansible configuration management integration
//!   - `template` - Template renderers for Ansible inventory and playbooks
//! - `docker_compose` - Docker Compose file management
//!   - `file_manager` - File manager for Docker Compose configuration files
//! - `tofu` - `OpenTofu` infrastructure provisioning integration
//!   - `template` - Template renderers for `OpenTofu` configuration files
//! - `tracker` - Torrust Tracker configuration management
//!   - `template` - Template renderers for Tracker configuration files
//!
//! ## Template Rendering
//!
//! Each tool module provides template renderers that:
//! - Generate tool-specific configuration files from `.tera` templates
//! - Manage template contexts with runtime values
//! - Handle template validation and error reporting

pub mod ansible;
pub mod docker_compose;
pub mod tofu;
pub mod tracker;
