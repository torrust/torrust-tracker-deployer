//! JSON View for Environment List
//!
//! This module provides JSON-based rendering for the environment list command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`EnvironmentList` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes environment list information to JSON using `serde_json`.
//! The output includes environment summaries, failed environments, and metadata.

use crate::presentation::cli::views::commands::list::view_data::EnvironmentList;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering environment list as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the environment list without any transformations,
/// preserving all field names and structure from the domain DTOs.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::list::info::{
///     EnvironmentList, EnvironmentSummary,
/// };
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::list::JsonView;
///
/// let summaries = vec![
///     EnvironmentSummary::new(
///         "production-tracker".to_string(),
///         "Running".to_string(),
///         "LXD".to_string(),
///         "2026-02-14T16:45:00Z".to_string(),
///     ),
/// ];
///
/// let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());
/// let output = JsonView::render(&list);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["total_count"], 1);
/// ```
pub struct JsonView;

impl JsonView {
    /// Render environment list as JSON
    ///
    /// Serializes the environment list to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environments`: Array of environment summaries
    /// - `total_count`: Number of successfully loaded environments
    /// - `failed_environments`: Array of failures (name, error pairs)
    /// - `data_directory`: Path to scanned directory
    ///
    /// # Arguments
    ///
    /// * `list` - Environment list to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized environment list.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::list::info::{
    ///     EnvironmentList, EnvironmentSummary,
    /// };
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::list::JsonView;
    ///
    /// let summaries = vec![
    ///     EnvironmentSummary::new(
    ///         "env1".to_string(),
    ///         "Running".to_string(),
    ///         "LXD".to_string(),
    ///         "2026-01-05T10:30:00Z".to_string(),
    ///     ),
    ///     EnvironmentSummary::new(
    ///         "env2".to_string(),
    ///         "Created".to_string(),
    ///         "Hetzner".to_string(),
    ///         "2026-01-06T14:15:30Z".to_string(),
    ///     ),
    /// ];
    ///
    /// let list = EnvironmentList::new(summaries, vec![], "/data".to_string());
    /// let json = JsonView::render(&list);
    ///
    /// assert!(json.contains("\"total_count\": 2"));
    /// assert!(json.contains("\"env1\""));
    /// assert!(json.contains("\"env2\""));
    /// ```
    #[must_use]
    pub fn render(list: &EnvironmentList) -> String {
        serde_json::to_string_pretty(list).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize environment list",
                "message": e.to_string(),
            }))
            .unwrap_or_else(|_| {
                r#"{
  "error": "Failed to serialize error message"
}"#
                .to_string()
            })
        })
    }
}

impl Render<EnvironmentList> for JsonView {
    fn render(data: &EnvironmentList) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::presentation::cli::views::commands::list::view_data::EnvironmentSummary;

    #[test]
    fn it_should_render_empty_environment_list_as_json() {
        let list = EnvironmentList::new(vec![], vec![], "/path/to/data".to_string());

        let output = JsonView::render(&list);

        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert_eq!(parsed["total_count"], 0);
        assert_eq!(parsed["environments"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["failed_environments"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["data_directory"], "/path/to/data");
    }

    #[test]
    fn it_should_render_single_environment_as_json() {
        let summaries = vec![EnvironmentSummary::new(
            "my-production".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-02-14T16:45:00Z".to_string(),
        )];

        let list = EnvironmentList::new(summaries, vec![], "/data".to_string());

        let output = JsonView::render(&list);

        let parsed: Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert_eq!(parsed["total_count"], 1);

        let envs = parsed["environments"].as_array().unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0]["name"], "my-production");
        assert_eq!(envs[0]["state"], "Running");
        assert_eq!(envs[0]["provider"], "LXD");
        assert_eq!(envs[0]["created_at"], "2026-02-14T16:45:00Z");
    }

    #[test]
    fn it_should_render_multiple_environments_as_json() {
        let summaries = vec![
            EnvironmentSummary::new(
                "env1".to_string(),
                "Running".to_string(),
                "LXD".to_string(),
                "2026-01-05T10:30:00Z".to_string(),
            ),
            EnvironmentSummary::new(
                "env2".to_string(),
                "Created".to_string(),
                "Hetzner Cloud".to_string(),
                "2026-01-06T14:15:30Z".to_string(),
            ),
            EnvironmentSummary::new(
                "staging-high-availability-tracker".to_string(),
                "Provisioned".to_string(),
                "LXD".to_string(),
                "2026-01-10T09:00:00Z".to_string(),
            ),
        ];

        let list = EnvironmentList::new(summaries, vec![], "/workspace/data".to_string());

        let output = JsonView::render(&list);

        let parsed: Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert_eq!(parsed["total_count"], 3);

        let envs = parsed["environments"].as_array().unwrap();
        assert_eq!(envs.len(), 3);

        // Verify full names are preserved (no truncation)
        assert_eq!(envs[2]["name"], "staging-high-availability-tracker");
    }

    #[test]
    fn it_should_include_failed_environments_in_json() {
        let summaries = vec![EnvironmentSummary::new(
            "working-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let failures = vec![
            (
                "broken-env-1".to_string(),
                "Invalid JSON format".to_string(),
            ),
            (
                "broken-env-2".to_string(),
                "Missing required field".to_string(),
            ),
        ];

        let list = EnvironmentList::new(summaries, failures, "/data".to_string());

        let output = JsonView::render(&list);

        let parsed: Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert_eq!(parsed["total_count"], 1);

        let failed = parsed["failed_environments"].as_array().unwrap();
        assert_eq!(failed.len(), 2);

        // Failed environments are represented as [name, error] tuples
        assert_eq!(failed[0][0], "broken-env-1");
        assert_eq!(failed[0][1], "Invalid JSON format");
        assert_eq!(failed[1][0], "broken-env-2");
        assert_eq!(failed[1][1], "Missing required field");
    }

    #[test]
    fn it_should_produce_pretty_printed_json() {
        let summaries = vec![EnvironmentSummary::new(
            "test".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let list = EnvironmentList::new(summaries, vec![], "/data".to_string());

        let output = JsonView::render(&list);

        // Pretty-printed JSON should have newlines and indentation
        assert!(output.contains('\n'));
        assert!(output.contains("  "));
    }
}
