//! Template Engine Implementation
//!
//! Provides the `TemplateEngine` struct that handles actual template rendering with Tera.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::path::Path;
use tera::Tera;

/// Template rendering utilities and helper functions
#[derive(Debug)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Creates a new `TemplateEngine` instance
    ///
    /// # Errors
    /// Returns an error if Tera template engine cannot be initialized
    pub fn new() -> Result<Self> {
        let tera = Tera::new("templates/**/*").context("Failed to initialize Tera engine")?;
        Ok(Self { tera })
    }

    /// Creates a new `TemplateEngine` instance with a single template file
    ///
    /// # Errors
    /// Returns an error if the template file cannot be loaded or parsed
    pub fn with_template(template_path: &Path) -> Result<Self> {
        let mut tera = Tera::default();

        let template_content = std::fs::read_to_string(template_path).with_context(|| {
            format!("Failed to read template file: {}", template_path.display())
        })?;

        let template_name = template_path
            .strip_prefix("templates/")
            .unwrap_or(template_path)
            .to_string_lossy()
            .to_string();

        tera.add_raw_template(&template_name, &template_content)
            .with_context(|| format!("Failed to parse template: {}", template_path.display()))?;

        Ok(Self { tera })
    }

    /// Render a template file with the given context
    ///
    /// # Errors
    /// Returns an error if template cannot be rendered, context serialization fails,
    /// or output file cannot be written
    pub fn render_template<T: Serialize>(
        &self,
        template_path: &Path,
        context: &T,
        output_path: &Path,
    ) -> Result<()> {
        // Get template name relative to templates directory
        let template_name = template_path
            .strip_prefix("templates/")
            .unwrap_or(template_path)
            .to_string_lossy()
            .to_string();

        // Convert context to Tera context
        let tera_context = tera::Context::from_serialize(context)
            .context("Failed to serialize template context")?;

        // Render template
        let rendered = self
            .tera
            .render(&template_name, &tera_context)
            .context(format!("Failed to render template: {template_name}"))?;

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create output directory: {}",
                parent.display()
            ))?;
        }

        // Write rendered content to output file
        std::fs::write(output_path, rendered).context(format!(
            "Failed to write rendered template to: {}",
            output_path.display()
        ))?;

        Ok(())
    }

    /// Validate that a context contains required variables
    ///
    /// # Errors
    /// Returns an error if any required variables are missing from the context
    pub fn validate_required_variables<T: Serialize>(
        &self,
        context: &T,
        required_vars: &[&str],
    ) -> Result<()> {
        // Serialize context to get access to fields
        let value =
            serde_json::to_value(context).context("Failed to serialize context for validation")?;

        let obj = value
            .as_object()
            .ok_or_else(|| anyhow!("Context must be a JSON object"))?;

        // Check each required variable
        for var in required_vars {
            if !obj.contains_key(*var) {
                return Err(anyhow!("Required template variable missing: {}", var));
            }

            // Check if the value is null
            if obj[*var].is_null() {
                return Err(anyhow!("Required template variable is null: {}", var));
            }
        }

        Ok(())
    }

    /// Validates template substitution by rendering in memory and returning the result
    ///
    /// # Errors
    /// Returns an error if template rendering fails or variables cannot be substituted
    pub fn validate_template_substitution<T: Serialize>(
        &self,
        template_path: &Path,
        context: &T,
    ) -> Result<String> {
        // Get template name relative to templates directory
        let template_name = template_path
            .strip_prefix("templates/")
            .unwrap_or(template_path)
            .to_string_lossy()
            .to_string();

        // Render template to string instead of file
        let rendered_content = self
            .tera
            .render(&template_name, &tera::Context::from_serialize(context)?)
            .with_context(|| {
                format!("Failed to validate template substitution: {template_name}")
            })?;

        // Verify no placeholders remain (basic check for {{ }} patterns)
        if rendered_content.contains("{{") && rendered_content.contains("}}") {
            return Err(anyhow!(
                "Template validation failed: unresolved placeholders remain in rendered content"
            ));
        }

        Ok(rendered_content)
    }
}
