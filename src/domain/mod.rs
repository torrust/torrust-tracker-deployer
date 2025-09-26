//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `instance_name` - LXD instance name validation and management
//! - `template` - Core template domain models and business logic

pub mod instance_name;
pub mod template;

// Re-export commonly used domain types for convenience
pub use instance_name::{InstanceName, InstanceNameError};
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
