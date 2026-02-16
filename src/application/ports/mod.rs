//! Application Ports (DDD)
//!
//! This module defines the interfaces (ports) that the application layer
//! depends on. These traits are defined here (where they're consumed) and
//! implemented in outer layers (Presentation, Infrastructure), following
//! the Dependency Inversion Principle.
//!
//! ## Components
//!
//! - `progress` - Progress reporting interface for command workflows

pub mod progress;

// Re-export main types for convenience
pub use progress::{CommandProgressListener, NullProgressListener};
