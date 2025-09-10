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
//! - `ssh` - SSH command execution utilities with secure defaults
//! - `opentofu` - `OpenTofu` infrastructure management utilities
//! - `lxd` - LXD container management utilities
//!
//! Linting functionality has been moved to its own package: packages/linting

pub mod actions;
pub mod ansible;
pub mod command;
pub mod lxd;
pub mod opentofu;
pub mod ssh;
pub mod template;
