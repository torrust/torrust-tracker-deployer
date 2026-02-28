//! Text View for Exists Command
//!
//! This module provides text-based rendering for the exists command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`ExistsResult` DTO).
//!
//! # Design
//!
//! The `TextView` renders the existence check result as a bare boolean value.
//! `true` and `false` are the most natural human-readable representation of
//! a boolean existence check, and are also valid JSON, making the output
//! format-agnostic and directly scriptable.

use crate::presentation::cli::views::commands::exists::view_data::ExistsResult;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering exists result as human-readable text
///
/// Outputs a bare `true` or `false` string.
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::exists::TextView;
/// use torrust_tracker_deployer_lib::application::command_handlers::exists::handler::ExistsResult;
///
/// let result = ExistsResult { name: "my-env".to_string(), exists: true };
/// let output = TextView::render(&result).unwrap();
/// assert_eq!(output, "true");
///
/// let result = ExistsResult { name: "my-env".to_string(), exists: false };
/// let output = TextView::render(&result).unwrap();
/// assert_eq!(output, "false");
/// ```
pub struct TextView;

impl Render<ExistsResult> for TextView {
    fn render(data: &ExistsResult) -> Result<String, ViewRenderError> {
        Ok(if data.exists {
            "true".to_string()
        } else {
            "false".to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::Render;

    #[test]
    fn it_should_render_true_when_environment_exists() {
        let result = ExistsResult {
            name: "my-env".to_string(),
            exists: true,
        };
        assert_eq!(TextView::render(&result).unwrap(), "true");
    }

    #[test]
    fn it_should_render_false_when_environment_does_not_exist() {
        let result = ExistsResult {
            name: "my-env".to_string(),
            exists: false,
        };
        assert_eq!(TextView::render(&result).unwrap(), "false");
    }
}
