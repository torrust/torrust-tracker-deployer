//! Application-Layer Trait Definitions (DDD Ports)
//!
//! This module defines the traits that the application layer depends on.
//! These traits are defined here (where they're consumed) and implemented
//! in outer layers (Presentation, Infrastructure), following the Dependency
//! Inversion Principle.
//!
//! The term "traits" avoids confusion with network ports in this infrastructure project.
//!
//! ## Components
//!
//! - `progress` - Progress reporting trait for command workflows

pub mod progress;
pub mod repository_provider;

// Re-export main types for convenience
pub use progress::{CommandProgressListener, NullProgressListener};
pub use repository_provider::RepositoryProvider;
