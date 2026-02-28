//! Text View for Environment List
//!
//! This module provides text-based rendering for the environment list command.
//! It follows the Strategy Pattern, providing one specific rendering strategy
//! (human-readable text table) for environment lists.

use crate::presentation::cli::views::commands::list::view_data::EnvironmentList;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// Text view for rendering environment list
///
/// This view is responsible for formatting and rendering the list of
/// environments that users see when running the `list` command.
///
/// # Design
///
/// This view is part of a Strategy Pattern implementation where:
/// - Each format (Text, JSON, XML, etc.) has its own dedicated view
/// - Adding new formats requires creating new view files, not modifying existing ones
/// - Follows Open/Closed Principle from SOLID
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::application::command_handlers::list::info::{
///     EnvironmentList, EnvironmentSummary,
/// };
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::list::TextView;
///
/// let summaries = vec![
///     EnvironmentSummary::new(
///         "my-production".to_string(),
///         "Running".to_string(),
///         "Hetzner Cloud".to_string(),
///         "2026-01-05T10:30:00Z".to_string(),
///     ),
/// ];
///
/// let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());
/// let output = TextView::render(&list).unwrap();
/// assert!(output.contains("my-production"));
/// assert!(output.contains("Running"));
/// ```
pub struct TextView;

impl TextView {
    /// Render empty workspace message
    fn render_empty(list: &EnvironmentList) -> String {
        let mut lines = Vec::new();

        lines.push(String::new());
        lines.push(format!("No environments found in: {}", list.data_directory));
        lines.push(String::new());
        lines.push("To create a new environment:".to_string());
        lines.push(
            "  torrust-tracker-deployer create environment --env-file <config.json>".to_string(),
        );
        lines.push(String::new());
        lines.push("For more information, see docs/user-guide/commands.md".to_string());

        lines.join("\n")
    }

    /// Render table header row
    fn render_table_header() -> String {
        format!(
            "{:<50} {:<18} {:<14} {}",
            "Name", "State", "Provider", "Created"
        )
    }

    /// Render table separator
    fn render_table_separator() -> String {
        "─".repeat(106)
    }

    /// Render a single table row
    fn render_table_row(
        env: &crate::application::command_handlers::list::info::EnvironmentSummary,
    ) -> String {
        format!(
            "{:<50} {:<18} {:<14} {}",
            Self::truncate(&env.name, 50),
            Self::truncate(&env.state, 18),
            Self::truncate(&env.provider, 14),
            &env.created_at
        )
    }

    /// Truncate a string to fit column width
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len > 3 {
            format!("{}...", &s[..max_len - 3])
        } else {
            s[..max_len].to_string()
        }
    }
}

impl Render<EnvironmentList> for TextView {
    fn render(list: &EnvironmentList) -> Result<String, ViewRenderError> {
        let mut lines = Vec::new();

        if list.is_empty() {
            return Ok(Self::render_empty(list));
        }

        // Header with count
        lines.push(String::new());
        lines.push(format!("Environments ({} found):", list.total_count));
        lines.push(String::new());

        // Table header
        lines.push(Self::render_table_header());
        lines.push(Self::render_table_separator());

        // Table rows
        for env in &list.environments {
            lines.push(Self::render_table_row(env));
        }

        // Partial failure warnings
        if list.has_failures() {
            lines.push(String::new());
            lines.push("Warning: Failed to load the following environments:".to_string());
            for (name, error) in &list.failed_environments {
                lines.push(format!("  - {name}: {error}"));
            }
            lines.push(String::new());
            lines.push("For troubleshooting, see docs/user-guide/commands.md".to_string());
        }

        // Hint about purge command
        lines.push(String::new());
        lines.push(
            "Hint: Use 'purge' command to completely remove destroyed environments.".to_string(),
        );

        Ok(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::commands::list::view_data::EnvironmentSummary;

    #[test]
    fn it_should_render_empty_workspace() {
        let list = EnvironmentList::new(vec![], vec![], "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        assert!(output.contains("No environments found in: /path/to/data"));
        assert!(output.contains("create environment --env-file"));
    }

    #[test]
    fn it_should_render_environment_list_with_header() {
        let summaries = vec![EnvironmentSummary::new(
            "test-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        assert!(output.contains("Environments (1 found):"));
        assert!(output.contains("Name"));
        assert!(output.contains("State"));
        assert!(output.contains("Provider"));
        assert!(output.contains("Created"));
        assert!(output
            .contains("Hint: Use 'purge' command to completely remove destroyed environments."));
    }

    #[test]
    fn it_should_render_environment_rows() {
        let summaries = vec![
            EnvironmentSummary::new(
                "production".to_string(),
                "Running".to_string(),
                "Hetzner Cloud".to_string(),
                "2026-01-05T10:30:00Z".to_string(),
            ),
            EnvironmentSummary::new(
                "staging".to_string(),
                "Provisioned".to_string(),
                "LXD".to_string(),
                "2026-01-06T14:15:30Z".to_string(),
            ),
        ];

        let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        assert!(output.contains("production"));
        assert!(output.contains("Running"));
        assert!(output.contains("Hetzner Cloud"));
        assert!(output.contains("staging"));
        assert!(output.contains("Provisioned"));
        assert!(output.contains("LXD"));
        assert!(output
            .contains("Hint: Use 'purge' command to completely remove destroyed environments."));
    }

    #[test]
    fn it_should_render_partial_failure_warnings() {
        let summaries = vec![EnvironmentSummary::new(
            "good-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let failures = vec![
            ("broken-env".to_string(), "Invalid JSON".to_string()),
            ("old-env".to_string(), "Permission denied".to_string()),
        ];

        let list = EnvironmentList::new(summaries, failures, "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        assert!(output.contains("Warning: Failed to load the following environments:"));
        assert!(output.contains("broken-env: Invalid JSON"));
        assert!(output.contains("old-env: Permission denied"));
        assert!(output
            .contains("Hint: Use 'purge' command to completely remove destroyed environments."));
    }

    #[test]
    fn it_should_truncate_long_names() {
        let summaries = vec![EnvironmentSummary::new(
            "very-long-environment-name-that-exceeds-column-width".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        // Should truncate the long name at 50 characters
        assert!(output.contains("very-long-environment-name-that-exceeds-column-..."));
        assert!(output
            .contains("Hint: Use 'purge' command to completely remove destroyed environments."));
    }

    #[test]
    fn it_should_handle_multiple_environments() {
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
                "env3".to_string(),
                "Destroyed".to_string(),
                "LXD".to_string(),
                "2026-01-07T09:00:12Z".to_string(),
            ),
        ];

        let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());

        let output = TextView::render(&list).unwrap();

        assert!(output.contains("Environments (3 found):"));
        assert!(output.contains("env1"));
        assert!(output.contains("env2"));
        assert!(output.contains("env3"));
        assert!(output
            .contains("Hint: Use 'purge' command to completely remove destroyed environments."));
    }
}
