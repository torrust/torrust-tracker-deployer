//! Error types for the create-config module.
//!
//! Two distinct error enums cover different failure stages:
//!
//! - [`ConfigLoadError`] — I/O and JSON-parse failures that occur **before** any
//!   domain validation (reading a file, deserializing JSON).
//! - [`CreateConfigError`] — domain validation failures that occur **after** the
//!   configuration has been successfully loaded.

pub mod create_config_error;
pub mod load_error;

pub use create_config_error::CreateConfigError;
pub use load_error::ConfigLoadError;
