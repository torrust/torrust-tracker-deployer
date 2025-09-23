//! Integration tests for the template system
//!
//! These tests verify that the template system works with real template files
//! and validates the complete workflow without actually provisioning infrastructure.

use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;
use torrust_tracker_deploy::domain::template::file::File;
use torrust_tracker_deploy::infrastructure::template::wrappers::ansible::inventory::{
    AnsibleHost, InventoryContext, InventoryTemplate, SshPrivateKeyFile,
};

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
        let template_file = File::new("inventory.yml.tera", template_content.clone()).unwrap();
        let host = AnsibleHost::from_str("192.168.1.100")?;
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/testing_rsa")?;
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()?;
        let inventory = InventoryTemplate::new(&template_file, inventory_context)?;

        // Render the template
        inventory.render(&output_path)?;

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
        assert!(file_content.contains("torrust-tracker-vm:"));
        assert!(file_content.contains("ansible_user: torrust"));

        println!("✅ Real inventory template rendered successfully");
        Ok(())
    }

    /// Test variable validation with real template
    #[test]
    fn test_real_template_variable_validation() -> Result<()> {
        let template_path = PathBuf::from("templates/ansible/inventory.yml.tera");

        // Skip test if template file doesn't exist
        if !template_path.exists() {
            println!("Skipping test: inventory template not found");
            return Ok(());
        }

        // Read the template content
        let Ok(template_content) = std::fs::read_to_string(&template_path) else {
            println!("Skipping test: Could not read template file");
            return Ok(());
        };

        // Test that valid variables are accepted
        let template_file = File::new("inventory.yml.tera", template_content.clone()).unwrap();
        let host = AnsibleHost::from_str("127.0.0.1")?;
        let ssh_key = SshPrivateKeyFile::new("/path/to/key")?;
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()?;
        let result = InventoryTemplate::new(&template_file, inventory_context);

        // Construction should succeed with valid IP and SSH key path
        assert!(result.is_ok());

        // Test that invalid IP address is rejected
        let result = AnsibleHost::from_str("invalid.ip.address");
        assert!(result.is_err());

        // Test that empty SSH key path is rejected
        let result = SshPrivateKeyFile::new("");
        assert!(result.is_err());

        // Test that invalid template content fails
        let invalid_content = "invalid template content without required variables";
        let invalid_template_file =
            File::new("inventory.yml.tera", invalid_content.to_string()).unwrap();
        let host = AnsibleHost::from_str("192.168.1.100")?;
        let ssh_key = SshPrivateKeyFile::new("/path/to/key")?;
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()?;
        let result = InventoryTemplate::new(&invalid_template_file, inventory_context.clone());

        // Static templates are now valid - they just don't use template variables
        assert!(result.is_ok());

        // Test that templates with undefined variables fail
        let undefined_var_content = "server ansible_host={{undefined_variable}}\n";
        let undefined_template_file =
            File::new("inventory.yml.tera", undefined_var_content.to_string()).unwrap();
        let result = InventoryTemplate::new(&undefined_template_file, inventory_context);

        assert!(result.is_err());
        println!("✅ Template with undefined variables correctly rejected");

        Ok(())
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
            let template_file = File::new("inventory.yml.tera", original_content.clone()).unwrap();
            let host = AnsibleHost::from_str(&format!("192.168.1.{i}"))?;
            let ssh_key = SshPrivateKeyFile::new(format!("/home/user{i}/.ssh/key"))?;
            let inventory_context = InventoryContext::builder()
                .with_host(host)
                .with_ssh_priv_key_path(ssh_key)
                .build()?;
            let inventory = InventoryTemplate::new(&template_file, inventory_context)?;

            inventory.render(&output_path)?;
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
            let template_file = File::new("inventory.yml.tera", template_content.clone()).unwrap();
            let output_path = build_ansible.join("inventory.yml");

            let host = AnsibleHost::from_str("10.0.0.100")?;
            let ssh_key =
                SshPrivateKeyFile::new(temp_dir.path().join("ssh_key").to_string_lossy().as_ref())?;
            let inventory_context = InventoryContext::builder()
                .with_host(host)
                .with_ssh_priv_key_path(ssh_key)
                .build()?;
            let inventory = InventoryTemplate::new(&template_file, inventory_context)?;

            inventory.render(&output_path)?;

            // Verify output in build directory
            assert!(output_path.exists());
            let file_content = std::fs::read_to_string(&output_path)?;
            assert!(file_content.contains("10.0.0.100"));
        }

        println!("✅ Build directory workflow completed successfully");
        Ok(())
    }
}
