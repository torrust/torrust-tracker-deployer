//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `environment` - Environment module with entity, name validation, and state management
//!   - `environment::name` - Environment name validation and management
//!   - `environment::state` - State marker types and type erasure for environment state machine
//! - `instance_name` - LXD instance name validation and management
//! - `profile_name` - LXD profile name validation and management
//! - `provider` - Infrastructure provider types (LXD, Hetzner) and configuration
//! - `template` - Core template domain models and business logic

pub mod environment;
pub mod grafana;
pub mod https;
pub mod instance_name;
pub mod profile_name;
pub mod prometheus;
pub mod provider;
pub mod template;
pub mod tracker;

// Re-export commonly used domain types for convenience
pub use environment::{
    name::{EnvironmentName, EnvironmentNameError},
    state::{AnyEnvironmentState, StateTypeError},
    Environment,
};
pub use instance_name::{InstanceName, InstanceNameError};
pub use profile_name::{ProfileName, ProfileNameError};
pub use provider::{HetznerConfig, LxdConfig, Provider, ProviderConfig};
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
