//! Template Rendering Services
//!
//! This module contains application-layer services for rendering infrastructure
//! templates. Each service encapsulates the logic for rendering a specific type
//! of template (Ansible, `OpenTofu`, Docker Compose, etc.) and is designed to be
//! shared across multiple command handlers and steps.
//!
//! ## Architecture
//!
//! Rendering services follow the DDD application layer pattern:
//!
//! - **Orchestrate**: Bridge multiple domain types into infrastructure generator calls
//! - **Transform**: Map domain types to template-specific context types
//! - **Decide**: Apply conditional logic (e.g., "if Prometheus is configured, include it")
//!
//! ## Services
//!
//! - `AnsibleTemplateRenderingService` - Renders Ansible inventory and playbook templates
//! - `OpenTofuTemplateRenderingService` - Renders `OpenTofu` infrastructure templates
//! - `TrackerTemplateRenderingService` - Renders Tracker configuration templates
//! - `PrometheusTemplateRenderingService` - Renders Prometheus configuration templates
//! - `GrafanaTemplateRenderingService` - Renders Grafana provisioning templates
//! - `DockerComposeTemplateRenderingService` - Renders Docker Compose configuration templates
//! - `CaddyTemplateRenderingService` - Renders Caddy TLS proxy configuration templates
//! - `BackupTemplateRenderingService` - Renders backup configuration templates
//!
//! ## Design Principles
//!
//! All rendering services follow these principles:
//!
//! 1. **Explicit Inputs**: Services take explicit domain config types (e.g., `&TrackerConfig`)
//!    rather than `Environment<S>`. This makes dependencies clear and allows both the render
//!    command handler and the release steps to call them.
//!
//! 2. **Factory Pattern**: Services use `from_paths()` or `from_params()` factory methods
//!    that accept `templates_dir`, `build_dir`, and `clock` as construction parameters.
//!
//! 3. **Single Responsibility**: Each service handles exactly one template type and its
//!    associated context building logic.
//!
//! 4. **Error Wrapping**: Services define thin error types that wrap infrastructure
//!    generator errors while preserving context.
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::application::services::rendering::AnsibleTemplateRenderingService;
//! use torrust_tracker_deployer_lib::shared::clock::SystemClock;
//!
//! let service = AnsibleTemplateRenderingService::from_paths(
//!     templates_dir,
//!     build_dir,
//!     Arc::new(SystemClock),
//! );
//!
//! service.render_templates(&user_inputs, instance_ip, None).await?;
//! ```

mod ansible;
mod backup;
mod caddy;
mod docker_compose;
mod grafana;
mod opentofu;
mod prometheus;
mod tracker;

pub use ansible::{AnsibleTemplateRenderingService, AnsibleTemplateRenderingServiceError};
pub use backup::{BackupTemplateRenderingService, BackupTemplateRenderingServiceError};
pub use caddy::{CaddyTemplateRenderingService, CaddyTemplateRenderingServiceError};
pub use docker_compose::{
    DockerComposeTemplateRenderingService, DockerComposeTemplateRenderingServiceError,
};
pub use grafana::{GrafanaTemplateRenderingService, GrafanaTemplateRenderingServiceError};
pub use opentofu::{OpenTofuTemplateRenderingService, OpenTofuTemplateRenderingServiceError};
pub use prometheus::{PrometheusTemplateRenderingService, PrometheusTemplateRenderingServiceError};
pub use tracker::{TrackerTemplateRenderingService, TrackerTemplateRenderingServiceError};
