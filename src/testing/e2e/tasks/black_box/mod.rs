//! Black-box E2E testing tasks.
//!
//! This module provides reusable tasks for black-box E2E testing,
//! where tests execute CLI commands as external processes.
//!
//! ## Module Structure
//!
//! - `verify_dependencies` - Verify required system dependencies are installed

pub mod verify_dependencies;

// Re-export commonly used items
pub use verify_dependencies::verify_required_dependencies;
