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
//! Linting functionality has been moved to its own package: packages/linting

pub mod template;
