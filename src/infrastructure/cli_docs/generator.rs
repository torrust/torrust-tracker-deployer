//! CLI Documentation Generator
//!
//! Generates JSON documentation representation of CLI structure using Clap introspection.
//! This provides a machine-readable, versionable specification of the CLI interface.

use clap::{Command, CommandFactory};
use serde_json::{json, Value};

use super::errors::CliDocsGenerationError;
use super::schema_builder;

/// CLI documentation generator for creating JSON documentation from Clap CLI structures
///
/// This is a stateless utility that uses Clap's introspection APIs to extract
/// comprehensive CLI metadata and convert it to a structured JSON documentation format.
///
/// # Architecture
///
/// - Uses `CommandFactory` trait to access CLI structure
/// - Recursively traverses commands and subcommands
/// - Delegates JSON construction to `schema_builder` module
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use torrust_tracker_deployer_lib::infrastructure::cli_docs::CliDocsGenerator;
///
/// #[derive(Parser)]
/// struct MyCli {
///     #[arg(short, long)]
///     verbose: bool,
/// }
///
/// let docs = CliDocsGenerator::generate::<MyCli>()?;
/// assert!(docs.contains("\"name\""));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CliDocsGenerator;

impl CliDocsGenerator {
    /// Generates JSON documentation for the given CLI type
    ///
    /// The type must implement `CommandFactory` from Clap, which is automatically
    /// provided by the `#[derive(Parser)]` macro.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The CLI type to generate documentation for (must implement `CommandFactory`)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The JSON documentation as a pretty-printed JSON string
    /// * `Err(CliDocsGenerationError)` - If documentation generation or serialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use torrust_tracker_deployer_lib::infrastructure::cli_docs::CliDocsGenerator;
    ///
    /// #[derive(Parser)]
    /// #[command(name = "my-app", version = "1.0.0", about = "My application")]
    /// struct MyCli {
    ///     #[arg(short, long)]
    ///     verbose: bool,
    /// }
    ///
    /// let docs = CliDocsGenerator::generate::<MyCli>()?;
    /// assert!(docs.contains("my-app"));
    /// assert!(docs.contains("verbose"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CliDocsGenerationError::SerializationFailed` if the documentation
    /// cannot be serialized to JSON (this should be extremely rare).
    pub fn generate<T: CommandFactory>() -> Result<String, CliDocsGenerationError> {
        let command = T::command();
        let schema = Self::build_schema(&command);

        // Serialize to pretty-printed JSON
        serde_json::to_string_pretty(&schema)
            .map_err(|source| CliDocsGenerationError::SerializationFailed { source })
    }

    /// Builds the complete documentation structure from a Clap command
    ///
    /// Creates a JSON object with:
    /// - `format`: Documentation format identifier ("cli-documentation")
    /// - `format_version`: Format version ("1.0")
    /// - `cli`: Complete CLI metadata
    ///   - Application info (name, version, description)
    ///   - Global arguments
    ///   - Subcommands (recursively)
    ///
    /// # Arguments
    ///
    /// * `command` - The Clap command structure to introspect
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the complete documentation
    fn build_schema(command: &Command) -> Value {
        let mut cli = schema_builder::build_app_metadata(command);

        // Extract global arguments (excluding built-in help/version)
        let global_args: Vec<Value> = command
            .get_arguments()
            .filter(|arg| {
                let id = arg.get_id().as_str();
                id != "help" && id != "version"
            })
            .map(|arg| Value::Object(schema_builder::build_argument_schema(arg)))
            .collect();

        if !global_args.is_empty() {
            cli.insert("global_arguments".to_string(), json!(global_args));
        }

        // Extract subcommands (excluding built-in help)
        let subcommands: Vec<Value> = command
            .get_subcommands()
            .filter(|cmd| cmd.get_name() != "help")
            .map(|cmd| Value::Object(schema_builder::build_subcommand_schema(cmd)))
            .collect();

        if !subcommands.is_empty() {
            cli.insert("commands".to_string(), json!(subcommands));
        }

        // Build complete documentation with format metadata
        json!({
            "format": "cli-documentation",
            "format_version": "1.0",
            "cli": cli
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, Subcommand};

    // Test CLI structure
    #[derive(Parser)]
    #[command(name = "test-cli", version = "1.0.0", about = "A test CLI")]
    struct TestCli {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        #[command(subcommand)]
        command: Option<TestCommands>,
    }

    #[derive(Subcommand)]
    enum TestCommands {
        /// Create a resource
        Create {
            /// Resource name
            name: String,
        },
    }

    #[test]
    fn it_should_generate_valid_json_schema_when_given_valid_cli() {
        let result = CliDocsGenerator::generate::<TestCli>();
        assert!(result.is_ok());

        let docs = result.unwrap();
        assert!(docs.contains("\"format\""));
        assert!(docs.contains("cli-documentation"));
        assert!(docs.contains("test-cli"));
    }

    #[test]
    fn it_should_include_app_metadata_in_schema() {
        let schema = CliDocsGenerator::generate::<TestCli>().unwrap();
        assert!(schema.contains("\"name\""));
        assert!(schema.contains("\"version\""));
        assert!(schema.contains("\"description\""));
        assert!(schema.contains("test-cli"));
        assert!(schema.contains("1.0.0"));
    }

    #[test]
    fn it_should_include_global_arguments_in_schema() {
        let schema = CliDocsGenerator::generate::<TestCli>().unwrap();
        assert!(schema.contains("global_arguments"));
        assert!(schema.contains("verbose"));
    }

    #[test]
    fn it_should_include_subcommands_in_schema() {
        let schema = CliDocsGenerator::generate::<TestCli>().unwrap();
        assert!(schema.contains("commands"));
        assert!(schema.contains("Create"));
    }

    #[test]
    fn it_should_generate_pretty_printed_json_output() {
        let schema = CliDocsGenerator::generate::<TestCli>().unwrap();
        // Pretty-printed JSON has newlines
        assert!(schema.contains('\n'));
        // And indentation
        assert!(schema.contains("  "));
    }

    #[test]
    fn it_should_include_json_schema_version() {
        let docs = CliDocsGenerator::generate::<TestCli>().unwrap();
        assert!(docs.contains("\"format\""));
        assert!(docs.contains("cli-documentation"));
        assert!(docs.contains("\"format_version\""));
        assert!(docs.contains("1.0"));
    }
}
