//! Basic Environment Information View
//!
//! This module provides a view for rendering basic environment information
//! (name, state, provider, creation date).

use chrono::{DateTime, Utc};

/// View for rendering basic environment information
///
/// This view handles the display of fundamental environment properties
/// that are always available regardless of state.
pub struct BasicInfoView;

impl BasicInfoView {
    /// Render basic environment information as formatted lines
    ///
    /// # Arguments
    ///
    /// * `name` - Environment name
    /// * `state` - Current state display name
    /// * `provider` - Provider display name
    /// * `created_at` - Creation timestamp
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(
        name: &str,
        state: &str,
        provider: &str,
        created_at: DateTime<Utc>,
    ) -> Vec<String> {
        vec![
            String::new(), // blank line
            format!("Environment: {name}"),
            format!("State: {state}"),
            format!("Provider: {provider}"),
            format!("Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 7, 12, 30, 45).unwrap()
    }

    #[test]
    fn it_should_render_environment_name() {
        let lines = BasicInfoView::render("my-env", "Created", "LXD", test_timestamp());
        assert!(lines.iter().any(|l| l.contains("Environment: my-env")));
    }

    #[test]
    fn it_should_render_state() {
        let lines = BasicInfoView::render("my-env", "Running", "LXD", test_timestamp());
        assert!(lines.iter().any(|l| l.contains("State: Running")));
    }

    #[test]
    fn it_should_render_provider() {
        let lines = BasicInfoView::render("my-env", "Created", "Hetzner Cloud", test_timestamp());
        assert!(lines.iter().any(|l| l.contains("Provider: Hetzner Cloud")));
    }

    #[test]
    fn it_should_render_creation_date_in_utc_format() {
        let lines = BasicInfoView::render("my-env", "Created", "LXD", test_timestamp());
        assert!(lines
            .iter()
            .any(|l| l.contains("Created: 2025-01-07 12:30:45 UTC")));
    }

    #[test]
    fn it_should_start_with_blank_line() {
        let lines = BasicInfoView::render("my-env", "Created", "LXD", test_timestamp());
        assert!(lines.first().is_some_and(String::is_empty));
    }
}
