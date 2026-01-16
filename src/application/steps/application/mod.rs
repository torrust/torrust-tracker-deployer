//! Application deployment and lifecycle steps
//!
//! This module contains steps that manage application deployment and lifecycle
//! operations. These steps handle application-specific operations like deployment,
//! service management, configuration, and application health monitoring.
//!
//! ## Available Steps
//!
//! - `create_tracker_storage` - Creates tracker storage directory structure on remote host
//! - `init_tracker_database` - Initializes `SQLite` database file for the tracker
//! - `deploy_tracker_config` - Deploys tracker.toml configuration file to remote host
//! - `create_prometheus_storage` - Creates Prometheus storage directory structure on remote host
//! - `deploy_prometheus_config` - Deploys prometheus.yml configuration file to remote host
//! - `deploy_grafana_provisioning` - Deploys Grafana provisioning files (datasources/dashboards) to remote host
//! - `deploy_compose_files` - Deploys Docker Compose files to remote host via Ansible
//! - `start_services` - Starts Docker Compose services via Ansible
//! - `run` - Legacy run step (placeholder)
//!
//! ## Future Steps
//!
//! This module is prepared for future application deployment steps such as:
//! - Application health checks and validation
//! - Service stop and restart operations
//! - Status monitoring and reporting
//!
//! ## Integration
//!
//! Application steps integrate with the existing infrastructure and
//! software installation steps to provide complete deployment workflows
//! from infrastructure provisioning to application operation.

pub mod create_prometheus_storage;
pub mod create_tracker_storage;
pub mod deploy_caddy_config;
pub mod deploy_compose_files;
pub mod deploy_grafana_provisioning;
pub mod deploy_prometheus_config;
pub mod deploy_tracker_config;
pub mod init_tracker_database;
pub mod run;
pub mod start_services;

pub use create_prometheus_storage::CreatePrometheusStorageStep;
pub use create_tracker_storage::CreateTrackerStorageStep;
pub use deploy_caddy_config::DeployCaddyConfigStep;
pub use deploy_compose_files::{DeployComposeFilesStep, DeployComposeFilesStepError};
pub use deploy_grafana_provisioning::DeployGrafanaProvisioningStep;
pub use deploy_prometheus_config::DeployPrometheusConfigStep;
pub use deploy_tracker_config::{DeployTrackerConfigStep, DeployTrackerConfigStepError};
pub use init_tracker_database::InitTrackerDatabaseStep;
pub use run::{RunStep, RunStepError};
pub use start_services::{StartServicesStep, StartServicesStepError};
