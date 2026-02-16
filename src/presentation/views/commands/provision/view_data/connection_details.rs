//! Connection Details View for Provision Command
//!
//! This module provides a view for rendering SSH connection details after
//! successful infrastructure provisioning.

use std::net::IpAddr;
use std::path::PathBuf;

use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;

/// Connection details data for rendering
///
/// This struct holds all the data needed to render SSH connection information
/// for a provisioned instance.
#[derive(Debug, Clone)]
pub struct ConnectionDetailsData {
    /// Instance IP address (None if not available)
    pub instance_ip: Option<IpAddr>,
    /// SSH port for connections
    pub ssh_port: u16,
    /// Path to SSH private key
    pub ssh_priv_key_path: PathBuf,
    /// SSH username for connections
    pub ssh_username: String,
}

/// Conversion from domain model to presentation DTO
///
/// This `From` trait implementation is placed in the presentation layer
/// (not in the domain layer) to maintain proper DDD layering:
///
/// - Domain layer should not depend on presentation layer DTOs
/// - Presentation layer can depend on domain models (allowed)
/// - This keeps the domain clean and focused on business logic
///
/// Alternative approaches considered:
/// - Adding method to `Environment<Provisioned>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Provisioned>> for ConnectionDetailsData {
    fn from(provisioned: &Environment<Provisioned>) -> Self {
        Self {
            instance_ip: provisioned.instance_ip(),
            ssh_port: provisioned.ssh_port(),
            ssh_priv_key_path: provisioned.ssh_private_key_path().clone(),
            ssh_username: provisioned.ssh_username().as_str().to_string(),
        }
    }
}

/// View for rendering SSH connection details
///
/// This view is responsible for formatting and rendering the connection
/// information that users need to SSH into a provisioned instance.
///
/// # Design
///
/// Following MVC pattern, this view:
/// - Receives data from the controller
/// - Formats the output for display
/// - Handles missing data gracefully
/// - Returns a string ready for output to stdout
///
/// # Examples
///
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr};
/// use std::path::PathBuf;
/// use torrust_tracker_deployer_lib::presentation::views::commands::provision::ConnectionDetailsView;
/// use torrust_tracker_deployer_lib::presentation::views::commands::provision::view_data::connection_details::ConnectionDetailsData;
///
/// let data = ConnectionDetailsData {
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171))),
///     ssh_port: 22,
///     ssh_priv_key_path: PathBuf::from("fixtures/testing_rsa"),
///     ssh_username: "torrust".to_string(),
/// };
///
/// let output = ConnectionDetailsView::render(&data);
/// assert!(output.contains("Instance Connection Details"));
/// assert!(output.contains("10.140.190.171"));
/// ```
pub struct ConnectionDetailsView;

impl ConnectionDetailsView {
    /// Render connection details as a formatted string
    ///
    /// Takes connection data and produces a human-readable output suitable
    /// for displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - Connection details to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Section header
    /// - IP address (or warning if missing)
    /// - SSH port
    /// - SSH private key path (absolute - as provided)
    /// - SSH username
    /// - Ready-to-copy SSH command (if IP is available)
    ///
    /// # Missing IP Handling
    ///
    /// If the instance IP is not available (which should not happen after
    /// successful provisioning), the view displays a warning message and
    /// omits the SSH connection command.
    ///
    /// # Path Handling
    ///
    /// SSH private key paths are expected to be absolute paths and are
    /// displayed as-is without further resolution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::views::commands::provision::ConnectionDetailsView;
    /// use torrust_tracker_deployer_lib::presentation::views::commands::provision::view_data::connection_details::ConnectionDetailsData;
    ///
    /// let data = ConnectionDetailsData {
    ///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
    ///     ssh_port: 2222,
    ///     ssh_priv_key_path: PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     ssh_username: "admin".to_string(),
    /// };
    ///
    /// let output = ConnectionDetailsView::render(&data);
    /// assert!(output.contains("192.168.1.100"));
    /// assert!(output.contains("2222"));
    /// assert!(output.contains("admin"));
    /// ```
    #[must_use]
    pub fn render(data: &ConnectionDetailsData) -> String {
        match data.instance_ip {
            Some(ip) => format!(
                "\nInstance Connection Details:\n\
                 \x20 IP Address:        {}\n\
                 \x20 SSH Port:          {}\n\
                 \x20 SSH Private Key:   {}\n\
                 \x20 SSH Username:      {}\n\
                 \n\
                 Connect using:\n\
                 \x20 ssh -i {} {}@{} -p {}",
                ip,
                data.ssh_port,
                data.ssh_priv_key_path.display(),
                data.ssh_username,
                data.ssh_priv_key_path.display(),
                data.ssh_username,
                ip,
                data.ssh_port
            ),
            None => format!(
                "\nInstance Connection Details:\n\
                 \x20 IP Address:        <not available>\n\
                 \x20 WARNING: Instance IP not captured - this is an unexpected state.\n\
                 \x20          The environment may not be fully provisioned.\n\
                 \x20 SSH Port:          {}\n\
                 \x20 SSH Private Key:   {}\n\
                 \x20 SSH Username:      {}",
                data.ssh_port,
                data.ssh_priv_key_path.display(),
                data.ssh_username
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn it_should_render_complete_connection_details_when_ip_is_available() {
        let data = ConnectionDetailsData {
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171))),
            ssh_port: 22,
            ssh_priv_key_path: PathBuf::from("fixtures/testing_rsa"),
            ssh_username: "torrust".to_string(),
        };

        let output = ConnectionDetailsView::render(&data);

        assert!(output.contains("Instance Connection Details"));
        assert!(output.contains("10.140.190.171"));
        assert!(output.contains("SSH Port:          22"));
        assert!(output.contains("SSH Username:      torrust"));
        assert!(output.contains("Connect using:"));
        assert!(output.contains("ssh -i"));
    }

    #[test]
    fn it_should_render_warning_when_ip_is_missing() {
        let data = ConnectionDetailsData {
            instance_ip: None,
            ssh_port: 22,
            ssh_priv_key_path: PathBuf::from("fixtures/testing_rsa"),
            ssh_username: "torrust".to_string(),
        };

        let output = ConnectionDetailsView::render(&data);

        assert!(output.contains("Instance Connection Details"));
        assert!(output.contains("<not available>"));
        assert!(output.contains("WARNING"));
        assert!(output.contains("unexpected state"));
        assert!(!output.contains("Connect using:"));
    }

    #[test]
    fn it_should_display_custom_ssh_port() {
        let data = ConnectionDetailsData {
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
            ssh_port: 2222,
            ssh_priv_key_path: PathBuf::from("/home/user/.ssh/key"),
            ssh_username: "admin".to_string(),
        };

        let output = ConnectionDetailsView::render(&data);

        assert!(output.contains("SSH Port:          2222"));
        assert!(output.contains("-p 2222"));
    }

    #[test]
    fn it_should_include_absolute_path_in_output() {
        let data = ConnectionDetailsData {
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))),
            ssh_port: 22,
            ssh_priv_key_path: PathBuf::from("/absolute/path/to/key"),
            ssh_username: "user".to_string(),
        };

        let output = ConnectionDetailsView::render(&data);

        // Should contain the path as provided
        assert!(output.contains("SSH Private Key:"));
        assert!(output.contains("/absolute/path/to/key"));
    }

    #[test]
    fn it_should_preserve_absolute_paths() {
        let data = ConnectionDetailsData {
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            ssh_port: 22,
            ssh_priv_key_path: PathBuf::from("/absolute/path/to/key"),
            ssh_username: "deploy".to_string(),
        };

        let output = ConnectionDetailsView::render(&data);

        assert!(output.contains("/absolute/path/to/key"));
    }
}
