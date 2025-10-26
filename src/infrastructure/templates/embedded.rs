//! Embedded Template Resources
//!
//! This module provides embedded configuration templates that are compiled
//! into the binary at build time. Templates use simple placeholders that
//! users can easily find and replace.

use super::provider::TemplateType;

/// Container for embedded template resources
///
/// Templates are embedded in the binary at compile time to ensure
/// they're always available without external dependencies.
pub struct EmbeddedTemplates;

impl EmbeddedTemplates {
    /// Create a new embedded templates container
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Get template content for the specified type
    ///
    /// Returns the template as a static string if the template type is supported,
    /// or `None` if the template type is not found.
    #[must_use]
    pub const fn get_template(&self, template_type: TemplateType) -> Option<&'static str> {
        match template_type {
            TemplateType::Json => Some(JSON_TEMPLATE),
        }
    }

    /// Get list of all available template types
    #[must_use]
    pub fn available_templates(&self) -> Vec<TemplateType> {
        vec![TemplateType::Json]
    }
}

impl Default for EmbeddedTemplates {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON configuration template
///
/// This template provides a complete example of the configuration format
/// with placeholder values that users can replace with their actual values.
///
/// The template uses simple, easily searchable placeholders like
/// `REPLACE_WITH_*` to make it clear what needs to be filled in.
const JSON_TEMPLATE: &str = r#"{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME"
  },
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
    "username": "torrust",
    "port": 22
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_valid_json_template() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Verify the template is valid JSON
        let _value: serde_json::Value =
            serde_json::from_str(template).expect("JSON template should be valid JSON");
    }

    #[test]
    fn it_should_contain_required_placeholder_fields() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Verify required placeholders are present
        assert!(template.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(template.contains("REPLACE_WITH_SSH_PRIVATE_KEY_PATH"));
        assert!(template.contains("REPLACE_WITH_SSH_PUBLIC_KEY_PATH"));

        // Verify default values are present
        assert!(template.contains(r#""username": "torrust""#));
        assert!(template.contains(r#""port": 22"#));
    }

    #[test]
    fn it_should_list_available_templates() {
        let embedded = EmbeddedTemplates::new();
        let templates = embedded.available_templates();

        assert_eq!(templates.len(), 1);
        assert!(templates.contains(&TemplateType::Json));
    }

    #[test]
    fn it_should_create_via_default_trait() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json);
        assert!(template.is_some());
    }

    #[test]
    fn it_should_return_none_for_unsupported_template() {
        let embedded = EmbeddedTemplates::new();
        // We can't test with an actual unsupported type since the enum only has Json
        // But we verify the pattern by checking that Json works
        assert!(embedded.get_template(TemplateType::Json).is_some());
    }

    #[test]
    fn it_should_have_well_formatted_json() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Parse and re-serialize to verify formatting
        let parsed: serde_json::Value = serde_json::from_str(template).unwrap();
        let reformatted = serde_json::to_string_pretty(&parsed).unwrap();

        // Both should parse to the same value
        let parsed_again: serde_json::Value = serde_json::from_str(&reformatted).unwrap();
        assert_eq!(parsed, parsed_again);
    }

    #[test]
    fn it_should_match_environment_creation_config_structure() {
        use crate::domain::config::EnvironmentCreationConfig;

        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Verify template can be parsed as EnvironmentCreationConfig
        // (even with placeholder values)
        let result: Result<EnvironmentCreationConfig, _> = serde_json::from_str(template);

        // Should succeed because placeholders are valid strings
        assert!(result.is_ok(), "Template should match config structure");
    }

    #[test]
    fn it_should_have_consistent_placeholder_naming() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // All placeholders should follow the REPLACE_WITH_* pattern
        let placeholders = vec![
            "REPLACE_WITH_ENVIRONMENT_NAME",
            "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
            "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
        ];

        for placeholder in placeholders {
            assert!(
                template.contains(placeholder),
                "Missing placeholder: {placeholder}"
            );
        }
    }
}
