//! Template rendering system for configuration files
//!
//! This module provides a trait-based approach to template rendering using Tera.
//! It supports staged template resolution where different templates are resolved
//! at different lifecycle stages (e.g., static `OpenTofu` templates first, then
//! dynamic Ansible templates after VMs are provisioned and their IP addresses known).
//!
//! ## Module Structure
//!
//! - `renderer` - `TemplateRenderer` trait definition
//! - `engine` - `TemplateEngine` implementation using Tera
//! - `utils` - Utility functions for file operations
//! - `wrappers` - Concrete template wrapper implementations

pub mod engine;
pub mod file;
pub mod renderer;
pub mod utils;
pub mod wrappers;

// Re-export commonly used items
pub use engine::TemplateEngine;
pub use renderer::TemplateRenderer;
pub use utils::copy_static_file;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
