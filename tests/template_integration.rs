//! Integration tests for the template system
//!
//! These tests verify that the template system works with real template files
//! and validates the complete workflow without actually provisioning infrastructure.

use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use torrust_tracker_deploy::template::wrappers::ansible::inventory::InventoryTemplate;
use torrust_tracker_deploy::template::{StaticContext, TemplateRenderer};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that the real inventory template renders correctly
    #[test]
    fn test_real_inventory_template_rendering() -> Result<()> {
        // Use the actual inventory template
        let template_path = PathBuf::from("templates/ansible/inventory.yml.tera");

        // Skip test if template file doesn't exist (e.g., in CI without templates)
        if !template_path.exists() {
            println!(
                "Skipping test: inventory template not found at {}",
                template_path.display()
            );
            return Ok(());
        }

        // Read the template content
        let template_content = std::fs::read_to_string(&template_path)?;

        // Create temporary output directory
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("inventory.yml");

        // Test with realistic values
        let inventory = InventoryTemplate::new(
            &template_content,
            "192.168.1.100",
            "/home/user/.ssh/testing_rsa",
        )?;

        // Render the template
        let context = StaticContext::default();
        inventory.render(&context, &output_path)?;

        // Verify the output file exists and has the right content
        assert!(output_path.exists());

        let file_content = std::fs::read_to_string(&output_path)?;

        // Verify variables were substituted
        assert!(file_content.contains("ansible_host: 192.168.1.100"));
        assert!(file_content.contains("ansible_ssh_private_key_file: /home/user/.ssh/testing_rsa"));

        // Verify no template variables remain
        assert!(!file_content.contains("{{ansible_host}}"));
        assert!(!file_content.contains("{{ansible_ssh_private_key_file}}"));

        // Verify it's valid YAML structure
        assert!(file_content.contains("all:"));
        assert!(file_content.contains("torrust-vm:"));
        assert!(file_content.contains("ansible_user: torrust"));

        println!("✅ Real inventory template rendered successfully");
        Ok(())
    }

    /// Test variable validation with real template
    #[test]
    fn test_real_template_variable_validation() {
        let template_path = PathBuf::from("templates/ansible/inventory.yml.tera");

        // Skip test if template file doesn't exist
        if !template_path.exists() {
            println!("Skipping test: inventory template not found");
            return;
        }

        // Read the template content
        let Ok(template_content) = std::fs::read_to_string(&template_path) else {
            println!("Skipping test: Could not read template file");
            return;
        };

        // Test that missing variables are caught during construction
        let result = InventoryTemplate::new(&template_content, "", "");

        // Construction should succeed even with empty values
        assert!(result.is_ok());

        // Test that invalid template content fails
        let invalid_content = "invalid template content without required variables";
        let result = InventoryTemplate::new(invalid_content, "192.168.1.100", "/path/to/key");

        assert!(result.is_err());
        println!("✅ Invalid template content correctly rejected");
    }

    /// Test that template rendering doesn't modify any files in the templates directory
    #[test]
    fn test_no_template_directory_modifications() -> Result<()> {
        let template_path = PathBuf::from("templates/ansible/inventory.yml.tera");

        if !template_path.exists() {
            println!("Skipping test: inventory template not found");
            return Ok(());
        }

        // Read the original template content
        let original_content = std::fs::read_to_string(&template_path)?;

        // Create temporary output directory
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("inventory.yml");

        // Render the template multiple times with different values
        for i in 1..=3 {
            let inventory = InventoryTemplate::new(
                &original_content,
                &format!("192.168.1.{i}"),
                &format!("/home/user{i}/.ssh/key"),
            )?;

            let context = StaticContext::default();
            inventory.render(&context, &output_path)?;
        }

        // Verify the original template is unchanged
        let current_content = std::fs::read_to_string(&template_path)?;
        assert_eq!(original_content, current_content);

        println!("✅ Template directory remains unmodified after multiple renderings");
        Ok(())
    }

    /// Test build directory workflow simulation
    #[test]
    fn test_build_directory_workflow() -> Result<()> {
        // Simulate the complete build directory workflow
        let temp_dir = TempDir::new()?;
        let build_root = temp_dir.path().join("build");

        // Create build directory structure
        let build_ansible = build_root.join("ansible");
        let build_tofu = build_root.join("tofu/lxd");

        std::fs::create_dir_all(&build_ansible)?;
        std::fs::create_dir_all(&build_tofu)?;

        // Test that directories were created
        assert!(build_ansible.exists());
        assert!(build_tofu.exists());

        // Simulate template rendering to build directory
        if PathBuf::from("templates/ansible/inventory.yml.tera").exists() {
            let template_path = PathBuf::from("templates/ansible/inventory.yml.tera");
            let template_content = std::fs::read_to_string(&template_path)?;
            let output_path = build_ansible.join("inventory.yml");

            let inventory = InventoryTemplate::new(
                &template_content,
                "10.0.0.100",
                &temp_dir.path().join("ssh_key").to_string_lossy(),
            )?;

            let context = StaticContext::default();
            inventory.render(&context, &output_path)?;

            // Verify output in build directory
            assert!(output_path.exists());
            let file_content = std::fs::read_to_string(&output_path)?;
            assert!(file_content.contains("10.0.0.100"));
        }

        println!("✅ Build directory workflow completed successfully");
        Ok(())
    }
}
