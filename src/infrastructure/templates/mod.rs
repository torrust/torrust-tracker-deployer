//! Template System for Configuration File Generation
//!
//! This module provides infrastructure for generating configuration file templates
//! that users can fill out to create deployment environments. It handles embedded
//! template resources and file system operations for template generation.
//!
//! ## Key Features
//!
//! - Embedded JSON configuration templates
//! - Async file generation with proper directory creation
//! - Comprehensive error handling with actionable guidance
//! - Extensible architecture for future template formats
//!
//! ## Usage Example
//!
//! ```rust
//! use torrust_tracker_deployer_lib::infrastructure::templates::{TemplateProvider, TemplateType};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = TemplateProvider::new();
//!
//! // Generate template at specific path
//! provider.generate_template(
//!     TemplateType::Json,
//!     Path::new("./environment-config.json")
//! ).await?;
//!
//! // Or generate with default filename in a directory
//! let path = provider.generate_template_in_directory(
//!     TemplateType::Json,
//!     Path::new("./configs")
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! - `provider` - High-level template generation API
//! - `embedded` - Embedded template resources (compile-time)
//! - `errors` - Template-specific error types with detailed help

pub mod embedded;
pub mod errors;
pub mod provider;

// Re-export commonly used types
pub use embedded::EmbeddedTemplates;
pub use errors::TemplateError;
pub use provider::{TemplateProvider, TemplateType};
