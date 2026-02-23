//! Text View for Test Command
//!
//! This module provides text-based rendering for the test command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`TestResultData` DTO).
//!
//! # Design
//!
//! The `TextView` formats test results as human-readable text suitable
//! for terminal display and direct user consumption. DNS warnings are
//! rendered as indented bullet items when present.

use std::fmt::Write;

use crate::presentation::views::commands::test::TestResultData;

/// View for rendering test results as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption. It presents test results in a clear, readable format
/// with any DNS warnings listed as bullet items.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::commands::test::{
///     TestResultData, TextView,
/// };
///
/// let data = TestResultData {
///     environment_name: "my-env".to_string(),
///     instance_ip: "10.140.190.39".to_string(),
///     result: "pass".to_string(),
///     dns_warnings: vec![],
/// };
///
/// let output = TextView::render(&data);
/// assert!(output.contains("Test Results:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct TextView;

impl TextView {
    /// Render test results as human-readable formatted text
    ///
    /// Takes test results and produces human-readable output suitable for
    /// displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - Test result data to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Test Results section with environment name, instance IP, and result
    /// - DNS Warnings section (only if warnings are present)
    ///
    /// # Format
    ///
    /// The output follows this structure:
    /// ```text
    /// Test Results:
    ///   Environment:       <environment_name>
    ///   Instance IP:       <instance_ip>
    ///   Result:            <result>
    ///
    /// DNS Warnings:         (only if warnings exist)
    ///   - <domain>: <issue>
    /// ```
    #[must_use]
    pub fn render(data: &TestResultData) -> String {
        let mut output = format!(
            r"Test Results:
  Environment:       {}
  Instance IP:       {}
  Result:            {}",
            data.environment_name, data.instance_ip, data.result,
        );

        if !data.dns_warnings.is_empty() {
            output.push_str("\n\nDNS Warnings:");
            for warning in &data.dns_warnings {
                let _ = write!(output, "\n  - {}: {}", warning.domain, warning.issue);
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::views::commands::test::DnsWarningData;

    // Test fixtures and helpers

    fn create_test_data_no_warnings() -> TestResultData {
        TestResultData {
            environment_name: "test-env".to_string(),
            instance_ip: "10.140.190.39".to_string(),
            result: "pass".to_string(),
            dns_warnings: vec![],
        }
    }

    fn create_test_data_with_warnings() -> TestResultData {
        TestResultData {
            environment_name: "test-env".to_string(),
            instance_ip: "10.140.190.39".to_string(),
            result: "pass".to_string(),
            dns_warnings: vec![
                DnsWarningData {
                    domain: "tracker.local".to_string(),
                    expected_ip: "10.140.190.39".to_string(),
                    issue: "tracker.local does not resolve (expected: 10.140.190.39): name resolution failed".to_string(),
                },
                DnsWarningData {
                    domain: "api.tracker.local".to_string(),
                    expected_ip: "10.140.190.39".to_string(),
                    issue: "api.tracker.local resolves to [192.168.1.1] but expected 10.140.190.39".to_string(),
                },
            ],
        }
    }

    /// Helper to assert text contains all expected substrings
    fn assert_contains_all(text: &str, expected: &[&str]) {
        for substring in expected {
            assert!(
                text.contains(substring),
                "Expected text to contain '{substring}' but it didn't.\nActual text:\n{text}"
            );
        }
    }

    // Tests

    #[test]
    fn it_should_render_test_results_as_formatted_text() {
        // Arrange
        let data = create_test_data_no_warnings();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_contains_all(
            &text,
            &[
                "Test Results:",
                "Environment:",
                "test-env",
                "Instance IP:",
                "10.140.190.39",
                "Result:",
                "pass",
            ],
        );
    }

    #[test]
    fn it_should_not_include_dns_warnings_section_when_no_warnings() {
        // Arrange
        let data = create_test_data_no_warnings();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert!(!text.contains("DNS Warnings:"));
    }

    #[test]
    fn it_should_include_dns_warnings_section_when_warnings_present() {
        // Arrange
        let data = create_test_data_with_warnings();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_contains_all(
            &text,
            &[
                "DNS Warnings:",
                "tracker.local",
                "does not resolve",
                "api.tracker.local",
                "192.168.1.1",
            ],
        );
    }

    #[test]
    fn it_should_render_each_warning_as_a_bullet_item() {
        // Arrange
        let data = create_test_data_with_warnings();

        // Act
        let text = TextView::render(&data);

        // Assert - each warning should be on its own bullet line
        let warning_lines: Vec<&str> = text
            .lines()
            .filter(|line| line.trim_start().starts_with("- "))
            .collect();
        assert_eq!(warning_lines.len(), 2);
    }
}
