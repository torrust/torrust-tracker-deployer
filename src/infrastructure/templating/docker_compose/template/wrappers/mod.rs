//! Docker Compose template wrappers
//!
//! Contains wrappers for templates that need variable substitution (.tera extension).
pub mod env;

// Re-export the main template structs for easier access
pub use env::EnvTemplate;
