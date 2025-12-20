//! Grafana Template Rendering
//!
//! Provides template rendering capabilities for Grafana provisioning configuration.
//!
//! ## Components
//!
//! - `renderer` - Project generator and template renderers
//! - `wrapper` - Context and template wrappers for Tera templates

pub mod renderer;
pub mod wrapper;

pub use wrapper::datasource::{DatasourceContext, DatasourceTemplate};
