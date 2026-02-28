//! JSON View for Provision Details
//!
//! This module provides JSON-based rendering for provision command details.
//! It follows the Strategy Pattern, providing one specific rendering strategy
//! (machine-readable JSON) for provision details.

use super::super::ProvisionDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// JSON view for rendering provision details
///
/// This view produces machine-readable JSON output suitable for programmatic
/// parsing, automation workflows, and AI agents.
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
/// use std::net::{IpAddr, Ipv4Addr};
/// use std::path::PathBuf;
/// use chrono::{TimeZone, Utc};
/// use torrust_tracker_deployer_lib::domain::provider::Provider;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::{
///     ProvisionDetailsData, JsonView
/// };
///
/// let data = ProvisionDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
///     ssh_username: "torrust".to_string(),
///     ssh_port: 22,
///     ssh_private_key_path: PathBuf::from("/path/to/key"),
///     provider: Provider::Lxd.to_string(),
///     provisioned_at: Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap(),
///     domains: vec!["tracker.example.com".to_string()],
/// };
///
/// let json = JsonView::render(&data).expect("JSON serialization failed");
/// assert!(json.contains(r#""environment_name": "my-env""#));
/// ```
pub struct JsonView;

impl Render<ProvisionDetailsData> for JsonView {
    fn render(data: &ProvisionDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::Render;
    use chrono::{TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use crate::domain::provider::Provider;

    fn test_timestamp() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap()
    }

    #[test]
    fn it_should_render_provision_details_as_json_format() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "my-env".to_string(),
            instance_name: "torrust-tracker-vm-my-env".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/testing_rsa"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec!["tracker.example.com".to_string()],
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization should succeed");

        // Then - Basic structure
        assert!(json.contains(r#""environment_name": "my-env""#));
        assert!(json.contains(r#""instance_name": "torrust-tracker-vm-my-env""#));
        assert!(json.contains(r#""ssh_username": "torrust""#));
        assert!(json.contains(r#""ssh_port": 22"#));
        assert!(json.contains(r#""provider": "lxd""#));
    }

    #[test]
    fn it_should_include_all_required_fields() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
            ssh_username: "admin".to_string(),
            ssh_port: 2222,
            ssh_private_key_path: PathBuf::from("/home/user/.ssh/deploy_key"),
            provider: Provider::Hetzner.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![
                "tracker1.example.com".to_string(),
                "tracker2.example.com".to_string(),
            ],
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization should succeed");

        // Then - All fields present
        assert!(json.contains(r#""environment_name""#));
        assert!(json.contains(r#""instance_name""#));
        assert!(json.contains(r#""instance_ip""#));
        assert!(json.contains(r#""ssh_username""#));
        assert!(json.contains(r#""ssh_port""#));
        assert!(json.contains(r#""ssh_private_key_path""#));
        assert!(json.contains(r#""provider""#));
        assert!(json.contains(r#""provisioned_at""#));
        assert!(json.contains(r#""domains""#));

        // Then - Values are correct
        assert!(json.contains(r#""192.168.1.100""#));
        assert!(json.contains(r#""hetzner""#));
        assert!(json.contains(r#""tracker1.example.com""#));
        assert!(json.contains(r#""tracker2.example.com""#));
    }

    #[test]
    fn it_should_handle_empty_domains_array() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "simple-env".to_string(),
            instance_name: "torrust-tracker-vm-simple-env".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/key"),
            provider: Provider::Hetzner.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![],
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization should succeed");

        // Then
        assert!(json.contains(r#""domains": []"#));
    }

    #[test]
    fn it_should_handle_missing_instance_ip() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "incomplete-env".to_string(),
            instance_name: "torrust-tracker-vm-incomplete-env".to_string(),
            instance_ip: None,
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/key"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![],
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization should succeed");

        // Then
        assert!(json.contains(r#""instance_ip": null"#));
    }

    #[test]
    fn it_should_produce_valid_json() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "test".to_string(),
            instance_name: "torrust-tracker-vm-test".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/key"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![],
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization should succeed");

        // Then - Should be parseable back to a value
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("JSON should be valid and parseable");

        assert_eq!(parsed["environment_name"], "test");
        assert_eq!(parsed["instance_name"], "torrust-tracker-vm-test");
        assert_eq!(parsed["instance_ip"], "10.0.0.1");
        assert_eq!(parsed["ssh_username"], "torrust");
        assert_eq!(parsed["ssh_port"], 22);
        assert_eq!(parsed["provider"], "lxd");
    }
}
