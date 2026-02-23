//! Result types for the test command handler
//!
//! These DTOs encapsulate the structured output from the test command,
//! including infrastructure test results and advisory DNS warnings.
//! The presentation layer is responsible for rendering these to the user.

use std::fmt;
use std::net::IpAddr;

use crate::shared::domain_name::DomainName;

/// Result of executing the test command
///
/// Contains the outcome of all validation checks performed, including
/// advisory DNS warnings that don't affect the overall test result.
///
/// This type follows the same pattern as `EnvironmentList` in the list command —
/// the application layer produces structured data, the presentation layer renders it.
#[derive(Debug)]
pub struct TestResult {
    /// IP address of the tested instance
    pub instance_ip: IpAddr,
    /// Advisory DNS warnings (domains that failed to resolve or resolved to wrong IP)
    pub dns_warnings: Vec<DnsWarning>,
}

impl TestResult {
    /// Create a new `TestResult` with no warnings
    #[must_use]
    pub fn success(instance_ip: IpAddr) -> Self {
        Self {
            instance_ip,
            dns_warnings: Vec::new(),
        }
    }

    /// Create a new `TestResult` with DNS warnings
    #[must_use]
    pub fn with_dns_warnings(instance_ip: IpAddr, dns_warnings: Vec<DnsWarning>) -> Self {
        Self {
            instance_ip,
            dns_warnings,
        }
    }

    /// Check if there are any DNS warnings
    #[must_use]
    pub fn has_dns_warnings(&self) -> bool {
        !self.dns_warnings.is_empty()
    }
}

/// A single DNS resolution warning for a configured domain
#[derive(Debug)]
pub struct DnsWarning {
    /// The domain that was checked
    pub domain: DomainName,

    /// The expected IP address (instance IP)
    pub expected_ip: IpAddr,

    /// What went wrong
    pub issue: DnsIssue,
}

impl fmt::Display for DnsWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.issue {
            DnsIssue::ResolutionFailed(reason) => {
                write!(
                    f,
                    "{domain} does not resolve (expected: {ip}): {reason}",
                    domain = self.domain,
                    ip = self.expected_ip,
                )
            }
            DnsIssue::IpMismatch { resolved_ips } => {
                let ips: Vec<String> = resolved_ips.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "{domain} resolves to [{ips}] but expected {expected}",
                    domain = self.domain,
                    ips = ips.join(", "),
                    expected = self.expected_ip,
                )
            }
        }
    }
}

/// The specific issue found during DNS resolution
#[derive(Debug)]
pub enum DnsIssue {
    /// DNS resolution failed entirely (domain doesn't resolve or network error)
    ResolutionFailed(String),

    /// Domain resolved but to different IP(s) than expected
    IpMismatch {
        /// The IP addresses the domain actually resolved to
        resolved_ips: Vec<IpAddr>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ip() -> IpAddr {
        "10.0.0.1".parse().unwrap()
    }

    #[test]
    fn it_should_create_success_result_with_no_warnings() {
        let result = TestResult::success(test_ip());
        assert!(!result.has_dns_warnings());
        assert!(result.dns_warnings.is_empty());
        assert_eq!(result.instance_ip, test_ip());
    }

    #[test]
    fn it_should_create_result_with_dns_warnings() {
        let warnings = vec![DnsWarning {
            domain: DomainName::new("tracker.local").unwrap(),
            expected_ip: "10.0.0.1".parse().unwrap(),
            issue: DnsIssue::ResolutionFailed("name resolution failed".to_string()),
        }];

        let result = TestResult::with_dns_warnings(test_ip(), warnings);
        assert!(result.has_dns_warnings());
        assert_eq!(result.dns_warnings.len(), 1);
    }

    #[test]
    fn it_should_display_resolution_failed_warning() {
        let warning = DnsWarning {
            domain: DomainName::new("tracker.local").unwrap(),
            expected_ip: "10.0.0.1".parse().unwrap(),
            issue: DnsIssue::ResolutionFailed("name resolution failed".to_string()),
        };

        let display = format!("{warning}");
        assert!(display.contains("tracker.local"));
        assert!(display.contains("does not resolve"));
        assert!(display.contains("10.0.0.1"));
    }

    #[test]
    fn it_should_display_ip_mismatch_warning() {
        let warning = DnsWarning {
            domain: DomainName::new("tracker.local").unwrap(),
            expected_ip: "10.0.0.1".parse().unwrap(),
            issue: DnsIssue::IpMismatch {
                resolved_ips: vec!["192.168.1.1".parse().unwrap()],
            },
        };

        let display = format!("{warning}");
        assert!(display.contains("tracker.local"));
        assert!(display.contains("192.168.1.1"));
        assert!(display.contains("10.0.0.1"));
    }
}
