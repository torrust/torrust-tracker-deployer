//! Template wrapper for templates/ansible/inventory.yml
//!
//! This template has mandatory variables that must be provided at construction time.

use crate::template::{StaticContext, TemplateRenderer};
use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Template wrapper for templates/ansible/inventory.yml with mandatory variables as fields
#[derive(Debug)]
pub struct InventoryTemplate {
    template_path: PathBuf,
    ansible_host: String,
    ansible_ssh_private_key_file: String,
    validated_content: String,
}

/// Context for the inventory template (generated from the struct fields)
#[derive(Serialize)]
struct InventoryContext {
    ansible_host: String,
    ansible_ssh_private_key_file: String,
}

impl InventoryTemplate {
    /// Creates a new `InventoryTemplate`, validating the template file and variable substitution
    ///
    /// # Errors
    /// Returns an error if:
    /// - Template file doesn't exist or cannot be read
    /// - Required variables are missing from the template
    /// - Template validation fails
    pub fn new(
        template_path: PathBuf,
        ansible_host: String,
        ansible_ssh_private_key_file: String,
    ) -> Result<Self> {
        // Validate template file exists and is readable
        if !template_path.exists() {
            return Err(anyhow!(
                "Template file not found: {}",
                template_path.display()
            ));
        }

        let template_content = fs::read_to_string(&template_path).with_context(|| {
            format!("Failed to read template file: {}", template_path.display())
        })?;

        // Validate that required placeholders exist in template
        let required_vars = ["ansible_host", "ansible_ssh_private_key_file"];
        for var in &required_vars {
            let placeholder = format!("{{{{{var}}}}}");
            if !template_content.contains(&placeholder) {
                return Err(anyhow!(
                    "Template file missing required placeholder '{}': {}",
                    placeholder,
                    template_path.display()
                ));
            }
        }

        // Test variable substitution to ensure it works
        let test_context = InventoryContext {
            ansible_host: ansible_host.clone(),
            ansible_ssh_private_key_file: ansible_ssh_private_key_file.clone(),
        };

        let engine = crate::template::TemplateEngine::with_template(&template_path)?;
        let validated_content = engine
            .validate_template_substitution(&template_path, &test_context)
            .with_context(|| "Template validation failed during construction")?;

        Ok(Self {
            template_path,
            ansible_host,
            ansible_ssh_private_key_file,
            validated_content,
        })
    }

    /// Get the ansible host value
    #[must_use]
    pub fn ansible_host(&self) -> &str {
        &self.ansible_host
    }

    /// Get the ansible SSH private key file path
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> &str {
        &self.ansible_ssh_private_key_file
    }
}

impl TemplateRenderer for InventoryTemplate {
    type Context = StaticContext; // We don't use external context since we have fields

    fn template_path(&self) -> &Path {
        &self.template_path
    }

    fn required_variables(&self) -> Vec<&'static str> {
        vec!["ansible_host", "ansible_ssh_private_key_file"]
    }

    fn render(&self, _context: &Self::Context, output_path: &Path) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        // Write the pre-validated content directly
        fs::write(output_path, &self.validated_content).with_context(|| {
            format!("Failed to write template output: {}", output_path.display())
        })?;

        Ok(())
    }

    fn validate_context(&self, _context: &Self::Context) -> Result<()> {
        // Validation is built-in since fields are mandatory at construction
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_inventory_template_creation() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create a valid template file
        std::fs::write(&template_file,
            "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n"
        ).unwrap();

        let template = InventoryTemplate::new(
            template_file,
            "192.168.1.100".to_string(),
            "/path/to/key".to_string(),
        )
        .unwrap();

        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(template.ansible_ssh_private_key_file(), "/path/to/key");
        assert_eq!(
            template.required_variables(),
            vec!["ansible_host", "ansible_ssh_private_key_file"]
        );
    }

    #[test]
    fn test_inventory_template_context_generation() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create a valid template file
        std::fs::write(&template_file,
            "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n"
        ).unwrap();

        let template = InventoryTemplate::new(
            template_file,
            "10.0.0.1".to_string(),
            "/home/user/.ssh/id_rsa".to_string(),
        )
        .unwrap();

        assert_eq!(template.ansible_host(), "10.0.0.1");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
    }

    #[test]
    fn test_template_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.yml");

        let result = InventoryTemplate::new(
            nonexistent_file,
            "10.0.0.1".to_string(),
            "/home/user/.ssh/id_rsa".to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template file not found"));
    }

    #[test]
    fn test_missing_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create template with missing placeholder
        std::fs::write(
            &template_file,
            "[all]\nserver ansible_host={{ansible_host}}\n", // missing ansible_ssh_private_key_file
        )
        .unwrap();

        let result = InventoryTemplate::new(
            template_file,
            "10.0.0.1".to_string(),
            "/home/user/.ssh/id_rsa".to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing required placeholder"));
    }

    #[test]
    fn test_early_error_detection_both_variables_missing() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create template with no placeholder variables at all
        std::fs::write(
            &template_file,
            "[all]\nserver ansible_host=192.168.1.1\n", // Static values instead of template variables
        )
        .unwrap();

        let result = InventoryTemplate::new(
            template_file,
            "10.0.0.1".to_string(),
            "/home/user/.ssh/id_rsa".to_string(),
        );

        // Should fail because both required variables are missing
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("missing required placeholder"));
    }

    #[test]
    fn test_early_error_detection_template_validation_fails() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create template with malformed Tera syntax
        std::fs::write(
            &template_file,
            "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\nmalformed={{unclosed_var\n",
        )
        .unwrap();

        let result = InventoryTemplate::new(
            template_file,
            "10.0.0.1".to_string(),
            "/home/user/.ssh/id_rsa".to_string(),
        );

        // Should fail during template validation
        assert!(result.is_err());
    }

    #[test]
    fn test_early_error_detection_unreadable_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create file but make it unreadable (on Unix systems)
        std::fs::write(&template_file, "content").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&template_file).unwrap().permissions();
            perms.set_mode(0o000); // No permissions
            std::fs::set_permissions(&template_file, perms).unwrap();

            let result = InventoryTemplate::new(
                template_file.clone(),
                "10.0.0.1".to_string(),
                "/home/user/.ssh/id_rsa".to_string(),
            );

            assert!(result.is_err());

            // Restore permissions for cleanup
            let mut perms = std::fs::metadata(&template_file).unwrap().permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&template_file, perms).unwrap();
        }
    }

    #[test]
    fn test_template_validation_at_construction() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("inventory.yml");

        // Create valid template
        std::fs::write(&template_file,
            "[all]\nserver ansible_host={{ansible_host}} ansible_ssh_private_key_file={{ansible_ssh_private_key_file}}\n"
        ).unwrap();

        // Template validation happens during construction, not during render
        let template = InventoryTemplate::new(
            template_file,
            "192.168.1.100".to_string(),
            "/home/user/.ssh/test_key".to_string(),
        )
        .unwrap();

        // Verify that the template was pre-validated and contains rendered content
        assert_eq!(template.ansible_host(), "192.168.1.100");
        assert_eq!(
            template.ansible_ssh_private_key_file(),
            "/home/user/.ssh/test_key"
        );
    }
}
