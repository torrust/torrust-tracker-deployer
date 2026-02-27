//! Infrastructure Information View
//!
//! This module provides a view for rendering infrastructure details
//! including IP address, SSH credentials, and connection commands.

use crate::presentation::cli::views::commands::show::view_data::InfrastructureInfo;

/// View for rendering infrastructure information
///
/// This view handles the display of infrastructure details that become
/// available after an environment has been provisioned.
pub struct InfrastructureView;

impl InfrastructureView {
    /// Render infrastructure information as formatted lines
    ///
    /// # Arguments
    ///
    /// * `infra` - Infrastructure information containing IP, SSH details
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(infra: &InfrastructureInfo) -> Vec<String> {
        let mut lines = vec![
            String::new(), // blank line
            "Infrastructure:".to_string(),
            format!("  Instance IP: {}", infra.instance_ip),
            format!("  SSH Port: {}", infra.ssh_port),
            format!("  SSH User: {}", infra.ssh_user),
            format!("  SSH Key: {}", infra.ssh_key_path),
            String::new(), // blank line
            "Connection:".to_string(),
            format!("  {}", infra.ssh_command()),
        ];

        // Hint for Docker users when container path pattern detected
        if Self::looks_like_container_path(&infra.ssh_key_path) {
            lines.push(String::new()); // blank line
            lines.push("Note: Paths shown are inside the container.".to_string());
            lines.push(
                "      If using Docker, translate to your host path (e.g., ~/.ssh/).".to_string(),
            );
        }

        lines
    }

    /// Check if a path looks like it's inside the Docker container.
    ///
    /// The deployer Docker image uses `/home/deployer/` as the home directory.
    /// When paths contain this prefix, it indicates the environment was managed
    /// from inside a container, and users need to translate paths to their host
    /// equivalents.
    fn looks_like_container_path(path: &str) -> bool {
        path.starts_with("/home/deployer/")
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    fn sample_infrastructure() -> InfrastructureInfo {
        InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171)),
            22,
            "torrust".to_string(),
            "~/.ssh/id_rsa".to_string(),
        )
    }

    #[test]
    fn it_should_render_instance_ip() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(lines
            .iter()
            .any(|l| l.contains("Instance IP: 10.140.190.171")));
    }

    #[test]
    fn it_should_render_ssh_port() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(lines.iter().any(|l| l.contains("SSH Port: 22")));
    }

    #[test]
    fn it_should_render_ssh_user() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(lines.iter().any(|l| l.contains("SSH User: torrust")));
    }

    #[test]
    fn it_should_render_ssh_key_path() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(lines.iter().any(|l| l.contains("SSH Key: ~/.ssh/id_rsa")));
    }

    #[test]
    fn it_should_render_ssh_connection_command() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(lines.iter().any(|l| l.contains("ssh -i")));
    }

    #[test]
    fn it_should_include_port_in_ssh_command_when_non_standard() {
        let infra = InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            2222,
            "user".to_string(),
            "/key".to_string(),
        );

        let lines = InfrastructureView::render(&infra);
        assert!(lines.iter().any(|l| l.contains("-p 2222")));
    }

    #[test]
    fn it_should_show_docker_hint_when_container_path_detected() {
        let infra = InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            22,
            "torrust".to_string(),
            "/home/deployer/.ssh/id_rsa".to_string(),
        );

        let lines = InfrastructureView::render(&infra);
        assert!(lines
            .iter()
            .any(|l| l.contains("Paths shown are inside the container")));
    }

    #[test]
    fn it_should_not_show_docker_hint_for_regular_paths() {
        let lines = InfrastructureView::render(&sample_infrastructure());
        assert!(!lines
            .iter()
            .any(|l| l.contains("Paths shown are inside the container")));
    }
}
