//! Tracker configuration domain types
//!
//! This module defines tracker-specific configuration used for deploying
//! and configuring the Torrust Tracker application.
//!
//! # Module Structure
//!
//! - `config` - Main `TrackerConfig` and component configurations (includes database)
//! - `binding_address` - Socket binding address with protocol information
//! - `protocol` - Network protocol types (UDP, TCP)
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
//! let config = TrackerConfig::new(
//!     TrackerCoreConfig::new(
//!         DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
//!         false,
//!     ),
//!     vec![
//!         UdpTrackerConfig::new("0.0.0.0:6868".parse().unwrap(), None).unwrap(),
//!     ],
//!     vec![
//!         HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).unwrap(),
//!     ],
//!     HttpApiConfig::new(
//!         "0.0.0.0:1212".parse().unwrap(),
//!         "MyToken".to_string().into(),
//!         None,
//!         false,
//!     ).expect("valid config"),
//!     HealthCheckApiConfig::new(
//!         "127.0.0.1:1313".parse().unwrap(),
//!         None,
//!         false,
//!     ).expect("valid config"),
//! ).expect("valid tracker config");
//! ```

mod binding_address;
pub mod config;
mod protocol;

pub use binding_address::BindingAddress;
pub use config::{
    is_localhost, DatabaseConfig, HealthCheckApiConfig, HealthCheckApiConfigError, HttpApiConfig,
    HttpApiConfigError, HttpTrackerConfig, HttpTrackerConfigError, MysqlConfig, MysqlConfigError,
    SqliteConfig, SqliteConfigError, TrackerConfig, TrackerConfigError, TrackerCoreConfig,
    UdpTrackerConfig, UdpTrackerConfigError,
};
pub use protocol::{Protocol, ProtocolParseError};
