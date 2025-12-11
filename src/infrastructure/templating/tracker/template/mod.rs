//! Tracker template functionality
//!
//! This module provides template-related functionality for Torrust Tracker configuration,
//! including the template renderer for tracker.toml files.
//!
//! ## Components
//!
//! - `renderer` - Template renderer for Tracker configuration files
//! - `wrapper` - Context and Template wrapper types

pub mod renderer;
pub mod wrapper;

pub use renderer::{TrackerConfigRenderer, TrackerConfigRendererError};
pub use renderer::{TrackerProjectGenerator, TrackerProjectGeneratorError};
pub use wrapper::{TrackerContext, TrackerTemplate};
