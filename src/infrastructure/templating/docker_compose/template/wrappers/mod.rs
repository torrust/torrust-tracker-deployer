//! Docker Compose template wrappers
//!
//! Contains wrappers for templates that need variable substitution (.tera extension).
pub mod docker_compose;
pub mod env;

// Re-export the main template structs for easier access
pub use docker_compose::DockerComposeTemplate;
pub use env::EnvTemplate;
