//! Error types for SSH server container operations

use std::path::PathBuf;
use thiserror::Error;

use crate::adapters::docker::DockerError;

/// Errors that can occur when working with SSH server containers
#[derive(Debug, Error)]
pub enum SshServerError {
    /// SSH server Dockerfile not found at expected location
    #[error(
        "SSH server Dockerfile not found at '{expected_path}'
Tip: Ensure 'docker/ssh-server/Dockerfile' exists in the project root"
    )]
    DockerfileNotFound { expected_path: PathBuf },

    /// Docker build command failed
    #[error("Docker build command failed for image '{image_name}:{image_tag}'
Tip: Run 'docker build -t {image_name}:{image_tag} {dockerfile_dir}' manually to see detailed errors")]
    DockerBuildFailed {
        image_name: String,
        image_tag: String,
        dockerfile_dir: String,
        stdout: String,
        stderr: String,
    },

    /// Failed to start SSH server container
    #[error(
        "Failed to start SSH server container
Tip: Check if Docker daemon is running with 'docker ps'"
    )]
    ContainerStartFailed {
        #[source]
        source: testcontainers::core::error::TestcontainersError,
    },

    /// Failed to get mapped port for SSH container
    #[error(
        "Failed to get mapped port for SSH container
Tip: Verify container is running with 'docker ps'"
    )]
    PortMappingFailed {
        #[source]
        source: testcontainers::core::error::TestcontainersError,
    },

    /// Docker command execution failed
    #[error(
        "Docker command execution failed: {command}
Tip: Verify Docker is installed and accessible: 'docker --version'"
    )]
    DockerCommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// Invalid UTF-8 in Dockerfile path
    #[error(
        "Invalid UTF-8 in Dockerfile path: '{path}'
Tip: Avoid non-ASCII characters in file paths"
    )]
    InvalidUtf8InPath { path: String },

    /// Docker client operation failed
    #[error(
        "Docker client operation failed
Tip: Check Docker daemon status with 'docker ps'"
    )]
    DockerClientError {
        #[source]
        source: Box<DockerError>,
    },
}

impl SshServerError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::RealSshServerContainer;
    ///
    /// # async fn example() {
    /// if let Err(e) = RealSshServerContainer::start().await {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerfileNotFound { .. } => {
                "Dockerfile Not Found - Detailed Troubleshooting:

1. Verify the Dockerfile exists:
   ls -la docker/ssh-server/Dockerfile

2. Check you're running from project root:
   pwd  # Should show the torrust-tracker-deployer directory

3. If using a custom Dockerfile location:
   - Update the DOCKERFILE_DIR constant in constants.rs
   - Ensure the path is relative to the project root

For more information, see the SSH server documentation."
            }

            Self::DockerBuildFailed { .. } => {
                "Docker Build Failed - Detailed Troubleshooting:

1. Run the build command manually to see full output:
   docker build -t torrust-ssh-server:latest docker/ssh-server

2. Common issues:
   - Check Dockerfile syntax
   - Verify base image is accessible: docker pull ubuntu:22.04
   - Check network connectivity for package downloads
   - Review build logs for specific error messages

3. Check Docker daemon status:
   systemctl status docker  # Linux systemd
   docker info  # General information

4. Try cleaning Docker build cache:
   docker builder prune

For more information, see Docker documentation."
            }

            Self::ContainerStartFailed { .. } => {
                "Container Start Failed - Detailed Troubleshooting:

1. Check if Docker daemon is running:
   docker ps

2. Verify sufficient resources:
   docker system df  # Check disk space
   docker info  # Check memory/CPU limits

3. Check for port conflicts:
   netstat -tlnp | grep :22  # Linux
   ss -tlnp | grep :22  # Alternative
   lsof -i :22  # macOS

4. Review Docker logs:
   docker logs <container_id>

For more information, see testcontainers documentation."
            }

            Self::PortMappingFailed { .. } => {
                "Port Mapping Failed - Detailed Troubleshooting:

1. Verify container is running:
   docker ps

2. Check container port configuration:
   docker port <container_id>

3. Check if the required port is already in use:
   netstat -tlnp  # Linux
   ss -tlnp  # Alternative
   lsof -i :22  # macOS

For more information, see Docker networking documentation."
            }

            Self::DockerCommandFailed { .. } => {
                "Docker Command Execution Failed - Detailed Troubleshooting:

1. Verify Docker is installed:
   docker --version

2. Check Docker daemon is running:
   systemctl status docker  # Linux systemd
   docker ps  # Quick check

3. Verify user permissions:
   groups  # Check if user is in 'docker' group
   sudo usermod -aG docker $USER  # Add user to docker group
   # Log out and log back in for group changes to take effect

4. Try running Docker with sudo (temporary workaround):
   sudo docker ps

For more information, see Docker installation documentation."
            }

            Self::InvalidUtf8InPath { .. } => {
                "Invalid UTF-8 in Path - Detailed Troubleshooting:

1. Check the Dockerfile path contains only valid UTF-8 characters
2. Avoid special characters, emoji, or non-ASCII characters in paths
3. Use ASCII characters for file and directory names

This is typically a configuration or system encoding issue."
            }

            Self::DockerClientError { .. } => {
                "Docker Client Operation Failed - Detailed Troubleshooting:

1. Verify Docker daemon is running:
   docker ps

2. Check Docker system status:
   docker info

3. Review Docker logs:
   journalctl -u docker  # Linux systemd
   docker system events  # Real-time events

4. Verify user permissions:
   groups  # Check if user is in 'docker' group
   sudo usermod -aG docker $USER  # Add user to docker group
   # Log out and log back in for group changes to take effect

For more information, see the source error details and Docker documentation."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    /// Helper to create a test `TestcontainersError`
    /// Note: We use `PortNotExposed` variant as it doesn't require many fields
    fn create_testcontainers_error() -> testcontainers::core::error::TestcontainersError {
        testcontainers::core::error::TestcontainersError::PortNotExposed {
            id: "test-container".to_string(),
            port: testcontainers::core::ContainerPort::Tcp(22),
        }
    }

    mod error_construction {
        use super::*;

        #[test]
        fn it_should_create_dockerfile_not_found_error() {
            let path = PathBuf::from("/path/to/dockerfile");
            let error = SshServerError::DockerfileNotFound {
                expected_path: path.clone(),
            };

            let message = error.to_string();
            assert!(message.contains("/path/to/dockerfile"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_docker_build_failed_error() {
            let error = SshServerError::DockerBuildFailed {
                image_name: "test-image".to_string(),
                image_tag: "latest".to_string(),
                dockerfile_dir: "/path/to/dockerfile".to_string(),
                stdout: "Building...".to_string(),
                stderr: "Error: base image not found".to_string(),
            };

            let message = error.to_string();
            assert!(message.contains("test-image:latest"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_container_start_failed_error() {
            let source = create_testcontainers_error();
            let error = SshServerError::ContainerStartFailed { source };

            let message = error.to_string();
            assert!(message.contains("Failed to start SSH server container"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_port_mapping_failed_error() {
            let source = create_testcontainers_error();
            let error = SshServerError::PortMappingFailed { source };

            let message = error.to_string();
            assert!(message.contains("Failed to get mapped port"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_docker_command_failed_error() {
            let source = io::Error::new(io::ErrorKind::NotFound, "docker not found");
            let error = SshServerError::DockerCommandFailed {
                command: "docker build".to_string(),
                source,
            };

            let message = error.to_string();
            assert!(message.contains("docker build"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_invalid_utf8_in_path_error() {
            let error = SshServerError::InvalidUtf8InPath {
                path: "/invalid/path/��".to_string(),
            };

            let message = error.to_string();
            assert!(message.contains("Invalid UTF-8"));
            assert!(message.contains("Tip:"));
        }

        #[test]
        fn it_should_create_docker_client_error() {
            let docker_error = DockerError::BuildFailed {
                image: "test:latest".to_string(),
                source: crate::shared::command::CommandError::ExecutionFailed {
                    command: "docker".to_string(),
                    exit_code: "1".to_string(),
                    stdout: String::new(),
                    stderr: "Build failed".to_string(),
                },
            };
            let error = SshServerError::DockerClientError {
                source: Box::new(docker_error),
            };

            let message = error.to_string();
            assert!(message.contains("Docker client operation failed"));
            assert!(message.contains("Tip:"));
        }
    }

    mod error_messages {
        use super::*;

        #[test]
        fn it_should_include_tip_in_all_error_messages() {
            let errors = vec![
                SshServerError::DockerfileNotFound {
                    expected_path: PathBuf::from("/test"),
                },
                SshServerError::DockerBuildFailed {
                    image_name: "test".to_string(),
                    image_tag: "latest".to_string(),
                    dockerfile_dir: "/test".to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
                },
                SshServerError::ContainerStartFailed {
                    source: create_testcontainers_error(),
                },
                SshServerError::PortMappingFailed {
                    source: create_testcontainers_error(),
                },
                SshServerError::DockerCommandFailed {
                    command: "test".to_string(),
                    source: io::Error::other("test"),
                },
                SshServerError::InvalidUtf8InPath {
                    path: "test".to_string(),
                },
                SshServerError::DockerClientError {
                    source: Box::new(DockerError::BuildFailed {
                        image: "test:latest".to_string(),
                        source: crate::shared::command::CommandError::ExecutionFailed {
                            command: "docker".to_string(),
                            exit_code: "1".to_string(),
                            stdout: String::new(),
                            stderr: "error".to_string(),
                        },
                    }),
                },
            ];

            for error in errors {
                let message = error.to_string();
                assert!(
                    message.contains("Tip:"),
                    "Error message should contain tip: {message}"
                );
            }
        }

        #[test]
        fn it_should_provide_clear_context_in_error_messages() {
            // DockerfileNotFound should include path
            let error = SshServerError::DockerfileNotFound {
                expected_path: PathBuf::from("/custom/path/Dockerfile"),
            };
            assert!(error.to_string().contains("/custom/path/Dockerfile"));

            // DockerBuildFailed should include image name and tag
            let error = SshServerError::DockerBuildFailed {
                image_name: "my-image".to_string(),
                image_tag: "v1.0".to_string(),
                dockerfile_dir: "/path".to_string(),
                stdout: String::new(),
                stderr: String::new(),
            };
            let message = error.to_string();
            assert!(message.contains("my-image:v1.0"));

            // DockerCommandFailed should include command
            let error = SshServerError::DockerCommandFailed {
                command: "docker ps -a".to_string(),
                source: io::Error::other("test"),
            };
            assert!(error.to_string().contains("docker ps -a"));
        }
    }

    mod help_methods {
        use super::*;

        #[test]
        fn it_should_provide_help_for_all_error_variants() {
            let errors = vec![
                SshServerError::DockerfileNotFound {
                    expected_path: PathBuf::from("/test"),
                },
                SshServerError::DockerBuildFailed {
                    image_name: "test".to_string(),
                    image_tag: "latest".to_string(),
                    dockerfile_dir: "/test".to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
                },
                SshServerError::ContainerStartFailed {
                    source: create_testcontainers_error(),
                },
                SshServerError::PortMappingFailed {
                    source: create_testcontainers_error(),
                },
                SshServerError::DockerCommandFailed {
                    command: "test".to_string(),
                    source: io::Error::other("test"),
                },
                SshServerError::InvalidUtf8InPath {
                    path: "test".to_string(),
                },
                SshServerError::DockerClientError {
                    source: Box::new(DockerError::BuildFailed {
                        image: "test:latest".to_string(),
                        source: crate::shared::command::CommandError::ExecutionFailed {
                            command: "docker".to_string(),
                            exit_code: "1".to_string(),
                            stdout: String::new(),
                            stderr: "error".to_string(),
                        },
                    }),
                },
            ];

            for error in errors {
                let help = error.help();
                assert!(
                    !help.is_empty(),
                    "Help should not be empty for error: {error}"
                );
                assert!(
                    help.contains("Troubleshooting"),
                    "Help should contain troubleshooting section for error: {error}"
                );
            }
        }

        #[test]
        fn it_should_include_platform_specific_commands_in_help() {
            // Check that help includes platform-specific commands
            let error = SshServerError::ContainerStartFailed {
                source: create_testcontainers_error(),
            };
            let help = error.help();

            // Should include both netstat and ss (Linux alternatives)
            assert!(help.contains("netstat") || help.contains("ss"));
        }

        #[test]
        fn it_should_include_actionable_steps_in_help() {
            let error = SshServerError::DockerBuildFailed {
                image_name: "test".to_string(),
                image_tag: "latest".to_string(),
                dockerfile_dir: "/test".to_string(),
                stdout: String::new(),
                stderr: String::new(),
            };

            let help = error.help();

            // Should include numbered steps
            assert!(help.contains("1."));
            assert!(help.contains("2."));

            // Should include specific commands
            assert!(help.contains("docker"));
        }
    }

    mod error_source_chaining {
        use super::*;

        #[test]
        fn it_should_preserve_source_error_for_container_start_failed() {
            let source = create_testcontainers_error();
            let error = SshServerError::ContainerStartFailed { source };

            // Should be able to access source through Error trait
            assert!(
                std::error::Error::source(&error).is_some(),
                "ContainerStartFailed should preserve source error"
            );
        }

        #[test]
        fn it_should_preserve_source_error_for_port_mapping_failed() {
            let source = create_testcontainers_error();
            let error = SshServerError::PortMappingFailed { source };

            assert!(
                std::error::Error::source(&error).is_some(),
                "PortMappingFailed should preserve source error"
            );
        }

        #[test]
        fn it_should_preserve_source_error_for_docker_command_failed() {
            let source = io::Error::other("test error");
            let error = SshServerError::DockerCommandFailed {
                command: "test".to_string(),
                source,
            };

            assert!(
                std::error::Error::source(&error).is_some(),
                "DockerCommandFailed should preserve source error"
            );
        }

        #[test]
        fn it_should_preserve_source_error_for_docker_client_error() {
            let docker_error = DockerError::BuildFailed {
                image: "test:latest".to_string(),
                source: crate::shared::command::CommandError::ExecutionFailed {
                    command: "docker".to_string(),
                    exit_code: "1".to_string(),
                    stdout: String::new(),
                    stderr: "error".to_string(),
                },
            };
            let error = SshServerError::DockerClientError {
                source: Box::new(docker_error),
            };

            assert!(
                std::error::Error::source(&error).is_some(),
                "DockerClientError should preserve source error"
            );
        }
    }

    mod pattern_matching {
        use super::*;

        #[test]
        fn it_should_enable_pattern_matching_on_error_variants() {
            let error = SshServerError::DockerfileNotFound {
                expected_path: PathBuf::from("/test"),
            };

            // Should be able to match on specific error variant
            match error {
                SshServerError::DockerfileNotFound { expected_path } => {
                    assert_eq!(expected_path, PathBuf::from("/test"));
                }
                _ => panic!("Should match DockerfileNotFound variant"),
            }
        }

        #[test]
        fn it_should_extract_error_context_through_pattern_matching() {
            let error = SshServerError::DockerBuildFailed {
                image_name: "my-image".to_string(),
                image_tag: "v2.0".to_string(),
                dockerfile_dir: "/custom/path".to_string(),
                stdout: "build output".to_string(),
                stderr: "build errors".to_string(),
            };

            match error {
                SshServerError::DockerBuildFailed {
                    image_name,
                    image_tag,
                    dockerfile_dir,
                    stdout,
                    stderr,
                } => {
                    assert_eq!(image_name, "my-image");
                    assert_eq!(image_tag, "v2.0");
                    assert_eq!(dockerfile_dir, "/custom/path");
                    assert_eq!(stdout, "build output");
                    assert_eq!(stderr, "build errors");
                }
                _ => panic!("Should match DockerBuildFailed variant"),
            }
        }
    }

    mod error_display {
        use super::*;

        #[test]
        fn it_should_implement_debug_trait() {
            let error = SshServerError::DockerfileNotFound {
                expected_path: PathBuf::from("/test"),
            };

            let debug_output = format!("{error:?}");
            assert!(!debug_output.is_empty());
            assert!(debug_output.contains("DockerfileNotFound"));
        }

        #[test]
        fn it_should_implement_display_trait() {
            let error = SshServerError::InvalidUtf8InPath {
                path: "/test".to_string(),
            };

            let display_output = format!("{error}");
            assert!(!display_output.is_empty());
            assert!(display_output.contains("Invalid UTF-8"));
        }
    }
}
