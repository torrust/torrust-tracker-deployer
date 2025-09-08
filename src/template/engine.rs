//! Template Engine Implementation
//!
//! Provides the `TemplateEngine` struct that handles template validation and rendering with Tera.

use serde::Serialize;
use tera::Tera;
use thiserror::Error;

/// Errors that can occur during template engine operations
#[derive(Debug, Error)]
pub enum TemplateEngineError {
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
}

/// Template processing engine for validation and rendering
#[derive(Debug, Default)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Creates a new `TemplateEngine` instance with an empty Tera engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            tera: Tera::default(),
        }
    }

    /// Creates a new `TemplateEngine` with template content and validates it with the given context
    ///
    /// This method combines template creation and validation to ensure templates are always
    /// instantiated in a valid state. It will fail if:
    /// - Template has syntax errors
    /// - Template references undefined variables
    /// - Template cannot be rendered for any reason
    ///
    /// # Errors
    /// Returns an error if template content cannot be parsed or validation fails
    pub fn render<T: Serialize>(
        &mut self,
        template_name: &str,
        template_content: &str,
        context: &T,
    ) -> Result<String, TemplateEngineError> {
        // Add the template content to this validator instance
        self.add_template(template_name, template_content)?;

        // Validate the template by rendering it
        let validated_content = self.render_template(template_name, context)?;

        Ok(validated_content)
    }

    /// Adds template content to this validator instance
    ///
    /// # Errors
    /// Returns an error if the template content cannot be parsed
    fn add_template(
        &mut self,
        template_name: &str,
        template_content: &str,
    ) -> Result<(), TemplateEngineError> {
        self.tera
            .add_raw_template(template_name, template_content)
            .map_err(|source| TemplateEngineError::TemplateParse {
                template_name: template_name.to_string(),
                source,
            })?;

        Ok(())
    }

    /// Renders a template by name with the given context and validates the result
    ///
    /// # Errors
    /// Returns an error if template rendering fails or variables cannot be substituted
    fn render_template<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
    ) -> Result<String, TemplateEngineError> {
        // Convert context to Tera context
        let tera_context = tera::Context::from_serialize(context)
            .map_err(|source| TemplateEngineError::ContextSerialization { source })?;

        // Render template to string
        let rendered_content =
            self.tera
                .render(template_name, &tera_context)
                .map_err(|source| TemplateEngineError::TemplateRender {
                    template_name: template_name.to_string(),
                    source,
                })?;

        // Verify no placeholders remain (basic check for {{ }} patterns)
        if rendered_content.contains("{{") && rendered_content.contains("}}") {
            return Err(TemplateEngineError::UnresolvedPlaceholders);
        }

        Ok(rendered_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestContext {
        name: String,
        value: u32,
    }

    #[derive(Serialize)]
    struct PartialContext {
        name: String,
    }

    // Tests for TemplateEngine::new()
    #[test]
    fn it_should_create_new_validator_instance() {
        let validator = TemplateEngine::new();

        // Verify it was created successfully - we can't inspect internal state
        // but we can verify the Debug trait works (indicating successful creation)
        assert!(format!("{validator:?}").contains("TemplateEngine"));
    }

    #[test]
    fn it_should_create_multiple_independent_validators() {
        let validator1 = TemplateEngine::new();
        let validator2 = TemplateEngine::new();

        // Both should be successfully created
        assert!(format!("{validator1:?}").contains("TemplateEngine"));
        assert!(format!("{validator2:?}").contains("TemplateEngine"));
    }

    // Tests for TemplateEngine::render()
    #[test]
    fn it_should_render_simple_template_successfully() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name}}! Value: {{value}}";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        let rendered_content = validator.render("test_template", template_content, &context)?;

        assert_eq!(rendered_content, "Hello World! Value: 42");
        Ok(())
    }

    #[test]
    fn it_should_render_template_with_no_variables() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "This is a static template with no variables.";
        let context = TestContext {
            name: "unused".to_string(),
            value: 0,
        };

        let rendered_content = validator.render("static_template", template_content, &context)?;

        assert_eq!(
            rendered_content,
            "This is a static template with no variables."
        );
        Ok(())
    }

    #[test]
    fn it_should_render_empty_template() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "";
        let context = TestContext {
            name: "test".to_string(),
            value: 42,
        };

        let rendered_content = validator.render("empty_template", template_content, &context)?;

        assert_eq!(rendered_content, "");
        Ok(())
    }

    #[test]
    fn it_should_handle_empty_template_name() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name}}!";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        let rendered_content = validator.render("", template_content, &context)?;

        assert_eq!(rendered_content, "Hello World!");
        Ok(())
    }

    #[test]
    fn it_should_fail_when_template_has_malformed_syntax() {
        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name! Invalid syntax";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        let result = validator.render("malformed_template", template_content, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateEngineError::TemplateParse { template_name, .. } => {
                assert_eq!(template_name, "malformed_template");
            }
            other => panic!("Expected TemplateParse error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_fail_when_template_references_undefined_variable() {
        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name}}! Value: {{undefined_variable}}";
        let context = PartialContext {
            name: "World".to_string(),
        };

        let result = validator.render("undefined_var_template", template_content, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateEngineError::TemplateRender { template_name, .. } => {
                assert_eq!(template_name, "undefined_var_template");
            }
            other => panic!("Expected TemplateRender error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_fail_when_context_serialization_fails() {
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

        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name}}!";
        let context = FailingSerialize;

        let result = validator.render("serialize_fail_template", template_content, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateEngineError::ContextSerialization { .. } => {
                // Expected
            }
            other => panic!("Expected ContextSerialization error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_render_yaml_like_template() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "name: {{name}}\nvalue: {{value}}\nstatic: true";
        let context = TestContext {
            name: "test".to_string(),
            value: 42,
        };

        let rendered_content = validator.render("yaml_template", template_content, &context)?;

        assert!(rendered_content.contains("name: test"));
        assert!(rendered_content.contains("value: 42"));
        assert!(rendered_content.contains("static: true"));
        Ok(())
    }

    #[test]
    fn it_should_render_different_templates_with_same_validator() -> Result<(), TemplateEngineError>
    {
        let mut validator = TemplateEngine::new();

        let context = TestContext {
            name: "Alice".to_string(),
            value: 100,
        };

        // Render first template
        let template1 = "Hello {{name}}!";
        let result1 = validator.render("template1", template1, &context)?;
        assert_eq!(result1, "Hello Alice!");

        // Render second template with same validator
        let template2 = "Value is: {{value}}";
        let result2 = validator.render("template2", template2, &context)?;
        assert_eq!(result2, "Value is: 100");

        Ok(())
    }

    #[test]
    fn it_should_handle_complex_template_with_multiple_variables() -> Result<(), TemplateEngineError>
    {
        let mut validator = TemplateEngine::new();
        let template_content = r#"
# Configuration for {{name}}
version: "1.0"
settings:
  port: {{value}}
  enabled: true
  name: "{{name}}"
"#;
        let context = TestContext {
            name: "MyApp".to_string(),
            value: 8080,
        };

        let rendered_content = validator.render("config_template", template_content, &context)?;

        assert!(rendered_content.contains("# Configuration for MyApp"));
        assert!(rendered_content.contains("port: 8080"));
        assert!(rendered_content.contains(r#"name: "MyApp""#));
        Ok(())
    }

    #[test]
    fn it_should_handle_special_characters_in_template_name() -> Result<(), TemplateEngineError> {
        let mut validator = TemplateEngine::new();
        let template_content = "Hello {{name}}!";
        let context = TestContext {
            name: "World".to_string(),
            value: 42,
        };

        // Test with special characters in template name
        let rendered_content =
            validator.render("template-with_special.chars", template_content, &context)?;

        assert_eq!(rendered_content, "Hello World!");
        Ok(())
    }
}
