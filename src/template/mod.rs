//! Template rendering system for configuration files
//!
//! This module provides a trait-based approach to template rendering using Tera.
//! It supports staged template resolution where different templates are resolved
//! at different lifecycle stages (e.g., static `OpenTofu` templates first, then
//! dynamic Ansible templates after VMs are provisioned and their IP addresses known).
//!
//! ## Module Structure
//! - `renderer` - `TemplateRenderer` trait definition
//! - `engine` - `TemplateEngine` implementation using Tera
//! - `context` - Template context types
//! - `utils` - Utility functions for file operations
//! - `wrappers` - Concrete template wrapper implementations

pub mod context;
pub mod engine;
pub mod renderer;
pub mod utils;
pub mod wrappers;

// Re-export commonly used items
pub use context::StaticContext;
pub use engine::TemplateEngine;
pub use renderer::TemplateRenderer;
pub use utils::copy_static_file;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_static_context_creation() {
        let context = StaticContext::default();
        let serialized = serde_json::to_value(&context).unwrap();
        assert!(serialized.is_object());
    }

    #[test]
    fn test_copy_static_file() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("subdir/dest.txt");

        std::fs::write(&source_file, "test content")?;

        copy_static_file(&source_file, &dest_file)?;

        let content = std::fs::read_to_string(&dest_file)?;
        assert_eq!(content, "test content");

        Ok(())
    }

    #[test]
    fn test_template_engine_creation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("test.yml.tera");
        std::fs::write(&template_file, "test: {{value}}")?;

        let _engine = TemplateEngine::with_template(&template_file)?;

        // Test that the engine was created successfully
        // We can't test much more without exposing internals, but creation is the main thing
        Ok(())
    }

    #[test]
    fn test_template_engine_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("missing.yml.tera");

        let result = TemplateEngine::with_template(&non_existent);

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Failed to read template file"));
    }

    #[test]
    fn test_template_engine_validation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("test.yml.tera");
        std::fs::write(&template_file, "name: {{name}}\nvalue: {{value}}")?;

        let engine = TemplateEngine::with_template(&template_file)?;

        // Test successful validation
        let context = serde_json::json!({
            "name": "test",
            "value": "hello"
        });

        let result = engine.validate_template_substitution(&template_file, &context)?;
        assert!(result.contains("name: test"));
        assert!(result.contains("value: hello"));

        Ok(())
    }

    #[test]
    fn test_template_engine_validation_missing_variable() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("test.yml.tera");
        std::fs::write(&template_file, "name: {{name}}\nvalue: {{missing_var}}")?;

        let engine = TemplateEngine::with_template(&template_file)?;

        // Test with missing variable
        let context = serde_json::json!({
            "name": "test"
            // missing_var is not provided
        });

        let result = engine.validate_template_substitution(&template_file, &context);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_build_directory_creation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("build/subdir/deep/dest.txt");

        std::fs::write(&source_file, "build test")?;

        // This should create all necessary parent directories
        copy_static_file(&source_file, &dest_file)?;

        assert!(dest_file.exists());
        let content = std::fs::read_to_string(&dest_file)?;
        assert_eq!(content, "build test");

        Ok(())
    }
}
