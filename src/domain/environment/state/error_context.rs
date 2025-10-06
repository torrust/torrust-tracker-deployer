//! Error context types for failed command states
//!
//! This module provides structured, type-safe error contexts for commands that fail.
//! Each command has its own failure context with command-specific step and error kind enums.

use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::TraceId;

// ============================================================================
// Provision Command Error Context
// ============================================================================

/// Error context for provision command failures
///
/// Captures comprehensive information about provision failures including
/// the specific step that failed, error classification, timing, and trace details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvisionFailureContext {
    /// Which step failed during provisioning
    pub failed_step: ProvisionStep,

    /// Error category for type-safe handling
    pub error_kind: ProvisionErrorKind,

    /// Human-readable error summary
    pub error_summary: String,

    /// When the failure occurred
    pub failed_at: DateTime<Utc>,

    /// When execution started
    pub execution_started_at: DateTime<Utc>,

    /// How long execution ran before failing
    pub execution_duration: Duration,

    /// Unique trace identifier
    pub trace_id: TraceId,

    /// Path to the detailed trace file (if generated)
    pub trace_file_path: Option<PathBuf>,
}

/// Steps in the provision workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionStep {
    /// Rendering `OpenTofu` templates
    RenderOpenTofuTemplates,
    /// Initializing `OpenTofu`
    OpenTofuInit,
    /// Validating infrastructure configuration
    OpenTofuValidate,
    /// Planning infrastructure changes
    OpenTofuPlan,
    /// Applying infrastructure changes
    OpenTofuApply,
    /// Retrieving instance information
    GetInstanceInfo,
    /// Rendering Ansible templates with runtime data
    RenderAnsibleTemplates,
    /// Waiting for SSH connectivity
    WaitSshConnectivity,
    /// Waiting for cloud-init completion
    CloudInitWait,
}

/// Error categories for provision failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionErrorKind {
    /// Template rendering failed
    TemplateRendering,
    /// Infrastructure provisioning failed (`OpenTofu` operations)
    InfrastructureProvisioning,
    /// Network connectivity issues
    NetworkConnectivity,
    /// Configuration or initialization timeout
    ConfigurationTimeout,
}

// ============================================================================
// Configure Command Error Context
// ============================================================================

/// Error context for configure command failures
///
/// Captures comprehensive information about configuration failures including
/// the specific step that failed, error classification, timing, and trace details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFailureContext {
    /// Which step failed during configuration
    pub failed_step: ConfigureStep,

    /// Error category for type-safe handling
    pub error_kind: ConfigureErrorKind,

    /// Human-readable error summary
    pub error_summary: String,

    /// When the failure occurred
    pub failed_at: DateTime<Utc>,

    /// When execution started
    pub execution_started_at: DateTime<Utc>,

    /// How long execution ran before failing
    pub execution_duration: Duration,

    /// Unique trace identifier
    pub trace_id: TraceId,

    /// Path to the detailed trace file (if generated)
    pub trace_file_path: Option<PathBuf>,
}

/// Steps in the configure workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureStep {
    /// Installing Docker
    InstallDocker,
    /// Installing Docker Compose
    InstallDockerCompose,
}

/// Error categories for configure failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureErrorKind {
    /// Software installation failed
    InstallationFailed,
    /// Command execution failed
    CommandExecutionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_serialize_provision_failure_context() {
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::OpenTofuApply,
            error_kind: ProvisionErrorKind::InfrastructureProvisioning,
            error_summary: "Infrastructure provisioning failed".to_string(),
            failed_at: Utc::now(),
            execution_started_at: Utc::now(),
            execution_duration: Duration::from_secs(30),
            trace_id: TraceId::new(),
            trace_file_path: Some(PathBuf::from("/data/env/traces/trace.log")),
        };

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("OpenTofuApply"));
        assert!(json.contains("InfrastructureProvisioning"));
    }

    #[test]
    fn it_should_deserialize_provision_failure_context() {
        let trace_id = TraceId::new();
        let json = format!(
            r#"{{
                "failed_step": "RenderOpenTofuTemplates",
                "error_kind": "TemplateRendering",
                "error_summary": "Template rendering failed",
                "failed_at": "2025-10-06T10:00:00Z",
                "execution_started_at": "2025-10-06T09:59:00Z",
                "execution_duration": {{"secs": 60, "nanos": 0}},
                "trace_id": "{trace_id}",
                "trace_file_path": null
            }}"#
        );

        let context: ProvisionFailureContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
        assert_eq!(context.error_kind, ProvisionErrorKind::TemplateRendering);
    }

    #[test]
    fn it_should_serialize_configure_failure_context() {
        let context = ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ConfigureErrorKind::InstallationFailed,
            error_summary: "Docker installation failed".to_string(),
            failed_at: Utc::now(),
            execution_started_at: Utc::now(),
            execution_duration: Duration::from_secs(15),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("InstallDocker"));
        assert!(json.contains("InstallationFailed"));
    }

    #[test]
    fn it_should_deserialize_configure_failure_context() {
        let trace_id = TraceId::new();
        let json = format!(
            r#"{{
                "failed_step": "InstallDockerCompose",
                "error_kind": "CommandExecutionFailed",
                "error_summary": "Command execution failed",
                "failed_at": "2025-10-06T10:00:00Z",
                "execution_started_at": "2025-10-06T09:59:30Z",
                "execution_duration": {{"secs": 30, "nanos": 0}},
                "trace_id": "{trace_id}",
                "trace_file_path": null
            }}"#
        );

        let context: ConfigureFailureContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context.failed_step, ConfigureStep::InstallDockerCompose);
        assert_eq!(
            context.error_kind,
            ConfigureErrorKind::CommandExecutionFailed
        );
    }
}
