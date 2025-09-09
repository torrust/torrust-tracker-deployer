//! Template rendering system for configuration files
//!
//! This module provides template rendering using Tera.
//! It supports staged template resolution where different templates are resolved
//! at different lifecycle stages (e.g., static `OpenTofu` templates first, then
//! dynamic Ansible templates after VMs are provisioned and their IP addresses known).
//!
//! ## Module Structure
//!
//! - `engine` - `TemplateEngine` implementation using Tera
//! - `file` - Template file utilities
//! - `file_ops` - File operation utilities
//! - `wrappers` - Concrete template wrapper implementations

pub mod engine;
pub mod file;
pub mod file_ops;
pub mod wrappers;

// Re-export commonly used items
pub use engine::{TemplateEngine, TemplateEngineError};
pub use file_ops::{copy_file_with_dir_creation, write_file_with_dir_creation, FileOperationError};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_file_with_dir_creation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("subdir/dest.txt");

        std::fs::write(&source_file, "test content")?;

        copy_file_with_dir_creation(&source_file, &dest_file)?;

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
        copy_file_with_dir_creation(&source_file, &dest_file)?;

        assert!(dest_file.exists());
        let content = std::fs::read_to_string(&dest_file)?;
        assert_eq!(content, "build test");

        Ok(())
    }
}
