//! DNS resolver implementation
//!
//! Provides system DNS resolution to check if configured domains resolve to
//! the expected instance IP addresses.

use std::net::{IpAddr, ToSocketAddrs};

use thiserror::Error;
use tracing::{debug, instrument};

use crate::shared::domain_name::DomainName;

/// Errors that can occur during DNS resolution
#[derive(Error, Debug)]
pub enum DnsResolutionError {
    /// DNS resolution failed (domain doesn't resolve or network error)
    #[error("DNS resolution failed for domain '{domain}': {source}")]
    ResolutionFailed {
        domain: String,
        #[source]
        source: std::io::Error,
    },

    /// Domain resolved but to a different IP than expected
    #[error("Domain '{domain}' resolves to {resolved_ip} but expected {expected_ip}")]
    IpMismatch {
        domain: String,
        resolved_ip: IpAddr,
        expected_ip: IpAddr,
    },
}

/// DNS resolver for validating domain name resolution
///
/// Uses the system's DNS resolver (`std::net::ToSocketAddrs`) to check if
/// domains resolve to expected IP addresses. This checks actual DNS configuration
/// including system DNS servers, `/etc/hosts`, and mDNS for `.local` domains.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// use torrust_tracker_deployer::infrastructure::dns::DnsResolver;
/// use torrust_tracker_deployer::shared::domain_name::DomainName;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let resolver = DnsResolver::new();
/// let domain = DomainName::new("tracker.local")?;
/// let expected_ip: IpAddr = "10.140.190.254".parse()?;
///
/// match resolver.resolve_and_verify(&domain, expected_ip) {
///     Ok(()) => println!("✓ Domain resolves correctly"),
///     Err(e) => eprintln!("⚠ DNS check failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DnsResolver;

impl DnsResolver {
    /// Create a new DNS resolver
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolve a domain name to IP addresses using system DNS
    ///
    /// # Arguments
    /// * `domain` - Domain name to resolve
    ///
    /// # Returns
    /// Vector of resolved IP addresses (may be empty if resolution fails)
    ///
    /// # Errors
    /// Returns `DnsResolutionError::ResolutionFailed` if DNS resolution fails
    #[instrument(skip(self), fields(domain = %domain.as_str()))]
    pub fn resolve(&self, domain: &DomainName) -> Result<Vec<IpAddr>, DnsResolutionError> {
        debug!("Resolving domain via system DNS");

        // Use port 80 as a dummy port for ToSocketAddrs
        // The port doesn't matter for DNS resolution, but ToSocketAddrs requires it
        let address_with_port = format!("{}:80", domain.as_str());

        let addresses: Vec<IpAddr> = address_with_port
            .to_socket_addrs()
            .map_err(|e| DnsResolutionError::ResolutionFailed {
                domain: domain.as_str().to_string(),
                source: e,
            })?
            .map(|addr| addr.ip())
            .collect();

        debug!(resolved_ips = ?addresses, "Domain resolved successfully");

        Ok(addresses)
    }

    /// Resolve a domain and verify it matches the expected IP address
    ///
    /// This method resolves the domain and checks if any of the resolved IPs
    /// match the expected IP. It's common for domains to resolve to multiple
    /// IPs (IPv4 and IPv6, or multiple servers).
    ///
    /// # Arguments
    /// * `domain` - Domain name to resolve and verify
    /// * `expected_ip` - Expected IP address
    ///
    /// # Returns
    /// `Ok(())` if the domain resolves and matches the expected IP
    ///
    /// # Errors
    /// - `DnsResolutionError::ResolutionFailed` if DNS resolution fails
    /// - `DnsResolutionError::IpMismatch` if domain resolves but not to expected IP
    #[instrument(skip(self), fields(domain = %domain.as_str(), expected_ip = %expected_ip))]
    pub fn resolve_and_verify(
        &self,
        domain: &DomainName,
        expected_ip: IpAddr,
    ) -> Result<(), DnsResolutionError> {
        let resolved_ips = self.resolve(domain)?;

        if resolved_ips.is_empty() {
            return Err(DnsResolutionError::ResolutionFailed {
                domain: domain.as_str().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "DNS resolution returned no addresses",
                ),
            });
        }

        // Check if any of the resolved IPs match the expected IP
        if resolved_ips.contains(&expected_ip) {
            debug!("Domain resolves to expected IP");
            Ok(())
        } else {
            // Return the first resolved IP in the error for clarity
            Err(DnsResolutionError::IpMismatch {
                domain: domain.as_str().to_string(),
                resolved_ip: resolved_ips[0],
                expected_ip,
            })
        }
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_new_resolver() {
        let resolver = DnsResolver::new();
        assert!(matches!(resolver, DnsResolver));
    }

    // Note: We can't test actual DNS resolution in unit tests as it depends on
    // the system's DNS configuration and network connectivity. These tests would
    // be better suited for integration tests with a controlled DNS environment.
    //
    // For comprehensive testing, see E2E tests which validate DNS checks against
    // running infrastructure.
}
