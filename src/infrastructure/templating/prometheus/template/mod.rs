//! Prometheus template functionality
//!
//! This module provides template-related functionality for Prometheus configuration,
//! including wrappers for dynamic templates.

pub mod renderer;
pub mod wrapper;

pub use renderer::{
    PrometheusConfigRenderer, PrometheusProjectGenerator, PrometheusProjectGeneratorError,
};
pub use wrapper::PrometheusContext;
