//! Infrastructure provider types
//!
//! This module defines the `Provider` enum and provider-specific configuration
//! domain types. These are core business concepts used throughout the codebase.
//!
//! # Module Structure
//!
//! Each provider has its own submodule for extensibility:
//! - `lxd` - LXD local development provider configuration
//! - `hetzner` - Hetzner cloud production provider configuration
//!
//! # Layer Separation
//!
//! - **Domain types** (this module): `Provider`, `ProviderConfig`, `LxdConfig`, `HetznerConfig`
//!   - Use validated domain types (e.g., `ProfileName`)
//!   - Represent semantic meaning of configuration
//!
//! - **Application config types** (`application::command_handlers::create::config::provider`):
//!   - `ProviderSection`, `LxdProviderSection`, `HetznerProviderSection`
//!   - Use raw primitives (e.g., `String`)
//!   - Handle JSON deserialization and conversion to domain types
//!
//! # Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::domain::provider::{Provider, ProviderConfig, LxdConfig};
//! use torrust_tracker_deployer_lib::domain::ProfileName;
//!
//! // Create a provider configuration
//! let config = ProviderConfig::Lxd(LxdConfig {
//!     profile_name: ProfileName::new("torrust-profile").unwrap(),
//! });
//!
//! // Access provider information
//! assert_eq!(config.provider(), Provider::Lxd);
//! assert_eq!(config.provider_name(), "lxd");
//! ```

mod config;
mod hetzner;
mod lxd;
mod provider_type;

pub use config::ProviderConfig;
pub use hetzner::HetznerConfig;
pub use lxd::LxdConfig;
pub use provider_type::Provider;
