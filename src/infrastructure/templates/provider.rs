//! Template Provider Implementation
//!
//! This module provides the high-level API for template generation with
//! async file operations and comprehensive error handling.

use std::path::{Path, PathBuf};

use super::embedded::EmbeddedTemplates;
use super::errors::TemplateError;

/// Provider for configuration templates
///
/// Handles template retrieval from embedded resources and generation
/// of template files on the filesystem.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templates::{TemplateProvider, TemplateType};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = TemplateProvider::new();
///
/// // Generate template at specific path
/// provider.generate_template(
///     TemplateType::Json,
///     Path::new("./config.json")
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct TemplateProvider {
    embedded: EmbeddedTemplates,
}

impl TemplateProvider {
    /// Create a new template provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            embedded: EmbeddedTemplates::new(),
        }
    }

    /// Generate a template file at the specified path
    ///
    /// This method creates a configuration template file with placeholder values
    /// that users can fill in. It handles directory creation and validates the
    /// output path.
    ///
    /// # Arguments
    ///
    /// * `template_type` - Type of template to generate (currently only JSON)
    /// * `output_path` - Path where the template file should be created
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Template generated successfully
    /// * `Err(TemplateError)` - Template generation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template type is not supported
    /// - Output path is invalid (wrong extension, is a directory, etc.)
    /// - Parent directory cannot be created
    /// - File cannot be written due to permissions or I/O errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templates::{TemplateProvider, TemplateType};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = TemplateProvider::new();
    /// provider.generate_template(
    ///     TemplateType::Json,
    ///     Path::new("./environment-template.json")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_template(
        &self,
        template_type: TemplateType,
        output_path: &Path,
    ) -> Result<(), TemplateError> {
        // Get template content from embedded resources
        let template_content = self.embedded.get_template(template_type).ok_or_else(|| {
            TemplateError::TemplateNotFound {
                template_type: template_type.to_string(),
            }
        })?;

        // Validate output path
        Self::validate_output_path(output_path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|source| {
                TemplateError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        // Write template to file
        tokio::fs::write(output_path, template_content)
            .await
            .map_err(|source| TemplateError::FileWriteFailed {
                path: output_path.to_path_buf(),
                source,
            })?;

        Ok(())
    }

    /// Generate template with default filename in specified directory
    ///
    /// This is a convenience method that generates a template using the default
    /// filename for the template type in the specified directory.
    ///
    /// # Arguments
    ///
    /// * `template_type` - Type of template to generate
    /// * `directory` - Directory where template should be created
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the generated template file
    /// * `Err(TemplateError)` - Template generation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template type is not supported
    /// - Directory cannot be created
    /// - File cannot be written due to permissions or I/O errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templates::{TemplateProvider, TemplateType};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = TemplateProvider::new();
    /// let path = provider.generate_template_in_directory(
    ///     TemplateType::Json,
    ///     Path::new("./configs")
    /// ).await?;
    /// println!("Template generated at: {}", path.display());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_template_in_directory(
        &self,
        template_type: TemplateType,
        directory: &Path,
    ) -> Result<PathBuf, TemplateError> {
        let filename = template_type.default_filename();
        let output_path = directory.join(filename);

        self.generate_template(template_type, &output_path).await?;

        Ok(output_path)
    }

    /// Get template content as string without writing to file
    ///
    /// Useful for testing and programmatic access to templates.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::TemplateNotFound` if the template type is not supported.
    pub fn get_template_content(&self, template_type: TemplateType) -> Result<&str, TemplateError> {
        self.embedded
            .get_template(template_type)
            .ok_or_else(|| TemplateError::TemplateNotFound {
                template_type: template_type.to_string(),
            })
    }

    /// List all available template types
    #[must_use]
    pub fn available_templates(&self) -> Vec<TemplateType> {
        self.embedded.available_templates()
    }

    /// Validate that the output path is suitable for template generation
    fn validate_output_path(path: &Path) -> Result<(), TemplateError> {
        // Check if path already exists and is not a file
        if path.exists() && !path.is_file() {
            return Err(TemplateError::InvalidOutputPath {
                path: path.to_path_buf(),
                reason: "Path exists but is not a file".to_string(),
            });
        }

        // Validate file extension matches template type (JSON)
        if let Some(extension) = path.extension() {
            if extension != "json" {
                return Err(TemplateError::InvalidOutputPath {
                    path: path.to_path_buf(),
                    reason: format!(
                        "File extension '{}' does not match JSON template type",
                        extension.to_string_lossy()
                    ),
                });
            }
        } else {
            return Err(TemplateError::InvalidOutputPath {
                path: path.to_path_buf(),
                reason: "No file extension specified".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for TemplateProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported template types
///
/// Currently only JSON is supported, with plans to add TOML and YAML in the future.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    /// JSON configuration template
    Json,
    // Future: Toml, Yaml
}

impl TemplateType {
    /// Get the default filename for this template type
    #[must_use]
    pub const fn default_filename(&self) -> &'static str {
        match self {
            Self::Json => "environment-template.json",
        }
    }

    /// Get the file extension for this template type
    #[must_use]
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = TemplateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            _ => Err(TemplateError::UnsupportedTemplateType {
                requested_type: s.to_string(),
                supported_types: vec!["json".to_string()],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn it_should_generate_template_at_specified_path() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config.json");

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &output_path)
            .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify content is valid JSON
        let content = std::fs::read_to_string(&output_path).unwrap();
        let _value: serde_json::Value = serde_json::from_str(&content).unwrap();
    }

    #[tokio::test]
    async fn it_should_generate_template_in_directory() {
        let temp_dir = TempDir::new().unwrap();

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template_in_directory(TemplateType::Json, temp_dir.path())
            .await;

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.exists());
        assert_eq!(
            output_path.file_name().unwrap(),
            "environment-template.json"
        );
    }

    #[tokio::test]
    async fn it_should_create_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("configs")
            .join("env")
            .join("test.json");

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &nested_path)
            .await;

        assert!(result.is_ok());
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn it_should_fail_with_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config.txt");

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &output_path)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateError::InvalidOutputPath { reason, .. } => {
                assert!(reason.contains("does not match JSON"));
            }
            other => panic!("Expected InvalidOutputPath error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_fail_with_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config");

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &output_path)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateError::InvalidOutputPath { reason, .. } => {
                assert!(reason.contains("No file extension"));
            }
            other => panic!("Expected InvalidOutputPath error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_fail_if_path_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        std::fs::create_dir(&dir_path).unwrap();

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &dir_path)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateError::InvalidOutputPath { reason, .. } => {
                assert!(reason.contains("not a file"));
            }
            other => panic!("Expected InvalidOutputPath error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config.json");

        // Create initial file
        std::fs::write(&output_path, "old content").unwrap();

        let provider = TemplateProvider::new();
        let result = provider
            .generate_template(TemplateType::Json, &output_path)
            .await;

        assert!(result.is_ok());

        // Verify content was replaced
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn it_should_get_template_content() {
        let provider = TemplateProvider::new();
        let result = provider.get_template_content(TemplateType::Json);

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[test]
    fn it_should_list_available_templates() {
        let provider = TemplateProvider::new();
        let templates = provider.available_templates();

        assert_eq!(templates.len(), 1);
        assert!(templates.contains(&TemplateType::Json));
    }

    #[test]
    fn it_should_create_via_default_trait() {
        let provider = TemplateProvider::default();
        let templates = provider.available_templates();
        assert!(!templates.is_empty());
    }

    // TemplateType tests
    #[test]
    fn it_should_have_correct_default_filename() {
        assert_eq!(
            TemplateType::Json.default_filename(),
            "environment-template.json"
        );
    }

    #[test]
    fn it_should_have_correct_file_extension() {
        assert_eq!(TemplateType::Json.file_extension(), "json");
    }

    #[test]
    fn it_should_display_template_type() {
        assert_eq!(TemplateType::Json.to_string(), "JSON");
    }

    #[test]
    fn it_should_parse_from_string() {
        assert_eq!("json".parse::<TemplateType>().unwrap(), TemplateType::Json);
        assert_eq!("JSON".parse::<TemplateType>().unwrap(), TemplateType::Json);
        assert_eq!("Json".parse::<TemplateType>().unwrap(), TemplateType::Json);
    }

    #[test]
    fn it_should_fail_parsing_unsupported_type() {
        let result = "yaml".parse::<TemplateType>();
        assert!(result.is_err());

        match result.unwrap_err() {
            TemplateError::UnsupportedTemplateType {
                requested_type,
                supported_types,
            } => {
                assert_eq!(requested_type, "yaml");
                assert_eq!(supported_types, vec!["json"]);
            }
            other => panic!("Expected UnsupportedTemplateType error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_be_copy_and_clone() {
        let t1 = TemplateType::Json;
        let t2 = t1; // Copy
        let t3 = t1; // Also copy (not clone)

        assert_eq!(t1, t2);
        assert_eq!(t1, t3);
    }
}
