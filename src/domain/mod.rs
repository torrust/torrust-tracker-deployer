//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `config` - Configuration value objects and validation for environment creation
//! - `environment` - Environment module with entity, name validation, and state management
//!   - `environment::name` - Environment name validation and management
//!   - `environment::state` - State marker types and type erasure for environment state machine
//! - `instance_name` - LXD instance name validation and management
//! - `profile_name` - LXD profile name validation and management
//! - `template` - Core template domain models and business logic

pub mod config;
pub mod environment;
pub mod instance_name;
pub mod profile_name;
pub mod template;

// Re-export commonly used domain types for convenience
pub use environment::{
    name::{EnvironmentName, EnvironmentNameError},
    state::{AnyEnvironmentState, StateTypeError},
    Environment,
};
pub use instance_name::{InstanceName, InstanceNameError};
pub use profile_name::{ProfileName, ProfileNameError};
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
