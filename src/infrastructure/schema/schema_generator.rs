//! Schema Generator
//!
//! Provides JSON Schema generation from Rust types that implement `JsonSchema`.
//! This is a thin wrapper around the Schemars library.

use schemars::{schema_for, JsonSchema};
use thiserror::Error;

/// Errors that can occur during schema generation
#[derive(Debug, Error)]
pub enum SchemaGenerationError {
    /// Failed to serialize schema to JSON
    #[error("Failed to serialize schema to JSON")]
    SerializationFailed {
        /// The underlying serialization error
        #[source]
        source: serde_json::Error,
    },
}

impl SchemaGenerationError {
    /// Returns actionable help text for resolving this error
    ///
    /// Following the project's tiered help system pattern.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::SerializationFailed { .. } => {
                "Schema serialization failed. This is likely a bug in the schema generator.\n\
                 \n\
                 What to do:\n\
                 1. Check if the type has valid JsonSchema derives\n\
                 2. Report this as a bug if the error persists\n\
                 3. Include the full error message in your bug report"
                    .to_string()
            }
        }
    }
}

/// Schema generator for creating JSON Schemas from Rust types
///
/// This is a stateless utility that wraps the Schemars library,
/// providing a clean interface for schema generation.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::schema::SchemaGenerator;
/// use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
///
/// let schema_json = SchemaGenerator::generate::<EnvironmentCreationConfig>()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct SchemaGenerator;

impl SchemaGenerator {
    /// Generates a JSON Schema for the given type
    ///
    /// The type must implement `JsonSchema` from the Schemars library.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to generate a schema for (must implement `JsonSchema`)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The JSON Schema as a pretty-printed JSON string
    /// * `Err(SchemaGenerationError)` - If serialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::schema::SchemaGenerator;
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
    ///
    /// let schema = SchemaGenerator::generate::<EnvironmentCreationConfig>()?;
    /// assert!(schema.contains("\"$schema\""));
    /// assert!(schema.contains("\"environment\""));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `SchemaGenerationError::SerializationFailed` if the schema
    /// cannot be serialized to JSON (this should be extremely rare).
    pub fn generate<T: JsonSchema>() -> Result<String, SchemaGenerationError> {
        // Generate schema using Schemars
        let schema = schema_for!(T);

        // Serialize to pretty-printed JSON
        serde_json::to_string_pretty(&schema)
            .map_err(|source| SchemaGenerationError::SerializationFailed { source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    // Test helper struct
    #[derive(Serialize, Deserialize, JsonSchema)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn it_should_generate_valid_json_schema_when_given_valid_type() {
        let result = SchemaGenerator::generate::<TestConfig>();
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert!(schema.contains("\"$schema\""));
        assert!(schema.contains("\"properties\""));
    }

    #[test]
    fn it_should_include_type_properties_in_generated_schema() {
        let schema = SchemaGenerator::generate::<TestConfig>().unwrap();
        assert!(schema.contains("\"name\""));
        assert!(schema.contains("\"value\""));
    }

    #[test]
    fn it_should_generate_pretty_printed_json_output() {
        let schema = SchemaGenerator::generate::<TestConfig>().unwrap();
        // Pretty-printed JSON has newlines
        assert!(schema.contains('\n'));
        // And indentation
        assert!(schema.contains("  "));
    }

    #[test]
    fn it_should_provide_help_text_for_serialization_error() {
        let error = SchemaGenerationError::SerializationFailed {
            source: serde_json::Error::io(std::io::Error::other("test")),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("bug"));
    }
}
