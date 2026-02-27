//! JSON View for Run Command Output
//!
//! This module provides JSON-based rendering for run command output.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`RunDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes the `RunDetailsData` DTO to JSON using `serde_json`.
//! The output includes the environment name, state (always "Running"), and
//! service information from the existing DTOs.

use crate::presentation::cli::views::commands::run::view_data::RunDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering run command output as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes service and Grafana information without
/// any transformations.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::show::info::ServiceInfo;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::run::{JsonView, RunDetailsData};
///
/// let services = ServiceInfo::new(
///     vec!["udp://10.0.0.1:6969/announce".to_string()],
///     vec![],
///     vec!["http://10.0.0.1:7070/announce".to_string()],
///     vec![],
///     "http://10.0.0.1:1212/api".to_string(),
///     false,
///     false,
///     "http://10.0.0.1:1313/health_check".to_string(),
///     false,
///     false,
///     vec![],
/// );
///
/// let data = RunDetailsData::new("my-env".to_string(), services, None);
/// let output = JsonView::render(&data);
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["state"], "Running");
/// ```
pub struct JsonView;

impl JsonView {
    /// Render run command output as JSON
    ///
    /// Serializes the `RunDetailsData` DTO to pretty-printed JSON format.
    ///
    /// # Arguments
    ///
    /// * `data` - Run details DTO containing environment name, state,
    ///   service endpoints, and optional Grafana information
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized run command output.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::show::info::{
    ///     ServiceInfo, GrafanaInfo,
    /// };
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::run::{JsonView, RunDetailsData};
    /// use url::Url;
    ///
    /// let services = ServiceInfo::new(
    ///     vec!["udp://10.0.0.1:6969/announce".to_string()],
    ///     vec![],
    ///     vec!["http://10.0.0.1:7070/announce".to_string()],
    ///     vec![],
    ///     "http://10.0.0.1:1212/api".to_string(),
    ///     false,
    ///     false,
    ///     "http://10.0.0.1:1313/health_check".to_string(),
    ///     false,
    ///     false,
    ///     vec![],
    /// );
    ///
    /// let grafana = GrafanaInfo::new(
    ///     Url::parse("http://10.0.0.1:3000").unwrap(),
    ///     false,
    /// );
    ///
    /// let data = RunDetailsData::new("my-env".to_string(), services, Some(grafana));
    /// let json = JsonView::render(&data);
    /// // Verify it's valid JSON and has expected fields
    /// let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    /// assert_eq!(parsed["environment_name"], "my-env");
    /// assert!(parsed["services"].is_object());
    /// assert!(parsed["grafana"].is_object());
    /// ```
    #[must_use]
    pub fn render(data: &RunDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize run details",
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

impl Render<RunDetailsData> for JsonView {
    fn render(data: &RunDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};

    use super::*;

    fn sample_data() -> RunDetailsData {
        let services = ServiceInfo::new(
            vec!["udp://udp.tracker.local:6969/announce".to_string()],
            vec![],
            vec!["http://10.140.190.133:7070/announce".to_string()],
            vec![],
            "http://10.140.190.133:1212/api".to_string(),
            false,
            false,
            "http://10.140.190.133:1313/health_check".to_string(),
            false,
            false,
            vec![],
        );
        RunDetailsData::new("test-env".to_string(), services, None)
    }

    #[test]
    fn it_should_render_basic_json_output() {
        let data = sample_data();
        let output = JsonView::render(&data);

        assert!(
            output.contains(r#""environment_name":"test-env""#)
                || output.contains(r#""environment_name": "test-env""#)
        );
        assert!(
            output.contains(r#""state":"Running""#) || output.contains(r#""state": "Running""#)
        );
        assert!(output.contains(r#""services":"#));
        assert!(output.contains(r#""grafana":null"#) || output.contains(r#""grafana": null"#));
    }

    #[test]
    fn it_should_include_grafana_when_provided() {
        let services = ServiceInfo::new(
            vec!["udp://udp.tracker.local:6969/announce".to_string()],
            vec![],
            vec!["http://10.140.190.133:7070/announce".to_string()],
            vec![],
            "http://10.140.190.133:1212/api".to_string(),
            false,
            false,
            "http://10.140.190.133:1313/health_check".to_string(),
            false,
            false,
            vec![],
        );
        let grafana = GrafanaInfo::new(Url::parse("http://10.140.190.133:3000").unwrap(), false);
        let data = RunDetailsData::new("test-env".to_string(), services, Some(grafana));

        let output = JsonView::render(&data);

        assert!(output.contains(r#""grafana":"#));
        assert!(
            output.contains(r#""url":"http://10.140.190.133:3000/""#)
                || output.contains(r#""url": "http://10.140.190.133:3000/""#)
        );
        assert!(
            output.contains(r#""uses_https":false"#) || output.contains(r#""uses_https": false"#)
        );
    }

    #[test]
    fn it_should_produce_valid_json() {
        let data = sample_data();
        let output = JsonView::render(&data);

        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("Output should be valid JSON");

        assert_eq!(parsed["environment_name"], "test-env");
        assert_eq!(parsed["state"], "Running");
        assert!(parsed["services"].is_object());
        assert!(parsed["grafana"].is_null());
    }

    #[test]
    fn it_should_include_all_service_fields() {
        let data = sample_data();
        let output = JsonView::render(&data);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let services_obj = &parsed["services"];

        assert!(services_obj["udp_trackers"].is_array());
        assert!(services_obj["https_http_trackers"].is_array());
        assert!(services_obj["direct_http_trackers"].is_array());
        assert!(services_obj["localhost_http_trackers"].is_array());
        assert!(services_obj["api_endpoint"].is_string());
        assert!(services_obj["api_uses_https"].is_boolean());
        assert!(services_obj["api_is_localhost_only"].is_boolean());
        assert!(services_obj["health_check_url"].is_string());
        assert!(services_obj["health_check_uses_https"].is_boolean());
        assert!(services_obj["health_check_is_localhost_only"].is_boolean());
        assert!(services_obj["tls_domains"].is_array());
    }
}
