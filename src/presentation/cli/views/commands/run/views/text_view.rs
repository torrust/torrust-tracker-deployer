//! Text View for Run Command Output
//!
//! This module provides human-readable text rendering for the run command.
//! It displays service URLs, DNS hints, and helpful tips after services are started.

use crate::presentation::cli::views::commands::run::view_data::RunDetailsData;
use crate::presentation::cli::views::commands::shared::service_urls::{
    CompactServiceUrlsView, DnsHintView,
};
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering run command output as human-readable text
///
/// This view displays service URLs and configuration hints after services
/// have been started. It uses shared view components for consistency across
/// commands.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::show::info::ServiceInfo;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::run::{TextView, RunDetailsData};
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
/// let output = TextView::render(&data);
/// assert!(output.contains("Services are now accessible:"));
/// assert!(output.contains("Tip:"));
/// ```
pub struct TextView;

impl TextView {
    /// Render run command output as human-readable text
    ///
    /// # Arguments
    ///
    /// * `data` - Run details DTO containing environment name, service endpoints,
    ///   and optional Grafana information
    ///
    /// # Returns
    ///
    /// A formatted string with service URLs, DNS hints (if applicable),
    /// and a tip about using the show command for more details.
    #[must_use]
    pub fn render(data: &RunDetailsData) -> String {
        let mut output = Vec::new();

        // Render service URLs (only public services)
        let service_urls_output =
            CompactServiceUrlsView::render(&data.services, data.grafana.as_ref());
        if !service_urls_output.is_empty() {
            output.push(format!("\n{service_urls_output}"));
        }

        // Show DNS hint if HTTPS services are configured
        if let Some(dns_hint) = DnsHintView::render(&data.services) {
            output.push(format!("\n{dns_hint}"));
        }

        // Show tip about show command
        output.push(format!(
            "\nTip: Run 'torrust-tracker-deployer show {}' for full details\n",
            data.environment_name
        ));

        output.join("")
    }
}

impl Render<RunDetailsData> for TextView {
    fn render(data: &RunDetailsData) -> Result<String, ViewRenderError> {
        Ok(TextView::render(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::show::info::ServiceInfo;

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
    fn it_should_render_basic_output() {
        let data = sample_data();
        let output = TextView::render(&data);

        assert!(output.contains("Services are now accessible:"));
        assert!(output.contains("udp://udp.tracker.local:6969/announce"));
        assert!(output.contains("http://10.140.190.133:7070/announce"));
        assert!(output.contains("http://10.140.190.133:1212/api"));
        assert!(output.contains("Tip: Run 'torrust-tracker-deployer show test-env'"));
        assert!(!output.contains("DNS"));
    }

    #[test]
    fn it_should_include_grafana_when_provided() {
        use crate::application::command_handlers::show::info::GrafanaInfo;
        use url::Url;

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

        let output = TextView::render(&data);

        assert!(output.contains("Grafana:"));
        assert!(output.contains("http://10.140.190.133:3000"));
    }

    #[test]
    fn it_should_include_dns_hint_for_https_services() {
        use crate::application::command_handlers::show::info::TlsDomainInfo;

        let services = ServiceInfo::new(
            vec!["udp://udp.tracker.local:6969/announce".to_string()],
            vec!["https://http.tracker.local/announce".to_string()],
            vec![],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            false,
            "https://health.tracker.local/health_check".to_string(),
            true,
            false,
            vec![
                TlsDomainInfo::new("http.tracker.local".to_string(), 7070),
                TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
            ],
        );
        let data = RunDetailsData::new("test-env".to_string(), services, None);

        let output = TextView::render(&data);

        assert!(output.contains("Note: HTTPS services require DNS configuration"));
    }

    #[test]
    fn it_should_always_include_tip() {
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
        let data = RunDetailsData::new("my-environment".to_string(), services, None);

        let output = TextView::render(&data);

        assert!(output.contains("Tip: Run 'torrust-tracker-deployer show my-environment'"));
    }
}
