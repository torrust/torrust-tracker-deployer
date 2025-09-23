//! Template rendering system for configuration files
//!
//! This module provides core template domain models and business logic using Tera.
//! It supports sequential template resolution where different templates are resolved
//! at different points in the deployment workflow.
//!
//! ## Module Structure
//!
//! - `engine` - `TemplateEngine` implementation using Tera
//! - `file` - Template file utilities
//! - `file_ops` - File operation utilities
//! - `embedded` - Embedded template management for distribution

pub mod embedded;
pub mod engine;
pub mod file;
pub mod file_ops;

// Re-export commonly used items
pub use embedded::{TemplateManager, TemplateManagerError};
pub use engine::{TemplateEngine, TemplateEngineError};
pub use file_ops::{copy_file_with_dir_creation, write_file_with_dir_creation, FileOperationError};
