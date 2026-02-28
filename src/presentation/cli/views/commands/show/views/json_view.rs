//! JSON View for Environment Information (Show Command)
//!
//! This module provides JSON-based rendering for environment information.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`EnvironmentInfo` DTO).
//!
//! # Design
//!
//! The `JsonView` simply serializes the `EnvironmentInfo` DTO to JSON using `serde_json`.
//! No transformation is needed since the DTO structure is already designed for display
//! purposes and contains all necessary information in a well-structured format.

use crate::presentation::cli::views::commands::show::view_data::EnvironmentInfo;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering environment information as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the complete `EnvironmentInfo` DTO without
/// any transformations.
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::application::command_handlers::show::info::EnvironmentInfo;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::show::JsonView;
/// use chrono::{TimeZone, Utc};
///
/// let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
/// let info = EnvironmentInfo::new(
///     "my-env".to_string(),
///     "Created".to_string(),
///     "LXD".to_string(),
///     created_at,
///     "created".to_string(),
/// );
///
/// let output = JsonView::render(&info).unwrap();
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["name"], "my-env");
/// assert_eq!(parsed["state"], "Created");
/// ```
pub struct JsonView;

impl Render<EnvironmentInfo> for JsonView {
    fn render(data: &EnvironmentInfo) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::presentation::cli::views::commands::show::view_data::InfrastructureInfo;
    use crate::presentation::cli::views::Render;

    #[test]
    fn it_should_render_created_state_as_json() {
        let created_at = Utc.with_ymd_and_hms(2026, 2, 16, 10, 0, 0).unwrap();
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Created".to_string(),
            "LXD".to_string(),
            created_at,
            "created".to_string(),
        );

        let output = JsonView::render(&info).unwrap();

        // Verify JSON structure
        assert!(
            output.contains(r#""name":"test-env""#) || output.contains(r#""name": "test-env""#)
        );
        assert!(
            output.contains(r#""state":"Created""#) || output.contains(r#""state": "Created""#)
        );
        assert!(output.contains(r#""provider":"LXD""#) || output.contains(r#""provider": "LXD""#));
        assert!(
            output.contains(r#""state_name":"created""#)
                || output.contains(r#""state_name": "created""#)
        );

        // Verify optional fields are null
        assert!(
            output.contains(r#""infrastructure":null"#)
                || output.contains(r#""infrastructure": null"#)
        );
        assert!(output.contains(r#""services":null"#) || output.contains(r#""services": null"#));
    }

    #[test]
    fn it_should_render_provisioned_state_with_infrastructure() {
        let created_at = Utc.with_ymd_and_hms(2026, 2, 16, 10, 0, 0).unwrap();
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            created_at,
            "provisioned".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39)),
            22,
            "torrust".to_string(),
            "/home/user/.ssh/key".to_string(),
        ));

        let output = JsonView::render(&info).unwrap();

        // Verify infrastructure section exists
        assert!(output.contains(r#""infrastructure":"#));
        assert!(
            output.contains(r#""instance_ip":"10.140.190.39""#)
                || output.contains(r#""instance_ip": "10.140.190.39""#)
        );
        assert!(output.contains(r#""ssh_port":22"#) || output.contains(r#""ssh_port": 22"#));
        assert!(
            output.contains(r#""ssh_user":"torrust""#)
                || output.contains(r#""ssh_user": "torrust""#)
        );
    }

    #[test]
    fn it_should_render_valid_json() {
        let created_at = Utc.with_ymd_and_hms(2026, 2, 16, 10, 0, 0).unwrap();
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Created".to_string(),
            "LXD".to_string(),
            created_at,
            "created".to_string(),
        );

        let output = JsonView::render(&info).unwrap();

        // Verify it's valid JSON by parsing it
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("Output should be valid JSON");

        // Verify key fields
        assert_eq!(parsed["name"], "test-env");
        assert_eq!(parsed["state"], "Created");
        assert_eq!(parsed["provider"], "LXD");
    }
}
