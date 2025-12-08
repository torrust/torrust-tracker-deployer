//! Tracker configuration domain types
//!
//! This module defines tracker-specific configuration used for deploying
//! and configuring the Torrust Tracker application.
//!
//! # Module Structure
//!
//! - `config` - Main `TrackerConfig` and component configurations
//! - `database` - Database configuration (`SQLite`, `MySQL`)
//!
//! # Layer Separation
//!
//! - **Domain types** (this module): `TrackerConfig`, `DatabaseConfig`, etc.
//!   - Represent semantic meaning of tracker configuration
//!   - Used in environment user inputs
//!
//! # Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::domain::tracker::{
//!     TrackerConfig, TrackerCoreConfig, DatabaseConfig,
//!     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig
//! };
//!
//! let config = TrackerConfig {
//!     core: TrackerCoreConfig {
//!         database: DatabaseConfig::Sqlite {
//!             database_name: "tracker.db".to_string(),
//!         },
//!         private: false,
//!     },
//!     udp_trackers: vec![
//!         UdpTrackerConfig { bind_address: "0.0.0.0:6868".to_string() },
//!     ],
//!     http_trackers: vec![
//!         HttpTrackerConfig { bind_address: "0.0.0.0:7070".to_string() },
//!     ],
//!     http_api: HttpApiConfig {
//!         admin_token: "MyToken".to_string(),
//!     },
//! };
//! ```

mod config;
mod database;

pub use config::{
    HttpApiConfig, HttpTrackerConfig, TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
};
pub use database::DatabaseConfig;
