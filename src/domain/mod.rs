//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `template` - Core template domain models and business logic

pub mod template;

// Re-export commonly used domain types for convenience
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
