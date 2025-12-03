//! Integration tests for Provision Command
//!
//! This module contains integration tests for the `ProvisionCommandHandler`.

use crate::adapters::ssh::SshError;
use crate::adapters::tofu::client::OpenTofuError;
use crate::application::command_handlers::provision::ProvisionCommandHandlerError;
use crate::infrastructure::external_tools::tofu::TofuTemplateRendererError;
use crate::shared::command::CommandError;

#[test]
fn it_should_have_correct_error_type_conversions() {
    // Test that all error types can convert to ProvisionCommandHandlerError
    let template_error = TofuTemplateRendererError::DirectoryCreationFailed {
        directory: "/test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
    };
    let provision_error: ProvisionCommandHandlerError = template_error.into();
    drop(provision_error);

    let command_error = CommandError::StartupFailed {
        command: "test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    let opentofu_error = OpenTofuError::CommandError(command_error);
    let provision_error: ProvisionCommandHandlerError = opentofu_error.into();
    drop(provision_error);

    let command_error_direct = CommandError::ExecutionFailed {
        command: "test".to_string(),
        exit_code: "1".to_string(),
        stdout: String::new(),
        stderr: "test error".to_string(),
    };
    let provision_error: ProvisionCommandHandlerError = command_error_direct.into();
    drop(provision_error);

    let ssh_error = SshError::ConnectivityTimeout {
        host_ip: "127.0.0.1".to_string(),
        attempts: 5,
        timeout_seconds: 30,
    };
    let provision_error: ProvisionCommandHandlerError = ssh_error.into();
    drop(provision_error);
}
