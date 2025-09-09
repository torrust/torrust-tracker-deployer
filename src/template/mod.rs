//! Template rendering system for configuration files
//!
//! This module provides template rendering using Tera.
//! It supports staged template resolution where different templates are resolved
//! at different lifecycle stages (e.g., static `OpenTofu` templates first, then
//! dynamic Ansible templates after VMs are provisioned and their IP addresses known).
//!
//! ## Module Structure
//!
//! - `engine` - `TemplateEngine` implementation using Tera
//! - `file` - Template file utilities
//! - `file_ops` - File operation utilities
//! - `wrappers` - Concrete template wrapper implementations

pub mod engine;
pub mod file;
pub mod file_ops;
pub mod wrappers;

// Re-export commonly used items
pub use engine::{TemplateEngine, TemplateEngineError};
pub use file_ops::{copy_file_with_dir_creation, write_file_with_dir_creation, FileOperationError};
