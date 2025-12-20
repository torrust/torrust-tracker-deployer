//! Create Schema Command Handler
//!
//! Handles the `create schema` command which generates JSON Schema from
//! the `EnvironmentCreationConfig` type.

use std::path::PathBuf;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::infrastructure::schema::SchemaGenerator;

use super::errors::CreateSchemaCommandHandlerError;

/// Handler for creating JSON Schema from configuration types
///
/// This handler orchestrates schema generation and output handling,
/// supporting both stdout and file output.
///
/// # Architecture
///
/// - No Step layer needed (single operation)
/// - Delegates schema generation to `SchemaGenerator` (Infrastructure)
/// - Handles output routing (stdout vs file)
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::create::schema::CreateSchemaCommandHandler;
/// use std::path::PathBuf;
/// use tempfile::TempDir;
///
/// // Generate to stdout
/// let schema = CreateSchemaCommandHandler::execute(None)?;
/// println!("{}", schema);
///
/// // Generate to file (use temp directory to avoid leaving artifacts)
/// let temp_dir = TempDir::new()?;
/// let schema_path = temp_dir.path().join("schema.json");
/// CreateSchemaCommandHandler::execute(Some(schema_path))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CreateSchemaCommandHandler;

impl CreateSchemaCommandHandler {
    /// Executes the create schema command
    ///
    /// Generates JSON Schema for `EnvironmentCreationConfig` and either writes
    /// it to a file or returns it as a string.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Optional path to write schema file. If `None`, returns schema as string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated schema (for stdout output or testing)
    /// * `Err(CreateSchemaCommandHandlerError)` - If generation or file writing fails
    ///
    /// # Output Behavior
    ///
    /// - **No path**: Returns schema string (caller handles stdout)
    /// - **With path**: Writes to file and returns the schema string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::schema::CreateSchemaCommandHandler;
    /// use std::path::PathBuf;
    /// use tempfile::TempDir;
    ///
    /// // Output to stdout (caller prints the returned string)
    /// let schema = CreateSchemaCommandHandler::execute(None)?;
    /// println!("{}", schema);
    ///
    /// // Output to file (use temp directory to avoid leaving artifacts)
    /// let temp_dir = TempDir::new()?;
    /// let schema_path = temp_dir.path().join("schema.json");
    /// let schema = CreateSchemaCommandHandler::execute(Some(schema_path))?;
    /// // File is written, schema string also returned for confirmation
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Schema generation fails (infrastructure error)
    /// - Parent directory creation fails (when writing to file)
    /// - File write fails (when writing to file)
    pub fn execute(
        output_path: Option<PathBuf>,
    ) -> Result<String, CreateSchemaCommandHandlerError> {
        // Generate schema using infrastructure layer
        let schema = SchemaGenerator::generate::<EnvironmentCreationConfig>()
            .map_err(|source| CreateSchemaCommandHandlerError::SchemaGenerationFailed { source })?;

        // If output path provided, write to file
        if let Some(path) = output_path {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|source| {
                    CreateSchemaCommandHandlerError::DirectoryCreationFailed {
                        path: parent.to_path_buf(),
                        source,
                    }
                })?;
            }

            // Write schema to file
            std::fs::write(&path, &schema).map_err(|source| {
                CreateSchemaCommandHandlerError::FileWriteFailed { path, source }
            })?;
        }

        // Return schema (for stdout or testing)
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_generate_schema_when_no_output_path_provided() {
        let result = CreateSchemaCommandHandler::execute(None);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert!(schema.contains("\"$schema\""));
        assert!(schema.contains("\"environment\""));
        assert!(schema.contains("\"ssh_credentials\""));
        assert!(schema.contains("\"provider\""));
        assert!(schema.contains("\"tracker\""));
    }

    #[test]
    fn it_should_write_schema_to_file_when_output_path_provided() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.json");

        let result = CreateSchemaCommandHandler::execute(Some(schema_path.clone()));
        assert!(result.is_ok());

        // Verify file was created
        assert!(schema_path.exists());

        // Verify file content is valid JSON schema
        let content = std::fs::read_to_string(&schema_path).unwrap();
        assert!(content.contains("\"$schema\""));
        assert!(content.contains("\"environment\""));
    }

    #[test]
    fn it_should_create_parent_directories_when_writing_to_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("configs")
            .join("schemas")
            .join("env.json");

        let result = CreateSchemaCommandHandler::execute(Some(nested_path.clone()));
        assert!(result.is_ok());

        // Verify nested directories were created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn it_should_return_schema_string_when_writing_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.json");

        let schema = CreateSchemaCommandHandler::execute(Some(schema_path)).unwrap();

        // Verify returned string is the schema
        assert!(schema.contains("\"$schema\""));
        assert!(schema.contains("\"environment\""));
    }

    #[test]
    fn it_should_generate_valid_json_schema_format() {
        let schema = CreateSchemaCommandHandler::execute(None).unwrap();

        // Parse as JSON to verify it's valid
        let json: serde_json::Value = serde_json::from_str(&schema).unwrap();

        // Verify it's a JSON Schema
        assert!(json.get("$schema").is_some());
        assert!(json.get("type").is_some());
        assert!(json.get("properties").is_some());
    }

    #[test]
    fn it_should_include_all_config_sections_in_schema() {
        let schema = CreateSchemaCommandHandler::execute(None).unwrap();
        let json: serde_json::Value = serde_json::from_str(&schema).unwrap();

        let properties = json.get("properties").unwrap().as_object().unwrap();

        // Verify all main sections are present
        assert!(properties.contains_key("environment"));
        assert!(properties.contains_key("ssh_credentials"));
        assert!(properties.contains_key("provider"));
        assert!(properties.contains_key("tracker"));
    }

    #[test]
    fn it_should_overwrite_existing_file_when_path_exists() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.json");

        // Create initial file
        std::fs::write(&schema_path, "old content").unwrap();

        // Generate schema to same path
        let result = CreateSchemaCommandHandler::execute(Some(schema_path.clone()));
        assert!(result.is_ok());

        // Verify content was replaced
        let content = std::fs::read_to_string(&schema_path).unwrap();
        assert!(content.contains("\"$schema\""));
        assert!(!content.contains("old content"));
    }
}
