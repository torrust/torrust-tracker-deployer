//! Prometheus configuration template wrapper
//!
//! This module provides the context for rendering the prometheus.yml.tera template.

pub mod context;

pub use context::PrometheusContext;
