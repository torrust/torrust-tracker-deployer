//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `environment` - Environment entity encapsulating all environment-specific configuration
//! - `environment_name` - Environment name validation and management
//! - `instance_name` - LXD instance name validation and management
//! - `template` - Core template domain models and business logic

pub mod environment;
pub mod environment_name;
pub mod instance_name;
pub mod template;

// Re-export commonly used domain types for convenience
pub use environment::Environment;
pub use environment_name::{EnvironmentName, EnvironmentNameError};
pub use instance_name::{InstanceName, InstanceNameError};
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
