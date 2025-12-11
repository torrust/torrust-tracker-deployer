//! Template wrapper for templates/tracker/tracker.toml.tera
//!
//! In Phase 4, this template has no variables - all values are hardcoded.
//! Phase 6 will add dynamic configuration.

pub mod context;
pub mod template;

pub use context::TrackerContext;
pub use template::TrackerTemplate;
