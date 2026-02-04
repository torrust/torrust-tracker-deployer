//! Domain Layer (DDD)
//!
//! This module contains domain-specific business logic and entities.
//! It includes pure domain models independent of technical implementation details.
//!
//! ## Components
//!
//! - `backup` - Backup configuration domain types (cron schedule, retention)
//! - `caddy` - Caddy TLS reverse proxy service domain types
//! - `environment` - Environment module with entity, name validation, and state management
//!   - `environment::name` - Environment name validation and management
//!   - `environment::state` - State marker types and type erasure for environment state machine
//! - `instance_name` - LXD instance name validation and management
//! - `mysql` - `MySQL` database service domain types (distinct from tracker database config)
//! - `profile_name` - LXD profile name validation and management
//! - `provider` - Infrastructure provider types (LXD, Hetzner) and configuration
//! - `template` - Core template domain models and business logic
//! - `topology` - Docker Compose topology domain types (networks, services)

pub mod backup;
pub mod caddy;
pub mod environment;
pub mod grafana;
pub mod https;
pub mod instance_name;
pub mod mysql;
pub mod profile_name;
pub mod prometheus;
pub mod provider;
pub mod template;
pub mod topology;
pub mod tracker;

// Re-export commonly used domain types for convenience
pub use backup::{BackupConfig, CronSchedule, RetentionDays};
pub use caddy::CaddyConfig;
pub use environment::{
    name::{EnvironmentName, EnvironmentNameError},
    state::{AnyEnvironmentState, StateTypeError},
    Environment,
};
pub use instance_name::{InstanceName, InstanceNameError};
pub use mysql::MysqlServiceConfig;
pub use profile_name::{ProfileName, ProfileNameError};
pub use provider::{HetznerConfig, LxdConfig, Provider, ProviderConfig};
pub use template::{TemplateEngine, TemplateEngineError, TemplateManager, TemplateManagerError};
pub use topology::{DockerComposeTopology, Network, Service, ServiceTopology};
