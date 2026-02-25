//! JSON View for Run Command Output
//!
//! This module provides JSON-based rendering for run command output.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`ServiceInfo` and `GrafanaInfo` DTOs).
//!
//! # Design
//!
//! The `JsonView` serializes service information to JSON using `serde_json`.
//! The output includes the environment name, state (always "Running"), and
//! service information from the existing DTOs.

use serde::Serialize;

use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};

/// DTO for JSON output of run command
///
/// This structure wraps the service information for JSON serialization.
#[derive(Debug, Serialize)]
struct RunCommandOutput<'a> {
    environment_name: &'a str,
    state: &'a str,
    services: &'a ServiceInfo,
    grafana: Option<&'a GrafanaInfo>,
}

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
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::run::JsonView;
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
/// let output = JsonView::render("my-env", &services, None);
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["state"], "Running");
/// ```
pub struct JsonView;

impl JsonView {
    /// Render run command output as JSON
    ///
    /// Serializes the service information to pretty-printed JSON format.
    /// The state is always "Running" since this command only executes
    /// when services are being started.
    ///
    /// # Arguments
    ///
    /// * `env_name` - Name of the environment
    /// * `services` - Service information containing tracker endpoints
    /// * `grafana` - Optional Grafana service information
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::run::JsonView;
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
    /// let json = JsonView::render("my-env", &services, Some(&grafana));
    /// // Verify it's valid JSON and has expected fields
    /// let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    /// assert_eq!(parsed["environment_name"], "my-env");
    /// assert!(parsed["services"].is_object());
    /// assert!(parsed["grafana"].is_object());
    /// ```
    #[must_use]
    pub fn render(env_name: &str, services: &ServiceInfo, grafana: Option<&GrafanaInfo>) -> String {
        let output = RunCommandOutput {
            environment_name: env_name,
            state: "Running",
            services,
            grafana,
        };

        serde_json::to_string_pretty(&output)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize: {e}"}}"#))
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    fn sample_basic_services() -> ServiceInfo {
        ServiceInfo::new(
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
        )
    }

    #[test]
    fn it_should_render_basic_json_output() {
        let services = sample_basic_services();
        let output = JsonView::render("test-env", &services, None);

        // Verify JSON structure
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
        let services = sample_basic_services();
        let grafana = GrafanaInfo::new(Url::parse("http://10.140.190.133:3000").unwrap(), false);

        let output = JsonView::render("test-env", &services, Some(&grafana));

        // Verify grafana section exists
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
        let services = sample_basic_services();
        let output = JsonView::render("test-env", &services, None);

        // Verify it's valid JSON by parsing it
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("Output should be valid JSON");

        // Verify key fields
        assert_eq!(parsed["environment_name"], "test-env");
        assert_eq!(parsed["state"], "Running");
        assert!(parsed["services"].is_object());
        assert!(parsed["grafana"].is_null());
    }

    #[test]
    fn it_should_include_all_service_fields() {
        let services = sample_basic_services();
        let output = JsonView::render("test-env", &services, None);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let services_obj = &parsed["services"];

        // Verify all service fields are present
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
