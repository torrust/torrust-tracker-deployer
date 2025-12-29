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
//!     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
//!     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig, HealthCheckApiConfig
//! };
//!
//! let config = TrackerConfig {
//!     core: TrackerCoreConfig {
//!         database: DatabaseConfig::Sqlite(SqliteConfig {
//!             database_name: "tracker.db".to_string(),
//!         }),
//!         private: false,
//!     },
//!     udp_trackers: vec![
//!         UdpTrackerConfig { bind_address: "0.0.0.0:6868".parse().unwrap() },
//!     ],
//!     http_trackers: vec![
//!         HttpTrackerConfig { bind_address: "0.0.0.0:7070".parse().unwrap() },
//!     ],
//!     http_api: HttpApiConfig {
//!         bind_address: "0.0.0.0:1212".parse().unwrap(),
//!         admin_token: "MyToken".to_string().into(),
//!     },
//!     health_check_api: HealthCheckApiConfig {
//!         bind_address: "127.0.0.1:1313".parse().unwrap(),
//!     },
//! };
//! ```

mod config;
mod database;

pub use config::{
    HealthCheckApiConfig, HttpApiConfig, HttpTrackerConfig, TrackerConfig, TrackerCoreConfig,
    UdpTrackerConfig,
};
pub use database::{DatabaseConfig, MysqlConfig, SqliteConfig};
