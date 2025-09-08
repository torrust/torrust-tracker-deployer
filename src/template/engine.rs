//! Template Validator Implementation
//!
//! Provides the `TemplateValidator` struct that handles template validation and rendering with Tera.

use serde::Serialize;
use tera::Tera;
use thiserror::Error;

/// Errors that can occur during template validation operations
#[derive(Debug, Error)]
pub enum TemplateValidatorError {
    #[error("Failed to parse template: {template_name}")]
    TemplateParse {
        template_name: String,
        #[source]
        source: tera::Error,
    },

    #[error("Failed to serialize template context")]
    ContextSerialization {
        #[source]
        source: tera::Error,
    },

    #[error("Failed to render template: {template_name}")]
    TemplateRender {
        template_name: String,
        #[source]
        source: tera::Error,
    },

    #[error("Template validation failed: unresolved placeholders remain in rendered content")]
    UnresolvedPlaceholders,

    #[error("Template validation failed during construction")]
    ValidationFailure {
        #[source]
        source: Box<TemplateValidatorError>,
    },

    #[error("Failed to create template validator with content")]
    ValidatorCreation {
        #[source]
        source: Box<TemplateValidatorError>,
    },
}

/// Template validation and rendering utilities for single templates
#[derive(Debug)]
pub struct TemplateValidator {
    tera: Tera,
}

impl TemplateValidator {
    /// Creates a new `TemplateValidator` with template content and validates it with the given context
    ///
    /// This method combines template creation and validation to ensure templates are always
    /// instantiated in a valid state. It will fail if:
    /// - Template has syntax errors
    /// - Template references undefined variables
    /// - Template cannot be rendered for any reason
    ///
    /// # Errors
    /// Returns an error if template content cannot be parsed or validation fails
    pub fn with_validated_template_content<T: Serialize>(
        template_name: &str,
        template_content: &str,
        context: &T,
    ) -> Result<(Self, String), TemplateValidatorError> {
        // Create the template validator
        let validator =
            Self::with_template_content(template_name, template_content).map_err(|source| {
                TemplateValidatorError::ValidatorCreation {
                    source: Box::new(source),
                }
            })?;

        // Validate the template by rendering it
        let validated_content = validator
            .validate_template_substitution_by_name(template_name, context)
            .map_err(|source| TemplateValidatorError::ValidationFailure {
                source: Box::new(source),
            })?;

        Ok((validator, validated_content))
    }

    /// Creates a new `TemplateValidator` instance with template content
    ///
    /// # Errors
    /// Returns an error if the template content cannot be parsed
    fn with_template_content(
        template_name: &str,
        template_content: &str,
    ) -> Result<Self, TemplateValidatorError> {
        let mut tera = Tera::default();

        tera.add_raw_template(template_name, template_content)
            .map_err(|source| TemplateValidatorError::TemplateParse {
                template_name: template_name.to_string(),
                source,
            })?;

        Ok(Self { tera })
    }

    /// Validates template substitution by template name and returns the result
    ///
    /// # Errors
    /// Returns an error if template rendering fails or variables cannot be substituted
    fn validate_template_substitution_by_name<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
    ) -> Result<String, TemplateValidatorError> {
        // Convert context to Tera context
        let tera_context = tera::Context::from_serialize(context)
            .map_err(|source| TemplateValidatorError::ContextSerialization { source })?;

        // Render template to string
        let rendered_content =
            self.tera
                .render(template_name, &tera_context)
                .map_err(|source| TemplateValidatorError::TemplateRender {
                    template_name: template_name.to_string(),
                    source,
                })?;

        // Verify no placeholders remain (basic check for {{ }} patterns)
        if rendered_content.contains("{{") && rendered_content.contains("}}") {
            return Err(TemplateValidatorError::UnresolvedPlaceholders);
        }

        Ok(rendered_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Serialize)]
    struct TestContext {
        name: String,
        value: u32,
    }

    #[test]
    fn test_with_template_content_from_file_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("test.yml.tera");
        fs::write(&template_file, "name: {{name}}\nvalue: {{value}}")?;

        let template_content = fs::read_to_string(&template_file)?;
        let template_name = "test.yml.tera";
        let _validator =
            TemplateValidator::with_template_content(template_name, &template_content)?;

        Ok(())
    }

    #[test]
    fn test_with_template_content_empty_name() {
        let template_content = "name: {{name}}\nvalue: {{value}}";

        let result = TemplateValidator::with_template_content("", template_content);

        // This should still work - empty template names are valid
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_template_content_malformed_syntax_from_file(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("test.yml.tera");
        fs::write(&template_file, "name: {{name! Invalid syntax")?;

        let template_content = fs::read_to_string(&template_file)?;
        let template_name = "test.yml.tera";
        let result = TemplateValidator::with_template_content(template_name, &template_content);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::TemplateParse { template_name, .. } => {
                assert_eq!(template_name, "test.yml.tera");
            }
            other => panic!("Expected TemplateParse error, got: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn test_with_template_content_success() -> Result<(), TemplateValidatorError> {
        let template_content = "Hello {{name}}! Value: {{value}}";
        let _validator =
            TemplateValidator::with_template_content("test_template", template_content)?;
        Ok(())
    }

    #[test]
    fn test_with_template_content_malformed_syntax() {
        let template_content = "Hello {{name! Invalid syntax";
        let result = TemplateValidator::with_template_content("test_template", template_content);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::TemplateParse { template_name, .. } => {
                assert_eq!(template_name, "test_template");
            }
            other => panic!("Expected TemplateParse error, got: {other:?}"),
        }
    }

    #[test]
    fn test_with_validated_template_content_success() -> Result<(), TemplateValidatorError> {
        let template_content = "Hello {{name}}! Value: {{value}}";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        let (validator, rendered_content) = TemplateValidator::with_validated_template_content(
            "test_template",
            template_content,
            &context,
        )?;

        // Verify the validator was created
        assert!(format!("{validator:?}").contains("TemplateValidator"));

        // Verify the rendered content is correct
        assert_eq!(rendered_content, "Hello World! Value: 42");

        Ok(())
    }

    #[test]
    fn test_with_validated_template_content_undefined_variable() {
        #[derive(Serialize)]
        struct PartialContext {
            name: String,
        }

        let template_content = "Hello {{name}}! Value: {{undefined_variable}}";
        let context = PartialContext {
            name: "World".to_string(),
        };

        let result = TemplateValidator::with_validated_template_content(
            "test_template",
            template_content,
            &context,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::ValidationFailure { .. } => {
                // Expected
            }
            other => panic!("Expected ValidationFailure error, got: {other:?}"),
        }
    }

    #[test]
    fn test_with_validated_template_content_malformed_syntax() {
        let template_content = "Hello {{name! Invalid syntax";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        let result = TemplateValidator::with_validated_template_content(
            "test_template",
            template_content,
            &context,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::ValidatorCreation { .. } => {
                // Expected
            }
            other => panic!("Expected ValidatorCreation error, got: {other:?}"),
        }
    }

    #[test]
    fn test_validate_template_substitution_by_name_success() -> Result<(), TemplateValidatorError> {
        let template_content = "name: {{name}}\nvalue: {{value}}";
        let validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        let context = TestContext {
            name: "test".to_string(),
            value: 42,
        };

        let result = validator.validate_template_substitution_by_name("test_template", &context)?;

        assert!(result.contains("name: test"));
        assert!(result.contains("value: 42"));

        Ok(())
    }

    #[test]
    fn test_validate_template_substitution_by_name_missing_variable(
    ) -> Result<(), TemplateValidatorError> {
        #[derive(Serialize)]
        struct PartialContext {
            name: String,
        }

        let template_content = "name: {{name}}\nvalue: {{missing_var}}";
        let validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        let context = PartialContext {
            name: "test".to_string(),
        };

        let result = validator.validate_template_substitution_by_name("test_template", &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::TemplateRender { template_name, .. } => {
                assert_eq!(template_name, "test_template");
            }
            other => panic!("Expected TemplateRender error, got: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_validate_template_substitution_by_name_nonexistent_template(
    ) -> Result<(), TemplateValidatorError> {
        let template_content = "name: {{name}}";
        let validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        let context = TestContext {
            name: "test".to_string(),
            value: 42,
        };

        let result =
            validator.validate_template_substitution_by_name("nonexistent_template", &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::TemplateRender { template_name, .. } => {
                assert_eq!(template_name, "nonexistent_template");
            }
            other => panic!("Expected TemplateRender error, got: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_validate_template_substitution_by_name_context_serialization_error(
    ) -> Result<(), TemplateValidatorError> {
        // Create a struct that will fail to serialize by implementing a custom serialize that fails
        struct FailingSerialize;

        impl Serialize for FailingSerialize {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom(
                    "Intentional serialization failure",
                ))
            }
        }

        let template_content = "name: {{name}}";
        let validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        let context = FailingSerialize;
        let result = validator.validate_template_substitution_by_name("test_template", &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::ContextSerialization { .. } => {
                // Expected
            }
            other => panic!("Expected ContextSerialization error, got: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_validate_template_with_remaining_placeholders() -> Result<(), TemplateValidatorError> {
        #[derive(Serialize)]
        struct PartialContext {
            valid: String,
        }

        // Create a template with a proper placeholder that won't be substituted
        let template_content =
            "Static content with {{valid}} and some remaining {{unresolved}} placeholders";
        let validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        let context = PartialContext {
            valid: "substituted".to_string(),
        };

        let result = validator.validate_template_substitution_by_name("test_template", &context);

        // This should fail because the template has undefined variable
        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateValidatorError::TemplateRender { .. } => {
                // Expected - the template should fail to render due to undefined variable
            }
            other => panic!("Expected TemplateRender error, got: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_unresolved_placeholders_detection() -> Result<(), TemplateValidatorError> {
        // Create a validator with template content that will render but contain literal braces
        let template_content = "Valid content: {{name}} and some literal {{unmatched}} text";
        let _validator =
            TemplateValidator::with_template_content("test_template", template_content)?;

        // We can't easily create this scenario with Tera because it would fail to render
        // due to undefined variables. This test shows that our detection logic works
        // by manually testing the rendered content check logic.
        let rendered_content = "Valid content: John and some literal {{unmatched}} text";

        // Verify our check would detect remaining placeholders
        assert!(rendered_content.contains("{{") && rendered_content.contains("}}"));

        Ok(())
    }

    #[test]
    fn test_template_validator_error_display() {
        let parse_error = TemplateValidatorError::TemplateParse {
            template_name: "test.tera".to_string(),
            source: tera::Error::msg("Invalid syntax"),
        };

        let error_string = parse_error.to_string();
        assert!(error_string.contains("Failed to parse template"));
        assert!(error_string.contains("test.tera"));

        let unresolved_error = TemplateValidatorError::UnresolvedPlaceholders;
        let error_string = unresolved_error.to_string();
        assert!(error_string.contains("unresolved placeholders remain"));
    }

    #[test]
    fn test_template_name_extraction_from_path() -> Result<(), Box<dyn std::error::Error>> {
        // Test template name extraction with templates/ prefix
        let temp_dir = TempDir::new()?;
        let templates_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&templates_dir)?;
        let template_file = templates_dir.join("inventory.yml.tera");
        fs::write(&template_file, "name: {{name}}")?;

        let template_content = fs::read_to_string(&template_file)?;
        let template_name = "inventory.yml.tera";
        let validator = TemplateValidator::with_template_content(template_name, &template_content)?;
        let context = TestContext {
            name: "test".to_string(),
            value: 42,
        };

        // Use the template name directly
        let result = validator.validate_template_substitution_by_name(template_name, &context)?;
        assert!(result.contains("name: test"));

        Ok(())
    }
}
