//! Torrust Tracker Deploy
//!
//! Main library for torrust-tracker-deploy with template-based configuration system.
//!
//! ## Template System
//! - `template` - Template rendering system with modular organization:
//!   - `renderer` - `TemplateRenderer` trait definition
//!   - `engine` - `TemplateEngine` implementation  
//!   - `context` - Template context types
//!   - `utils` - Utility functions
//!   - `wrappers` - Concrete template implementations organized by directory:
//!     - `ansible` - Wrappers for templates/ansible/ files
//!     - `tofu` - Wrappers for templates/tofu/ files
//!
//! Linting functionality has been moved to its own package: packages/linting

pub mod template;
