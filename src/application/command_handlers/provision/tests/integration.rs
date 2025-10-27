//! Integration tests for Provision Command
//!
//! This module contains integration tests for the `ProvisionCommandHandler`.

use std::sync::Arc;

use chrono::{TimeZone, Utc};

use super::builders::{create_test_environment, ProvisionCommandHandlerTestBuilder};
use crate::adapters::ssh::SshError;
use crate::adapters::tofu::client::OpenTofuError;
use crate::application::command_handlers::provision::ProvisionCommandHandlerError;
use crate::domain::environment::state::ProvisionStep;
use crate::infrastructure::external_tools::tofu::ProvisionTemplateError;
use crate::shared::command::CommandError;

#[test]
fn it_should_create_provision_command_handler_with_all_dependencies() {
    let (command_handler, _temp_dir, _ssh_credentials) =
        ProvisionCommandHandlerTestBuilder::new().build();

    // Verify the command handler was created (basic structure test)
    // This test just verifies that the command handler can be created with the dependencies
    assert_eq!(
        Arc::strong_count(&command_handler.tofu_template_renderer),
        1
    );
    assert_eq!(
        Arc::strong_count(&command_handler.ansible_template_renderer),
        1
    );
}

#[test]
fn it_should_have_correct_error_type_conversions() {
    // Test that all error types can convert to ProvisionCommandHandlerError
    let template_error = ProvisionTemplateError::DirectoryCreationFailed {
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

#[test]
fn it_should_build_failure_context_from_opentofu_template_error() {
    let (command_handler, temp_dir, _ssh_credentials) =
        ProvisionCommandHandlerTestBuilder::new().build();

    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

    let error = ProvisionCommandHandlerError::OpenTofuTemplateRendering(
        ProvisionTemplateError::DirectoryCreationFailed {
            directory: "/test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        },
    );

    let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let current_step = ProvisionStep::RenderOpenTofuTemplates;
    let context =
        command_handler.build_failure_context(&environment, &error, current_step, started_at);
    assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
    assert_eq!(
        context.error_kind,
        crate::shared::ErrorKind::TemplateRendering
    );
    assert_eq!(context.base.execution_started_at, started_at);
}

// Note: We don't test AnsibleTemplateRendering errors directly as the error types are complex
// and deeply nested. The build_failure_context method handles them by matching on the
// ProvisionCommandHandlerError::AnsibleTemplateRendering variant, which is sufficient for
// error context generation.

#[test]
fn it_should_build_failure_context_from_ssh_connectivity_error() {
    let (command_handler, temp_dir, _ssh_credentials) =
        ProvisionCommandHandlerTestBuilder::new().build();

    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

    let error = ProvisionCommandHandlerError::SshConnectivity(SshError::ConnectivityTimeout {
        host_ip: "127.0.0.1".to_string(),
        attempts: 5,
        timeout_seconds: 30,
    });

    let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let current_step = ProvisionStep::WaitSshConnectivity;
    let context =
        command_handler.build_failure_context(&environment, &error, current_step, started_at);
    assert_eq!(context.failed_step, ProvisionStep::WaitSshConnectivity);
    assert_eq!(
        context.error_kind,
        crate::shared::ErrorKind::NetworkConnectivity
    );
    assert_eq!(context.base.execution_started_at, started_at);
}

#[test]
fn it_should_build_failure_context_from_command_error() {
    let (command_handler, temp_dir, _ssh_credentials) =
        ProvisionCommandHandlerTestBuilder::new().build();

    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

    let error = ProvisionCommandHandlerError::Command(CommandError::ExecutionFailed {
        command: "test".to_string(),
        exit_code: "1".to_string(),
        stdout: String::new(),
        stderr: "test error".to_string(),
    });

    let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let current_step = ProvisionStep::CloudInitWait;
    let context =
        command_handler.build_failure_context(&environment, &error, current_step, started_at);
    assert_eq!(context.failed_step, ProvisionStep::CloudInitWait);
    assert_eq!(
        context.error_kind,
        crate::shared::ErrorKind::CommandExecution
    );
    assert_eq!(context.base.execution_started_at, started_at);
}

#[test]
fn it_should_build_failure_context_from_opentofu_error() {
    let (command_handler, temp_dir, _ssh_credentials) =
        ProvisionCommandHandlerTestBuilder::new().build();

    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

    let opentofu_error = OpenTofuError::CommandError(CommandError::ExecutionFailed {
        command: "tofu init".to_string(),
        exit_code: "1".to_string(),
        stdout: String::new(),
        stderr: "init failed".to_string(),
    });

    let error = ProvisionCommandHandlerError::OpenTofu(opentofu_error);

    let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let current_step = ProvisionStep::OpenTofuInit;
    let context =
        command_handler.build_failure_context(&environment, &error, current_step, started_at);
    assert_eq!(context.failed_step, ProvisionStep::OpenTofuInit);
    assert_eq!(
        context.error_kind,
        crate::shared::ErrorKind::InfrastructureOperation
    );
    assert_eq!(context.base.execution_started_at, started_at);
}
