//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `template` - Core template domain models and business logic
//! - `username` - Linux system username validation and management

pub mod template;
pub mod username;

// Re-export commonly used domain types for convenience
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
pub use username::{Username, UsernameError};
