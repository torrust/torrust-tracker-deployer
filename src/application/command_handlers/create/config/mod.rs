//! Configuration Module for Create Command
//!
//! This module provides configuration Data Transfer Objects (DTOs) and validation for
//! creating deployment environments. It sits at the boundary between external configuration
//! sources (JSON files, CLI arguments, etc.) and the internal domain model.
//!
//! ## Architecture
//!
//! The configuration DTOs in this module are specific to the create command and are distinct
//! from both the domain and adapter layers:
//!
//! - **Configuration DTOs** (`application::command_handlers::create::config`): String-based
//!   configuration objects that deserialize from external sources (JSON, TOML, CLI)
//! - **Domain Layer** (`domain::environment`): Strongly-typed domain entities
//!   with business validation
//! - **Adapter Layer** (`adapters::ssh`): Infrastructure-specific implementations
//!
//! ## Key Components
//!
//! ### Value Objects
//!
//! - `EnvironmentCreationConfig` - Top-level configuration for environment creation
//! - `SshCredentialsConfig` - SSH credentials configuration (config layer)
//! - `EnvironmentSection` - Environment-specific settings
//!
//! ### Provider Configuration
//!
//! Provider configuration is organized in the `provider` submodule:
//! - `ProviderSection` - Tagged enum for provider-specific settings
//! - `LxdProviderSection` - LXD provider configuration
//! - `HetznerProviderSection` - Hetzner provider configuration
//!
//! Note: `SshCredentialsConfig` (config layer) is distinct from
//! `adapters::ssh::SshCredentials` (adapter layer). The config version uses
//! strings for paths and usernames, while the adapter version uses domain types.
//!
//! ### Type Conversion
//!
//! Configuration objects provide `to_*` methods that convert string-based
//! configuration to strongly-typed domain objects:
//!
//! - String environment name → `EnvironmentName`
//! - String username → `Username`
//! - String paths → `PathBuf`
//!
//! ### Error Handling
//!
//! All errors implement the `.help()` method following the project's tiered
//! help system pattern, providing actionable guidance for resolving issues.
//!
//! ## Usage Example
//!
//! ```no_run
//! use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
//!     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
//!     ProviderSection, LxdProviderSection
//! };
//! use torrust_tracker_deployer_lib::domain::environment::EnvironmentParams;
//! use torrust_tracker_deployer_lib::domain::Environment;
//! use chrono::{TimeZone, Utc};
//!
//! // Deserialize configuration from JSON
//! let json = r#"{
//!     "environment": {
//!         "name": "dev"
//!     },
//!     "ssh_credentials": {
//!         "private_key_path": "fixtures/testing_rsa",
//!         "public_key_path": "fixtures/testing_rsa.pub"
//!     },
//!     "provider": {
//!         "provider": "lxd",
//!         "profile_name": "torrust-profile-dev"
//!     },
//!     "tracker": {
//!         "core": {
//!             "database": {
//!                 "driver": "sqlite3",
//!                 "database_name": "tracker.db"
//!             },
//!             "private": false
//!         },
//!         "udp_trackers": [
//!             {
//!                 "bind_address": "0.0.0.0:6969"
//!             }
//!         ],
//!         "http_trackers": [
//!             {
//!                 "bind_address": "0.0.0.0:7070"
//!             }
//!         ],
//!         "http_api": {
//!             "bind_address": "0.0.0.0:1212",
//!             "admin_token": "MyAccessToken"
//!         }
//!     }
//! }"#;
//!
//! let config: EnvironmentCreationConfig = serde_json::from_str(json)?;
//!
//! // Convert to validated domain parameters using TryInto
//! let params: EnvironmentParams = config.try_into()?;
//!
//! // Create domain entity using the factory pattern
//! let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
//! let working_dir = std::path::Path::new("/tmp/my-env");
//! let environment = Environment::create(params, working_dir, created_at)?;
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Validation Strategy
//!
//! Validation occurs in two phases:
//!
//! 1. **Format Validation** (during conversion):
//!    - Environment name format (via `EnvironmentName::new()`)
//!    - Username format (via `Username::new()`)
//!    - Path string to `PathBuf` conversion
//!
//! 2. **Business Validation** (during conversion):
//!    - SSH key file existence
//!    - SSH key file accessibility
//!    - Domain-specific business rules
//!
//! ## Design Principles
//!
//! - **Type Safety**: String-based config → strongly-typed domain objects
//! - **Single Responsibility**: Config objects only handle deserialization and conversion
//! - **Explicit Errors**: All validation errors are explicit enum variants with context
//! - **Actionable Feedback**: All errors provide `.help()` with troubleshooting steps
//! - **Clean Separation**: Config layer is distinct from domain and adapter layers

pub mod backup;
pub mod environment_config;
pub mod errors;
pub mod grafana;
pub mod https;
pub mod prometheus;
pub mod provider;
pub mod ssh_credentials_config;
pub mod tracker;
pub mod validated_params;

// Re-export commonly used types for convenience
pub use backup::BackupSection;
pub use environment_config::{EnvironmentCreationConfig, EnvironmentSection};
pub use errors::ConfigLoadError;
pub use errors::CreateConfigError;
pub use grafana::GrafanaSection;
pub use https::HttpsSection;
pub use prometheus::PrometheusSection;
pub use provider::{HetznerProviderSection, LxdProviderSection, ProviderSection};
pub use ssh_credentials_config::SshCredentialsConfig;

// Note: EnvironmentParams is now in domain layer (crate::domain::environment::EnvironmentParams)
// The validated_params module provides TryFrom<EnvironmentCreationConfig> for EnvironmentParams
