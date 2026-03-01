//! JSON View for Exists Command
//!
//! This module provides JSON-based rendering for the exists command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`ExistsResult` DTO).
//!
//! # Design
//!
//! The `JsonView` renders the existence check result as a bare boolean value.
//! Both `true` and `false` are valid JSON literals, so no serialization wrapper
//! is needed. This keeps the output scriptable and simple.

use crate::presentation::cli::views::commands::exists::view_data::ExistsResult;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering exists result as JSON
///
/// Outputs a bare `true` or `false` — valid JSON boolean literals.
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::exists::JsonView;
/// use torrust_tracker_deployer_lib::application::command_handlers::exists::handler::ExistsResult;
///
/// let result = ExistsResult { name: "my-env".to_string(), exists: true };
/// let output = JsonView::render(&result).unwrap();
/// assert_eq!(output, "true");
///
/// let result = ExistsResult { name: "my-env".to_string(), exists: false };
/// let output = JsonView::render(&result).unwrap();
/// assert_eq!(output, "false");
/// ```
pub struct JsonView;

impl Render<ExistsResult> for JsonView {
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
        assert_eq!(JsonView::render(&result).unwrap(), "true");
    }

    #[test]
    fn it_should_render_false_when_environment_does_not_exist() {
        let result = ExistsResult {
            name: "my-env".to_string(),
            exists: false,
        };
        assert_eq!(JsonView::render(&result).unwrap(), "false");
    }
}
