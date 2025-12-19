//! Prometheus Configuration DTO (Application Layer)
//!
//! This module contains the DTO type for Prometheus configuration used in
//! environment creation. This type uses raw primitives (u32) for JSON
//! deserialization and converts to the rich domain type (`PrometheusConfig`).

use std::num::NonZeroU32;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::prometheus::PrometheusConfig;

/// Prometheus configuration section (DTO)
///
/// This is a simple DTO that deserializes from JSON numbers and validates
/// when converting to the domain `PrometheusConfig`.
///
/// # Examples
///
/// ```json
/// {
///     "scrape_interval_in_secs": 15
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrometheusSection {
    /// Interval for Prometheus to scrape metrics from targets (in seconds)
    ///
    /// Must be greater than 0. The Prometheus template adds the 's' suffix.
    /// Examples: 15 (15 seconds), 30 (30 seconds), 60 (1 minute)
    pub scrape_interval_in_secs: u32,
}

impl Default for PrometheusSection {
    fn default() -> Self {
        Self {
            scrape_interval_in_secs: PrometheusConfig::default().scrape_interval_in_secs(),
        }
    }
}

impl PrometheusSection {
    /// Converts this DTO to a domain `PrometheusConfig`
    ///
    /// This method performs validation and type conversion from the
    /// u32 DTO to the strongly-typed domain model with `NonZeroU32`.
    ///
    /// # Errors
    ///
    /// Returns error if scrape interval is 0
    pub fn to_prometheus_config(&self) -> Result<PrometheusConfig, CreateConfigError> {
        let interval = NonZeroU32::new(self.scrape_interval_in_secs).ok_or_else(|| {
            CreateConfigError::InvalidPrometheusConfig(format!(
                "Scrape interval must be greater than 0, got: {}",
                self.scrape_interval_in_secs
            ))
        })?;
        Ok(PrometheusConfig::new(interval))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_default_values() {
        let section = PrometheusSection::default();
        assert_eq!(section.scrape_interval_in_secs, 15);
    }

    #[test]
    fn it_should_convert_to_prometheus_config() {
        let section = PrometheusSection {
            scrape_interval_in_secs: 30,
        };

        let config = section.to_prometheus_config().expect("Valid config");
        assert_eq!(config.scrape_interval_in_secs(), 30);
    }

    #[test]
    fn it_should_convert_default_section_to_default_config() {
        let section = PrometheusSection::default();
        let config = section.to_prometheus_config().expect("Valid config");

        assert_eq!(config, PrometheusConfig::default());
    }

    #[test]
    fn it_should_reject_zero_interval() {
        let section = PrometheusSection {
            scrape_interval_in_secs: 0,
        };

        let result = section.to_prometheus_config();
        assert!(result.is_err());
    }
}
