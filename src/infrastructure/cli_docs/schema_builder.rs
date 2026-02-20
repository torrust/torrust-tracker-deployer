//! CLI Documentation JSON Builder
//!
//! Utilities for building JSON documentation structures from Clap command metadata.

use clap::{Arg, Command};
use serde_json::{json, Map, Value};

/// Builds JSON object with application metadata
///
/// Extracts name, version, and description from the Clap command.
///
/// # Examples
///
/// ```rust,ignore
/// use clap::CommandFactory;
/// use torrust_tracker_deployer_lib::infrastructure::cli_docs::schema_builder;
///
/// let command = MyCli::command();
/// let metadata = schema_builder::build_app_metadata(&command);
/// assert!(metadata.contains_key("name"));
/// ```
#[must_use]
pub fn build_app_metadata(command: &Command) -> Map<String, Value> {
    let mut metadata = Map::new();

    metadata.insert("name".to_string(), json!(command.get_name()));

    if let Some(version) = command.get_version() {
        metadata.insert("version".to_string(), json!(version));
    }

    if let Some(about) = command.get_about() {
        metadata.insert("description".to_string(), json!(about.to_string()));
    }

    if let Some(long_about) = command.get_long_about() {
        metadata.insert(
            "long_description".to_string(),
            json!(long_about.to_string()),
        );
    }

    metadata
}

/// Builds JSON documentation for a single argument
///
/// Extracts all metadata from a Clap argument including flags, help text,
/// required status, and value names.
///
/// # Examples
///
/// ```rust,ignore
/// use clap::Arg;
/// use torrust_tracker_deployer_lib::infrastructure::cli_docs::schema_builder;
///
/// let arg = Arg::new("verbose").short('v').long("verbose");
/// let schema = schema_builder::build_argument_schema(&arg);
/// assert!(schema.contains_key("id"));
/// ```
#[must_use]
pub fn build_argument_schema(arg: &Arg) -> Map<String, Value> {
    let mut schema = Map::new();

    schema.insert("id".to_string(), json!(arg.get_id().as_str()));

    if let Some(short) = arg.get_short() {
        schema.insert("short".to_string(), json!(short.to_string()));
    }

    if let Some(long) = arg.get_long() {
        schema.insert("long".to_string(), json!(long));
    }

    if let Some(help) = arg.get_help() {
        schema.insert("help".to_string(), json!(help.to_string()));
    }

    if let Some(long_help) = arg.get_long_help() {
        schema.insert("long_help".to_string(), json!(long_help.to_string()));
    }

    schema.insert("required".to_string(), json!(arg.is_required_set()));

    if let Some(value_names) = arg.get_value_names() {
        let names: Vec<String> = value_names
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        schema.insert("value_names".to_string(), json!(names));
    }

    // Add action type if it's a flag (no value) vs option (takes value)
    let action_type = if arg.get_num_args().is_some_and(|n| n.max_values() == 0) {
        "flag"
    } else {
        "option"
    };
    schema.insert("type".to_string(), json!(action_type));

    schema
}

/// Builds JSON schema for a subcommand
///
/// Recursively extracts subcommand metadata including its own arguments
/// and any nested subcommands.
///
/// # Examples
///
/// ```rust,ignore
/// use clap::Command;
/// use torrust_tracker_deployer_lib::infrastructure::cli_docs::schema_builder;
///
/// let subcommand = Command::new("create").about("Create resource");
/// let schema = schema_builder::build_subcommand_schema(&subcommand);
/// assert!(schema.contains_key("name"));
/// ```
#[must_use]
pub fn build_subcommand_schema(subcommand: &Command) -> Map<String, Value> {
    let mut schema = Map::new();

    schema.insert("name".to_string(), json!(subcommand.get_name()));

    if let Some(about) = subcommand.get_about() {
        schema.insert("description".to_string(), json!(about.to_string()));
    }

    if let Some(long_about) = subcommand.get_long_about() {
        schema.insert(
            "long_description".to_string(),
            json!(long_about.to_string()),
        );
    }

    // Extract arguments (excluding built-in help/version)
    let arguments: Vec<Value> = subcommand
        .get_arguments()
        .filter(|arg| {
            let id = arg.get_id().as_str();
            id != "help" && id != "version"
        })
        .map(|arg| Value::Object(build_argument_schema(arg)))
        .collect();

    if !arguments.is_empty() {
        schema.insert("arguments".to_string(), json!(arguments));
    }

    // Recursively extract nested subcommands
    let nested_subcommands: Vec<Value> = subcommand
        .get_subcommands()
        .filter(|cmd| cmd.get_name() != "help")
        .map(|cmd| Value::Object(build_subcommand_schema(cmd)))
        .collect();

    if !nested_subcommands.is_empty() {
        schema.insert("subcommands".to_string(), json!(nested_subcommands));
    }

    schema
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    #[test]
    fn it_should_extract_app_metadata() {
        let command = Command::new("test-app")
            .version("1.0.0")
            .about("A test application");

        let metadata = build_app_metadata(&command);

        assert_eq!(metadata.get("name"), Some(&json!("test-app")));
        assert_eq!(metadata.get("version"), Some(&json!("1.0.0")));
        assert_eq!(
            metadata.get("description"),
            Some(&json!("A test application"))
        );
    }

    #[test]
    fn it_should_extract_argument_with_short_and_long_flags() {
        let arg = Arg::new("verbose")
            .short('v')
            .long("verbose")
            .help("Enable verbose output");

        let schema = build_argument_schema(&arg);

        assert_eq!(schema.get("id"), Some(&json!("verbose")));
        assert_eq!(schema.get("short"), Some(&json!("v")));
        assert_eq!(schema.get("long"), Some(&json!("verbose")));
        assert_eq!(schema.get("help"), Some(&json!("Enable verbose output")));
    }

    #[test]
    fn it_should_mark_required_arguments() {
        let required_arg = Arg::new("name").required(true);
        let optional_arg = Arg::new("description");

        let required_schema = build_argument_schema(&required_arg);
        let optional_schema = build_argument_schema(&optional_arg);

        assert_eq!(required_schema.get("required"), Some(&json!(true)));
        assert_eq!(optional_schema.get("required"), Some(&json!(false)));
    }

    #[test]
    fn it_should_extract_subcommand_with_arguments() {
        let subcommand = Command::new("create")
            .about("Create a resource")
            .arg(Arg::new("name").required(true).help("Resource name"));

        let schema = build_subcommand_schema(&subcommand);

        assert_eq!(schema.get("name"), Some(&json!("create")));
        assert_eq!(schema.get("description"), Some(&json!("Create a resource")));
        assert!(schema.contains_key("arguments"));
    }

    #[test]
    fn it_should_handle_nested_subcommands() {
        let nested = Command::new("template").about("Generate template");
        let parent = Command::new("create")
            .about("Create operations")
            .subcommand(nested);

        let schema = build_subcommand_schema(&parent);

        assert!(schema.contains_key("subcommands"));
        if let Some(Value::Array(subcommands)) = schema.get("subcommands") {
            assert_eq!(subcommands.len(), 1);
        } else {
            panic!("Expected subcommands array");
        }
    }

    #[test]
    fn it_should_extract_long_description_from_app_metadata() {
        let command = Command::new("test-app")
            .about("Short description")
            .long_about("Long description with multiple paragraphs.\n\nThis provides more detail.");

        let metadata = build_app_metadata(&command);

        assert_eq!(
            metadata.get("description"),
            Some(&json!("Short description"))
        );
        assert_eq!(
            metadata.get("long_description"),
            Some(&json!(
                "Long description with multiple paragraphs.\n\nThis provides more detail."
            ))
        );
    }

    #[test]
    fn it_should_extract_long_help_from_arguments() {
        let arg = Arg::new("config")
            .help("Path to config file")
            .long_help("Path to the configuration file.\n\nThis file should contain valid JSON.");

        let schema = build_argument_schema(&arg);

        assert_eq!(schema.get("help"), Some(&json!("Path to config file")));
        assert_eq!(
            schema.get("long_help"),
            Some(&json!(
                "Path to the configuration file.\n\nThis file should contain valid JSON."
            ))
        );
    }

    #[test]
    fn it_should_extract_long_description_from_subcommands() {
        let subcommand = Command::new("deploy")
            .about("Deploy application")
            .long_about("Deploy the application to the specified environment.\n\nThis will provision infrastructure and configure services.");

        let schema = build_subcommand_schema(&subcommand);

        assert_eq!(
            schema.get("description"),
            Some(&json!("Deploy application"))
        );
        assert_eq!(
            schema.get("long_description"),
            Some(&json!("Deploy the application to the specified environment.\n\nThis will provision infrastructure and configure services."))
        );
    }
}
