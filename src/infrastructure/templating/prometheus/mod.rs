//! Prometheus integration for metrics collection
//!
//! This module provides Prometheus-specific functionality for the deployment system,
//! including template rendering for Prometheus configuration files.
//!
//! ## Components
//!
//! - `template` - Template rendering functionality for Prometheus configuration

pub mod template;

pub use template::PrometheusContext;

/// Subdirectory name for Prometheus-related files within the build directory.
///
/// Prometheus configuration files will be rendered to `build_dir/storage/prometheus/etc/`.
pub const PROMETHEUS_SUBFOLDER: &str = "storage/prometheus/etc";
