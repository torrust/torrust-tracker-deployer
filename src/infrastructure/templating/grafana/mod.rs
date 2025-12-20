//! Grafana Configuration Management Integration
//!
//! This module provides template rendering for Grafana provisioning configuration,
//! enabling automatic datasource and dashboard configuration.
//!
//! ## Architecture
//!
//! Follows the Project Generator pattern:
//! - `template` - Template renderers for Grafana provisioning files
//!
//! ## Configuration Files Generated
//!
//! - **Datasources**: `datasources/prometheus.yml` - Auto-configures Prometheus as data source
//! - **Dashboards**: Dashboard provider and JSON files for metrics visualization

pub mod template;
