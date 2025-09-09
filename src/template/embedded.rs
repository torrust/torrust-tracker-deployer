use rust_embed::RustEmbed;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during template manager operations
#[derive(Debug, Error)]
pub enum TemplateManagerError {
    #[error("Failed to create templates directory: {path}")]
    DirectoryCreation {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Template file not found in embedded resources: {relative_path}")]
    TemplateNotFound { relative_path: String },

    #[error("Invalid UTF-8 in embedded template: {relative_path}")]
    InvalidUtf8 {
        relative_path: String,
        #[source]
        source: std::str::Utf8Error,
    },

    #[error("Failed to create parent directory for template: {path}")]
    ParentDirectoryCreation {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write template file: {path}")]
    TemplateWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Template file already exists: {path}")]
    TemplateAlreadyExists { path: String },
}

/// Embedded template files from the ./templates directory
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct EmbeddedTemplates;

/// Template manager that handles on-demand creation of templates from embedded resources
pub struct TemplateManager {
    templates_dir: PathBuf,
}

impl TemplateManager {
    /// Create a new template manager with a custom templates directory
    pub fn new<P: Into<PathBuf>>(templates_dir: P) -> Self {
        Self {
            templates_dir: templates_dir.into(),
        }
    }

    /// Create the templates directory if it doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if the directory creation fails due to permissions or filesystem issues.
    pub fn ensure_templates_dir(&self) -> Result<(), TemplateManagerError> {
        if !self.templates_dir.exists() {
            fs::create_dir_all(&self.templates_dir).map_err(|source| {
                TemplateManagerError::DirectoryCreation {
                    path: self.templates_dir.display().to_string(),
                    source,
                }
            })?;
        }
        Ok(())
    }

    /// Get the path to a template file, creating it from embedded resources if it doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template is not found in embedded resources
    /// - The embedded template contains invalid UTF-8
    /// - File system operations fail (directory creation or file writing)
    pub fn get_template_path(&self, relative_path: &str) -> Result<PathBuf, TemplateManagerError> {
        let template_path = self.templates_dir.join(relative_path);

        // If the template file already exists, return its path
        if template_path.exists() {
            return Ok(template_path);
        }

        // Create the template from embedded resources
        self.create_template_from_embedded(relative_path)?;

        Ok(template_path)
    }

    /// Create a template file from embedded resources
    fn create_template_from_embedded(
        &self,
        relative_path: &str,
    ) -> Result<(), TemplateManagerError> {
        let template_path = self.templates_dir.join(relative_path);

        // Check if template already exists - don't overwrite
        if template_path.exists() {
            return Err(TemplateManagerError::TemplateAlreadyExists {
                path: template_path.display().to_string(),
            });
        }

        // Get the embedded file content
        let embedded_file = EmbeddedTemplates::get(relative_path).ok_or_else(|| {
            TemplateManagerError::TemplateNotFound {
                relative_path: relative_path.to_string(),
            }
        })?;

        let content = std::str::from_utf8(&embedded_file.data).map_err(|source| {
            TemplateManagerError::InvalidUtf8 {
                relative_path: relative_path.to_string(),
                source,
            }
        })?;

        // Ensure parent directory exists
        if let Some(parent) = template_path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                TemplateManagerError::ParentDirectoryCreation {
                    path: template_path.display().to_string(),
                    source,
                }
            })?;
        }

        // Write the content to the file
        fs::write(&template_path, content).map_err(|source| {
            TemplateManagerError::TemplateWrite {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        tracing::debug!("Created template from embedded resources: {relative_path}");

        Ok(())
    }

    /// Get the templates directory path
    #[must_use]
    pub fn templates_dir(&self) -> &Path {
        &self.templates_dir
    }

    /// Clean the templates directory by removing all files and subdirectories
    ///
    /// This is useful for development/testing to ensure fresh templates are used
    /// from embedded resources.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory removal fails due to permissions or filesystem issues.
    pub fn clean_templates_dir(&self) -> Result<(), TemplateManagerError> {
        if self.templates_dir.exists() {
            fs::remove_dir_all(&self.templates_dir).map_err(|source| {
                TemplateManagerError::DirectoryCreation {
                    path: self.templates_dir.display().to_string(),
                    source,
                }
            })?;
            tracing::debug!(
                "Cleaned templates directory: {}",
                self.templates_dir.display()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_templates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);

        // Directory should not exist initially
        assert!(!templates_path.exists());

        // After ensuring directory, it should exist
        manager.ensure_templates_dir().unwrap();
        assert!(templates_path.exists());
        assert!(templates_path.is_dir());
    }

    #[test]
    fn it_should_create_template_from_embedded_resources() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Try to get a template path - this should create the file from embedded resources
        let template_path = manager.get_template_path("ansible/ansible.cfg").unwrap();

        // The file should now exist
        assert!(template_path.exists());
        assert!(template_path.is_file());

        // The content should match what we expect (basic verification)
        let content = fs::read_to_string(&template_path).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn it_should_fail_when_template_not_found_in_embedded_resources() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Try to get a non-existent template path
        let result = manager.get_template_path("non-existent/template.txt");

        // This should fail with TemplateNotFound error
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            TemplateManagerError::TemplateNotFound { relative_path } => {
                assert_eq!(relative_path, "non-existent/template.txt");
            }
            _ => panic!("Expected TemplateNotFound error, got: {error:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn it_should_fail_when_directory_creation_is_denied() {
        let temp_dir = TempDir::new().unwrap();
        let read_only_dir = temp_dir.path().join("read_only");
        fs::create_dir(&read_only_dir).unwrap();

        // Make directory read-only
        let mut perms = fs::metadata(&read_only_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&read_only_dir, perms).unwrap();

        let templates_path = read_only_dir.join("should_fail");
        let manager = TemplateManager::new(&templates_path);

        // This should fail with DirectoryCreation error
        let result = manager.ensure_templates_dir();

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            TemplateManagerError::DirectoryCreation { path, source } => {
                assert!(path.contains("should_fail"));
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected DirectoryCreation error, got: {error:?}"),
        }

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&read_only_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&read_only_dir, perms).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn it_should_fail_when_parent_directory_creation_is_denied() {
        let temp_dir = TempDir::new().unwrap();
        let read_only_dir = temp_dir.path().join("read_only");
        fs::create_dir(&read_only_dir).unwrap();

        // Make directory read-only
        let mut perms = fs::metadata(&read_only_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&read_only_dir, perms).unwrap();

        let templates_path = temp_dir.path().join("templates");
        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Create a template path that requires parent directory creation inside read-only dir
        // We'll need to temporarily modify the manager to use a path that will fail
        let failing_manager = TemplateManager::new(&read_only_dir);

        let result = failing_manager.get_template_path("some/nested/template.txt");

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            TemplateManagerError::TemplateNotFound { .. } => {
                // This is expected since the template doesn't exist in embedded resources
                // But let's test with a real template that exists
            }
            _ => panic!("Unexpected error: {error:?}"),
        }

        // Test with a real template that exists
        let result = failing_manager.get_template_path("ansible/ansible.cfg");

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            TemplateManagerError::ParentDirectoryCreation { path, source } => {
                assert!(path.contains("ansible.cfg"));
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected ParentDirectoryCreation error, got: {error:?}"),
        }

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&read_only_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&read_only_dir, perms).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn it_should_fail_when_template_write_is_denied() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");
        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Create the ansible subdirectory
        let ansible_dir = templates_path.join("ansible");
        fs::create_dir(&ansible_dir).unwrap();

        // Make the ansible directory read-only so file creation will fail
        let mut perms = fs::metadata(&ansible_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&ansible_dir, perms).unwrap();

        // Try to get a template path - this should fail when trying to write the file
        let result = manager.get_template_path("ansible/ansible.cfg");

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            TemplateManagerError::TemplateWrite { path, source } => {
                assert!(path.contains("ansible.cfg"));
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected TemplateWrite error, got: {error:?}"),
        }

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&ansible_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ansible_dir, perms).unwrap();
    }

    #[test]
    fn it_should_return_existing_template_path_without_recreating() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // First call should create the template
        let template_path1 = manager.get_template_path("ansible/ansible.cfg").unwrap();
        assert!(template_path1.exists());

        // Modify the file content to verify it's not recreated
        let original_content = fs::read_to_string(&template_path1).unwrap();
        fs::write(&template_path1, "modified content").unwrap();

        // Second call should return the same path without recreating
        let template_path2 = manager.get_template_path("ansible/ansible.cfg").unwrap();
        assert_eq!(template_path1, template_path2);

        // Content should still be modified, proving it wasn't recreated
        let current_content = fs::read_to_string(&template_path2).unwrap();
        assert_eq!(current_content, "modified content");
        assert_ne!(current_content, original_content);
    }

    #[test]
    fn it_should_provide_correct_templates_dir_path() {
        let test_path = PathBuf::from("/test/path");
        let manager = TemplateManager::new(&test_path);

        assert_eq!(manager.templates_dir(), Path::new("/test/path"));
    }

    #[test]
    fn it_should_handle_nested_template_paths() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Test deeply nested template path
        let template_path = manager.get_template_path("tofu/lxd/main.tf").unwrap();

        assert!(template_path.exists());
        assert!(template_path.is_file());

        // Verify parent directories were created
        assert!(template_path.parent().unwrap().exists());
        assert!(template_path.parent().unwrap().parent().unwrap().exists());
    }

    #[test]
    fn it_should_fail_when_trying_to_create_existing_template() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Create a template first time
        let first_path = manager.get_template_path("ansible/ansible.cfg").unwrap();

        // Try to create the same template again through get_template_path - should succeed since it returns existing
        let second_path = manager.get_template_path("ansible/ansible.cfg").unwrap();

        // Both paths should be the same and the file should exist
        assert_eq!(first_path, second_path);
        assert!(second_path.exists());
    }
    #[test]
    fn it_should_clean_templates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("test_templates");

        let manager = TemplateManager::new(&templates_path);
        manager.ensure_templates_dir().unwrap();

        // Create some templates
        let template_path1 = manager.get_template_path("ansible/ansible.cfg").unwrap();
        let template_path2 = manager.get_template_path("tofu/lxd/main.tf").unwrap();

        // Verify templates exist
        assert!(template_path1.exists());
        assert!(template_path2.exists());
        assert!(templates_path.exists());

        // Clean the directory
        manager.clean_templates_dir().unwrap();

        // Templates directory should be gone
        assert!(!templates_path.exists());
        assert!(!template_path1.exists());
        assert!(!template_path2.exists());
    }

    #[test]
    fn it_should_handle_clean_on_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("nonexistent_templates");

        let manager = TemplateManager::new(&templates_path);

        // Clean should not fail on non-existent directory
        let result = manager.clean_templates_dir();
        assert!(result.is_ok());
    }
}
