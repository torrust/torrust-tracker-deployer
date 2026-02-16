//! Views for List Command
//!
//! This module contains view components for rendering list command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `TextView`: Renders human-readable text table output
//!
//! # Structure
//!
//! - `views/`: View rendering implementations
//!   - `text_view.rs`: Human-readable table rendering
//!
//! # Future Expansion
//!
//! When JSON output support is added (EPIC #348 task 12.5), create `views/json_view.rs`.

pub mod views {
    pub mod text_view;

    // Re-export main types for convenience
    pub use text_view::TextView;
}

// Re-export everything at the module level for backward compatibility
pub use views::TextView;
