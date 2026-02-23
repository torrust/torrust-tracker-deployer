//! Test Result Data Transfer Object
//!
//! This module contains the presentation DTOs for test command results.
//! They serve as the data structures passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - These DTOs are the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify these DTOs or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structures
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use serde::Serialize;

use crate::application::command_handlers::test::result::TestResult;

/// Test result data for rendering
///
/// This struct holds all the data needed to render test command
/// results for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples application layer types from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestResultData {
    /// Name of the tested environment
    pub environment_name: String,
    /// IP address of the tested instance
    pub instance_ip: String,
    /// Overall test result (always "pass" — failures are errors, not results)
    pub result: String,
    /// Advisory DNS warnings (may be empty)
    pub dns_warnings: Vec<DnsWarningData>,
}

/// DNS warning data for rendering
///
/// Represents a single advisory DNS resolution warning.
/// DNS warnings do not affect the overall test result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DnsWarningData {
    /// The domain that was checked
    pub domain: String,
    /// The expected IP address (instance IP)
    pub expected_ip: String,
    /// Human-readable description of the DNS issue
    pub issue: String,
}

impl TestResultData {
    /// Create a new `TestResultData` from test results
    ///
    /// Converts the application layer `TestResult` and environment metadata
    /// into a presentation-ready DTO.
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the tested environment
    /// * `test_result` - The application layer test result containing instance IP and DNS warnings
    #[must_use]
    pub fn new(environment_name: &str, test_result: &TestResult) -> Self {
        Self {
            environment_name: environment_name.to_string(),
            instance_ip: test_result.instance_ip.to_string(),
            result: "pass".to_string(),
            dns_warnings: test_result
                .dns_warnings
                .iter()
                .map(|w| DnsWarningData {
                    domain: w.domain.to_string(),
                    expected_ip: w.expected_ip.to_string(),
                    issue: w.to_string(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;
    use crate::application::command_handlers::test::result::{DnsIssue, DnsWarning, TestResult};
    use crate::shared::domain_name::DomainName;

    // Test fixtures and helpers

    fn test_ip() -> IpAddr {
        "10.140.190.39".parse().unwrap()
    }

    fn create_test_result_no_warnings() -> TestResult {
        TestResult::success(test_ip())
    }

    fn create_test_result_with_warnings() -> TestResult {
        let warnings = vec![
            DnsWarning {
                domain: DomainName::new("tracker.local").unwrap(),
                expected_ip: test_ip(),
                issue: DnsIssue::ResolutionFailed("name resolution failed".to_string()),
            },
            DnsWarning {
                domain: DomainName::new("api.tracker.local").unwrap(),
                expected_ip: test_ip(),
                issue: DnsIssue::IpMismatch {
                    resolved_ips: vec!["192.168.1.1".parse().unwrap()],
                },
            },
        ];
        TestResult::with_dns_warnings(test_ip(), warnings)
    }

    // Tests

    #[test]
    fn it_should_create_dto_with_no_warnings() {
        // Arrange
        let test_result = create_test_result_no_warnings();

        // Act
        let dto = TestResultData::new("my-env", &test_result);

        // Assert
        assert_eq!(dto.environment_name, "my-env");
        assert_eq!(dto.instance_ip, "10.140.190.39");
        assert_eq!(dto.result, "pass");
        assert!(dto.dns_warnings.is_empty());
    }

    #[test]
    fn it_should_create_dto_with_dns_warnings() {
        // Arrange
        let test_result = create_test_result_with_warnings();

        // Act
        let dto = TestResultData::new("test-env", &test_result);

        // Assert
        assert_eq!(dto.dns_warnings.len(), 2);
        assert_eq!(dto.dns_warnings[0].domain, "tracker.local");
        assert_eq!(dto.dns_warnings[0].expected_ip, "10.140.190.39");
        assert!(dto.dns_warnings[0].issue.contains("does not resolve"));
        assert_eq!(dto.dns_warnings[1].domain, "api.tracker.local");
        assert!(dto.dns_warnings[1].issue.contains("192.168.1.1"));
    }

    #[test]
    fn it_should_always_have_pass_result() {
        // Arrange
        let test_result = create_test_result_with_warnings();

        // Act
        let dto = TestResultData::new("my-env", &test_result);

        // Assert - result is always "pass" because failures are errors, not results
        assert_eq!(dto.result, "pass");
    }

    #[test]
    fn it_should_convert_instance_ip_to_string() {
        // Arrange
        let test_result = create_test_result_no_warnings();

        // Act
        let dto = TestResultData::new("my-env", &test_result);

        // Assert
        assert_eq!(dto.instance_ip, "10.140.190.39");
    }
}
