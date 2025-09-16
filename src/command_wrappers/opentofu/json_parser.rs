//! JSON parsing utilities for `OpenTofu` command output
//!
//! This module provides the `OpenTofuJsonParser` which handles parsing of complex JSON
//! responses from `OpenTofu` commands and converts them into structured Rust types.
//!
//! ## Key Features
//!
//! - Parsing `OpenTofu` output command JSON into instance information
//! - IP address extraction from Terraform state outputs
//! - Error handling for malformed or unexpected JSON structures
//! - Type-safe conversion from JSON to Rust structs
//! - Support for complex nested JSON structures from Terraform state
//!
//! The parser encapsulates all JSON handling logic and provides a clean interface
//! for converting `OpenTofu` command output into usable data structures.

use std::net::IpAddr;
use std::str::FromStr;

use serde_json::Value;
use thiserror::Error;

use super::client::InstanceInfo;

/// Errors that can occur during `OpenTofu` JSON parsing
#[derive(Error, Debug)]
pub enum ParseError {
    /// JSON deserialization failed
    #[error("Failed to parse JSON: {message}")]
    JsonError { message: String },

    /// Required field is missing or has wrong type
    #[error("Field error: {message}")]
    FieldError { message: String },
}

/// A JSON parser for `OpenTofu` command outputs.
///
/// This parser handles the complex JSON structure returned by `OpenTofu` commands
/// and converts them into structured Rust types. It encapsulates all the
/// JSON parsing logic and can be unit tested independently.
pub(crate) struct OpenTofuJsonParser;

impl OpenTofuJsonParser {
    /// Parse `instance_info` from `OpenTofu` JSON output
    ///
    /// # Arguments
    ///
    /// * `json_output` - JSON string from `tofu output -json` command
    ///
    /// # Returns
    ///
    /// * `Ok(InstanceInfo)` - Parsed instance information
    /// * `Err(ParseError)` - Parsing error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The JSON cannot be parsed
    /// * The `instance_info` section is missing
    /// * Required fields are missing or have wrong types
    pub fn parse_instance_info(json_output: &str) -> Result<InstanceInfo, ParseError> {
        let outputs: Value =
            serde_json::from_str(json_output).map_err(|e| ParseError::JsonError {
                message: format!("Failed to parse OpenTofu output as JSON: {e}"),
            })?;

        let instance_info_value = outputs
            .get("instance_info")
            .and_then(|v| v.get("value"))
            .ok_or_else(|| ParseError::FieldError {
                message: "instance_info section not found in OpenTofu outputs".to_string(),
            })?;

        let image = instance_info_value
            .get("image")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::FieldError {
                message: "image field missing or not a string".to_string(),
            })?
            .to_string();

        let ip_address_str = instance_info_value
            .get("ip_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::FieldError {
                message: "ip_address field missing or not a string".to_string(),
            })?;

        let ip_address = IpAddr::from_str(ip_address_str).map_err(|e| ParseError::FieldError {
            message: format!("ip_address field is not a valid IP address: {e}"),
        })?;

        let name = instance_info_value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::FieldError {
                message: "name field missing or not a string".to_string(),
            })?
            .to_string();

        let status = instance_info_value
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::FieldError {
                message: "status field missing or not a string".to_string(),
            })?
            .to_string();

        Ok(InstanceInfo {
            image,
            ip_address,
            name,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_parse_instance_info_from_valid_json() {
        let json_output = r#"{
            "instance_info": {
                "value": {
                    "image": "ubuntu:24.04",
                    "ip_address": "10.140.190.68",
                    "name": "torrust-vm",
                    "status": "Running"
                }
            }
        }"#;

        let result = OpenTofuJsonParser::parse_instance_info(json_output).unwrap();

        assert_eq!(result.image, "ubuntu:24.04");
        assert_eq!(
            result.ip_address,
            IpAddr::from_str("10.140.190.68").unwrap()
        );
        assert_eq!(result.name, "torrust-vm");
        assert_eq!(result.status, "Running");
    }

    #[test]
    fn it_should_fail_with_invalid_json() {
        let invalid_json = "not valid json";

        let result = OpenTofuJsonParser::parse_instance_info(invalid_json);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::JsonError { .. }));
    }

    #[test]
    fn it_should_fail_when_instance_info_section_missing() {
        let json_output = r#"{
            "other_output": {
                "value": "some value"
            }
        }"#;

        let result = OpenTofuJsonParser::parse_instance_info(json_output);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ParseError::FieldError { .. }));
        assert!(error
            .to_string()
            .contains("instance_info section not found"));
    }

    #[test]
    fn it_should_fail_when_required_field_missing() {
        let json_output = r#"{
            "instance_info": {
                "value": {
                    "image": "ubuntu:24.04",
                    "ip_address": "10.140.190.68",
                    "name": "torrust-vm"
                }
            }
        }"#;

        let result = OpenTofuJsonParser::parse_instance_info(json_output);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ParseError::FieldError { .. }));
        assert!(error.to_string().contains("status field missing"));
    }

    #[test]
    fn it_should_fail_when_field_has_wrong_type() {
        let json_output = r#"{
            "instance_info": {
                "value": {
                    "image": 123,
                    "ip_address": "10.140.190.68",
                    "name": "torrust-vm",
                    "status": "Running"
                }
            }
        }"#;

        let result = OpenTofuJsonParser::parse_instance_info(json_output);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ParseError::FieldError { .. }));
        assert!(error
            .to_string()
            .contains("image field missing or not a string"));
    }

    #[test]
    fn it_should_fail_when_ip_address_is_invalid() {
        let json_output = r#"{
            "instance_info": {
                "value": {
                    "image": "ubuntu:24.04",
                    "ip_address": "invalid-ip-address",
                    "name": "torrust-vm",
                    "status": "Running"
                }
            }
        }"#;

        let result = OpenTofuJsonParser::parse_instance_info(json_output);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ParseError::FieldError { .. }));
        assert!(error
            .to_string()
            .contains("ip_address field is not a valid IP address"));
    }
}
