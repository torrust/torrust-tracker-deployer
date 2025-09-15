//! Torrust Tracker Deploy
//!
//! Main library for torrust-tracker-deploy with template-based configuration system.
//!
//! ## Template System
//! - `template` - Template rendering system with modular organization:
//!   - `engine` - `TemplateEngine` implementation  
//!   - `file` - Template file utilities
//!   - `file_ops` - File operation utilities
//!   - `wrappers` - Concrete template implementations organized by directory:
//!     - `ansible` - Wrappers for templates/ansible/ files
//!     - `tofu` - Wrappers for templates/tofu/ files
//!
//! ## Command Execution
//! - `command` - Command execution utilities with optional verbosity
//! - `command_wrappers` - Generic command wrappers for external tools:
//!   - `ssh` - SSH command execution utilities with secure defaults
//!   - `ansible` - Ansible configuration management utilities
//!   - `opentofu` - `OpenTofu` infrastructure management utilities
//!   - `lxd` - LXD container management utilities
//!
//! ## Template Renderers
//! - `ansible` - Ansible template rendering utilities for deployment stages
//! - `tofu` - `OpenTofu` template rendering utilities for infrastructure provisioning
//!
//! ## Configuration
//! - `config` - Configuration management for deployment environments
//! - `container` - Service container for dependency injection
//!
//! Linting functionality has been moved to its own package: packages/linting

pub mod actions;
pub mod ansible;
pub mod command;
pub mod command_wrappers;
pub mod commands;
pub mod config;
pub mod container;
pub mod steps;
pub mod template;
pub mod tofu;
