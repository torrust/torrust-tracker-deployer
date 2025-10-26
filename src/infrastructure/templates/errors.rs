//! Template Error Types
//!
//! This module provides structured error types for template operations with
//! comprehensive error context and actionable troubleshooting guidance.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during template operations
///
/// These errors represent infrastructure-level failures in template
/// handling and provide structured context for troubleshooting.
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Template was not found in embedded resources
    #[error("Template not found: {template_type}")]
    TemplateNotFound { template_type: String },

    /// Requested template type is not supported
    #[error("Unsupported template type: {requested_type}")]
    UnsupportedTemplateType {
        requested_type: String,
        supported_types: Vec<String>,
    },

    /// Output path is invalid or unsuitable for template generation
    #[error("Invalid output path: {path} - {reason}")]
    InvalidOutputPath { path: PathBuf, reason: String },

    /// Failed to create directory for template file
    #[error("Failed to create directory: {path}")]
    DirectoryCreationFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write template file
    #[error("Failed to write template file: {path}")]
    FileWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Template validation failed (indicates a bug in embedded templates)
    #[error("Template validation failed: {template_type}")]
    TemplateValidationFailed {
        template_type: String,
        #[source]
        source: serde_json::Error,
    },
}

impl TemplateError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// Returns multi-line help text with specific steps to resolve the error.
    /// This follows the project's tiered help system pattern.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::TemplateNotFound { .. } => {
                "Template Not Found - Detailed Troubleshooting:

1. Check if the template type is supported
2. Verify the application binary includes embedded templates
3. Try regenerating templates if they should be available
4. Report issue if template should be available but is missing

For more information, see the template documentation."
            }

            Self::UnsupportedTemplateType { .. } => {
                "Unsupported Template Type - Detailed Troubleshooting:

1. Use 'json' for JSON templates (currently supported)
2. TOML support will be added in a future release
3. Verify you are using the correct template type format

For more information, see the template format documentation."
            }

            Self::InvalidOutputPath { .. } => {
                "Invalid Output Path - Detailed Troubleshooting:

1. Ensure the path points to a file (not directory)
2. Use correct file extension (.json for JSON templates)
3. Verify parent directory exists or can be created
4. Check write permissions for the target location

For more information, see the file system documentation."
            }

            Self::DirectoryCreationFailed { .. } => {
                "Directory Creation Failed - Detailed Troubleshooting:

1. Check write permissions for the parent directory
2. Verify disk space is available: df -h
3. Ensure no file exists with the same name as the directory
4. Check path length limits on your system

For more information, see the filesystem troubleshooting guide."
            }

            Self::FileWriteFailed { .. } => {
                "Template File Write Failed - Detailed Troubleshooting:

1. Check write permissions for the target file and directory
2. Verify disk space is available: df -h
3. Ensure the file is not open in another application
4. Check if antivirus software is blocking file creation

For more information, see the file operations documentation."
            }

            Self::TemplateValidationFailed { .. } => {
                "Template Validation Failed - Detailed Troubleshooting:

1. This indicates a bug in the embedded templates
2. Report this issue with full error details
3. Use --generate-template to create a fresh template
4. Check for application updates

This is likely a software bug that needs to be reported."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_template_not_found() {
        let error = TemplateError::TemplateNotFound {
            template_type: "YAML".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Template Not Found"));
        assert!(help.contains("Check if the template type is supported"));
    }

    #[test]
    fn it_should_provide_help_for_unsupported_template_type() {
        let error = TemplateError::UnsupportedTemplateType {
            requested_type: "xml".to_string(),
            supported_types: vec!["json".to_string()],
        };

        let help = error.help();
        assert!(help.contains("Unsupported Template Type"));
        assert!(help.contains("Use 'json' for JSON templates"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_output_path() {
        let error = TemplateError::InvalidOutputPath {
            path: PathBuf::from("/tmp/test"),
            reason: "Path is a directory".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Invalid Output Path"));
        assert!(help.contains("Ensure the path points to a file"));
    }

    #[test]
    fn it_should_provide_help_for_directory_creation_failed() {
        let error = TemplateError::DirectoryCreationFailed {
            path: PathBuf::from("/tmp/test"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        let help = error.help();
        assert!(help.contains("Directory Creation Failed"));
        assert!(help.contains("Check write permissions"));
    }

    #[test]
    fn it_should_provide_help_for_file_write_failed() {
        let error = TemplateError::FileWriteFailed {
            path: PathBuf::from("/tmp/test.json"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        let help = error.help();
        assert!(help.contains("Template File Write Failed"));
        assert!(help.contains("Check write permissions"));
    }

    #[test]
    fn it_should_provide_help_for_template_validation_failed() {
        let error = TemplateError::TemplateValidationFailed {
            template_type: "JSON".to_string(),
            source: serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
        };

        let help = error.help();
        assert!(help.contains("Template Validation Failed"));
        assert!(help.contains("This indicates a bug"));
    }

    #[test]
    fn it_should_format_error_messages_correctly() {
        let error = TemplateError::TemplateNotFound {
            template_type: "YAML".to_string(),
        };
        assert_eq!(error.to_string(), "Template not found: YAML");

        let error = TemplateError::UnsupportedTemplateType {
            requested_type: "xml".to_string(),
            supported_types: vec!["json".to_string()],
        };
        assert_eq!(error.to_string(), "Unsupported template type: xml");
    }

    #[test]
    fn it_should_preserve_source_errors() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test error");
        let error = TemplateError::FileWriteFailed {
            path: PathBuf::from("/tmp/test.json"),
            source: io_error,
        };

        // Verify source error is accessible
        let source = std::error::Error::source(&error);
        assert!(source.is_some());
        assert_eq!(source.unwrap().to_string(), "test error");
    }
}
