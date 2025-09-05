use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::sleep;

// Import template system
use torrust_tracker_deploy::template::wrappers::ansible::inventory::InventoryTemplate;
use torrust_tracker_deploy::template::{StaticContext, TemplateRenderer};

#[derive(Parser)]
#[command(name = "e2e-tests")]
#[command(about = "E2E tests for Torrust Tracker Deploy")]
struct Cli {
    /// Keep the test environment after completion
    #[arg(long)]
    keep: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

struct TestEnvironment {
    project_root: PathBuf,
    #[allow(dead_code)] // Will be used in template rendering
    build_dir: PathBuf,
    keep_env: bool,
    verbose: bool,
    ssh_key_path: PathBuf,
    #[allow(dead_code)] // Kept to maintain temp directory lifetime
    temp_dir: Option<tempfile::TempDir>,
    #[allow(dead_code)] // Used for cleanup but not directly accessed
    original_inventory: Option<String>,
}

impl TestEnvironment {
    fn new(keep_env: bool, verbose: bool) -> Result<Self> {
        // Get project root (current directory when running from root)
        let project_root = std::env::current_dir()?;

        // Create temporary directory for SSH keys
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        // Copy SSH private key from fixtures to temp directory
        let fixtures_ssh_key = project_root.join("fixtures/testing_rsa");
        let temp_ssh_key = temp_dir.path().join("testing_rsa");

        std::fs::copy(&fixtures_ssh_key, &temp_ssh_key)
            .context("Failed to copy SSH private key to temporary directory")?;

        // Set proper permissions on the SSH key (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_ssh_key)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&temp_ssh_key, perms)?;
        }

        if verbose {
            println!(
                "🔑 SSH key copied to temporary location: {}",
                temp_ssh_key.display()
            );
            println!("📁 Temporary directory: {}", temp_dir.path().display());
        }

        Ok(Self {
            build_dir: project_root.join("build"),
            project_root,
            keep_env,
            verbose,
            ssh_key_path: temp_ssh_key,
            temp_dir: Some(temp_dir),
            original_inventory: None,
        })
    }

    /// Stage 1: Render static templates (tofu) to build/tofu/ directory
    async fn render_static_templates(&self) -> Result<()> {
        println!("🏗️  Stage 1: Rendering static templates to build directory...");

        // Create build directory structure
        let build_tofu_dir = self.build_dir.join("tofu/lxd");
        tokio::fs::create_dir_all(&build_tofu_dir)
            .await
            .context("Failed to create build/tofu/lxd directory")?;

        // Copy static tofu templates (no variables for now)
        let templates_tofu_dir = self.project_root.join("templates/tofu/lxd");

        // Copy main.tf
        let source_main_tf = templates_tofu_dir.join("main.tf");
        let dest_main_tf = build_tofu_dir.join("main.tf");
        tokio::fs::copy(&source_main_tf, &dest_main_tf)
            .await
            .context("Failed to copy main.tf to build directory")?;

        // Copy cloud-init.yml
        let source_cloud_init = templates_tofu_dir.join("cloud-init.yml");
        let dest_cloud_init = build_tofu_dir.join("cloud-init.yml");
        tokio::fs::copy(&source_cloud_init, &dest_cloud_init)
            .await
            .context("Failed to copy cloud-init.yml to build directory")?;

        if self.verbose {
            println!(
                "   ✅ Static templates copied to: {}",
                build_tofu_dir.display()
            );
        }

        println!("✅ Stage 1 complete: Static templates ready");
        Ok(())
    }

    /// Stage 3: Render ansible templates with runtime variables to build/ansible/
    async fn render_runtime_templates(&self, container_ip: &str) -> Result<()> {
        println!("🎭 Stage 3: Rendering runtime templates with variables...");

        // Create build directory structure
        let build_ansible_dir = self.build_dir.join("ansible");
        tokio::fs::create_dir_all(&build_ansible_dir)
            .await
            .context("Failed to create build/ansible directory")?;

        // Render inventory.yml.tera with runtime variables
        let inventory_template_path = self
            .project_root
            .join("templates/ansible/inventory.yml.tera");
        let inventory_output_path = build_ansible_dir.join("inventory.yml");

        let inventory_template_content = std::fs::read_to_string(&inventory_template_path)
            .context("Failed to read inventory template file")?;

        let inventory_template = InventoryTemplate::new(
            &inventory_template_content,
            container_ip,
            &self.ssh_key_path.to_string_lossy(),
        )
        .context("Failed to create InventoryTemplate")?;

        let static_context = StaticContext::default();
        inventory_template
            .render(&static_context, &inventory_output_path)
            .context("Failed to render inventory template")?;

        // Copy static ansible files
        let templates_ansible_dir = self.project_root.join("templates/ansible");

        // Copy ansible.cfg
        let source_cfg = templates_ansible_dir.join("ansible.cfg");
        let dest_cfg = build_ansible_dir.join("ansible.cfg");
        tokio::fs::copy(&source_cfg, &dest_cfg)
            .await
            .context("Failed to copy ansible.cfg to build directory")?;

        // Copy playbooks
        for playbook in &[
            "install-docker.yml",
            "install-docker-compose.yml",
            "wait-cloud-init.yml",
        ] {
            let source_playbook = templates_ansible_dir.join(playbook);
            let dest_playbook = build_ansible_dir.join(playbook);
            tokio::fs::copy(&source_playbook, &dest_playbook)
                .await
                .with_context(|| format!("Failed to copy {playbook} to build directory"))?;
        }

        if self.verbose {
            println!(
                "   ✅ Runtime templates rendered to: {}",
                build_ansible_dir.display()
            );
            println!("   ✅ Inventory rendered with IP: {container_ip}");
            println!(
                "   ✅ Inventory rendered with SSH key: {}",
                self.ssh_key_path.display()
            );
        }

        println!("✅ Stage 3 complete: Runtime templates ready");
        Ok(())
    }

    fn run_command(&self, cmd: &str, args: &[&str], working_dir: Option<&Path>) -> Result<String> {
        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        if self.verbose {
            println!("🔧 Running: {} {}", cmd, args.join(" "));
            if let Some(dir) = working_dir {
                println!("   Working directory: {}", dir.display());
            }
        }

        let output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow!(
                "Command failed: {} {}\nStdout: {}\nStderr: {}",
                cmd,
                args.join(" "),
                stdout,
                stderr
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn provision_infrastructure(&self) -> Result<String> {
        println!("🚀 Stage 2: Provisioning test infrastructure...");

        // Use the build directory instead of config directory
        let tofu_dir = self.build_dir.join("tofu/lxd");

        // Initialize OpenTofu
        println!("   Initializing OpenTofu...");
        self.run_command("tofu", &["init"], Some(&tofu_dir))
            .context("Failed to initialize OpenTofu")?;

        // Apply infrastructure
        println!("   Applying infrastructure...");
        self.run_command("tofu", &["apply", "-auto-approve"], Some(&tofu_dir))
            .context("Failed to apply OpenTofu configuration")?;

        // Get the container IP
        let container_ip = self
            .get_container_ip()
            .context("Failed to get container IP after provisioning")?;

        println!("✅ Stage 2 complete: Infrastructure provisioned");
        println!("   Container IP: {container_ip}");

        Ok(container_ip)
    }

    fn get_container_ip(&self) -> Result<String> {
        // Get container information
        let output = self
            .run_command("lxc", &["list", "torrust-vm", "--format=json"], None)
            .context("Failed to list LXC containers")?;

        let containers: Value =
            serde_json::from_str(&output).context("Failed to parse LXC list output")?;

        let container = containers
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow!("No container found"))?;

        let ip = container["state"]["network"]["eth0"]["addresses"]
            .as_array()
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr["family"].as_str() == Some("inet"))
            })
            .and_then(|addr| addr["address"].as_str())
            .ok_or_else(|| anyhow!("Could not find IPv4 address for container"))?;

        Ok(ip.to_string())
    }

    async fn wait_for_ssh_connectivity(&self, ip: &str) -> Result<()> {
        println!("🔌 Waiting for SSH connectivity...");

        let max_attempts = 30;
        let mut attempt = 0;

        while attempt < max_attempts {
            let result = Command::new("ssh")
                .args([
                    "-i",
                    self.ssh_key_path.to_str().unwrap(),
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    "-o",
                    "ConnectTimeout=5",
                    &format!("torrust@{ip}"),
                    "echo 'SSH connected'",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if let Ok(status) = result {
                if status.success() {
                    println!("✅ SSH connectivity established");
                    return Ok(());
                }
            }

            if attempt % 5 == 0 {
                println!(
                    "   Still waiting for SSH... (attempt {}/{})",
                    attempt + 1,
                    max_attempts
                );
            }

            sleep(Duration::from_secs(2)).await;
            attempt += 1;
        }

        Err(anyhow!(
            "SSH connectivity could not be established after {} attempts",
            max_attempts
        ))
    }

    fn run_ansible_playbook(&self, playbook: &str) -> Result<()> {
        println!("🎭 Stage 4: Running Ansible playbook: {playbook}");

        let ansible_dir = self.build_dir.join("ansible");
        let playbook_path = format!("{playbook}.yml");

        let mut args = vec!["ansible-playbook", &playbook_path];
        if self.verbose {
            args.push("-vvv");
        }

        self.run_command("ansible-playbook", &[&playbook_path], Some(&ansible_dir))
            .context(format!("Failed to run Ansible playbook: {playbook}"))?;

        println!("✅ Stage 4: Ansible playbook executed successfully");
        Ok(())
    }

    fn validate_cloud_init_completion(&self, container_ip: &str) -> Result<()> {
        println!("🔍 Validating cloud-init completion...");

        // Check cloud-init status
        let output = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "cloud-init status",
            ])
            .output()
            .context("Failed to check cloud-init status")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to execute cloud-init status command"));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        if !status_output.contains("status: done") {
            return Err(anyhow!(
                "Cloud-init status is not 'done': {}",
                status_output
            ));
        }

        // Check for completion marker file
        let marker_check = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "test -f /var/lib/cloud/instance/boot-finished",
            ])
            .status()
            .context("Failed to check cloud-init completion marker")?;

        if !marker_check.success() {
            return Err(anyhow!("Cloud-init completion marker file not found"));
        }

        println!("✅ Cloud-init validation passed");
        println!("   ✓ Cloud-init status is 'done'");
        println!("   ✓ Completion marker file exists");
        Ok(())
    }

    fn validate_docker_installation(&self, container_ip: &str) -> Result<()> {
        println!("🔍 Validating Docker installation...");

        // Check Docker version
        let output = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "docker --version",
            ])
            .output()
            .context("Failed to check Docker version")?;

        if !output.status.success() {
            return Err(anyhow!("Docker is not installed or not accessible"));
        }

        let docker_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("✅ Docker installation validated");
        println!("   ✓ Docker version: {docker_version}");

        // Check Docker daemon status
        let daemon_check = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "sudo systemctl is-active docker",
            ])
            .output()
            .context("Failed to check Docker daemon status")?;

        if !daemon_check.status.success() {
            return Err(anyhow!("Docker daemon is not running"));
        }

        println!("   ✓ Docker daemon is active");
        Ok(())
    }

    fn validate_docker_compose_installation(&self, container_ip: &str) -> Result<()> {
        println!("🔍 Validating Docker Compose installation...");

        // Check Docker Compose version
        let output = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "docker-compose --version",
            ])
            .output()
            .context("Failed to check Docker Compose version")?;

        if !output.status.success() {
            return Err(anyhow!("Docker Compose is not installed or not accessible"));
        }

        let compose_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("✅ Docker Compose installation validated");
        println!("   ✓ Docker Compose version: {compose_version}");

        // Test basic docker-compose functionality with a simple test file
        let test_compose_content = r"services:
  test:
    image: hello-world
";

        // Create a temporary test docker-compose.yml file
        let create_test_file = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                &format!("echo '{test_compose_content}' > /tmp/test-docker-compose.yml"),
            ])
            .status()
            .context("Failed to create test docker-compose.yml")?;

        if !create_test_file.success() {
            return Err(anyhow!("Failed to create test docker-compose.yml file"));
        }

        // Validate docker-compose file
        let validate_compose = Command::new("ssh")
            .args([
                "-i",
                self.ssh_key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("torrust@{container_ip}"),
                "cd /tmp && docker-compose -f test-docker-compose.yml config",
            ])
            .status()
            .context("Failed to validate docker-compose configuration")?;

        if !validate_compose.success() {
            return Err(anyhow!("Docker Compose configuration validation failed"));
        }

        // Clean up test file
        drop(
            Command::new("ssh")
                .args([
                    "-i",
                    self.ssh_key_path.to_str().unwrap(),
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    &format!("torrust@{container_ip}"),
                    "rm -f /tmp/test-docker-compose.yml",
                ])
                .status(),
        );

        println!("   ✓ Docker Compose configuration validation passed");
        Ok(())
    }

    fn cleanup(&self) {
        if self.keep_env {
            println!("🔒 Keeping test environment as requested");
            println!("   Container: torrust-vm");
            println!("   Connect with: lxc exec torrust-vm -- /bin/bash");
            return;
        }

        println!("🧹 Cleaning up test environment...");

        let tofu_dir = self.build_dir.join("tofu/lxd");

        // Destroy infrastructure
        let result = self.run_command("tofu", &["destroy", "-auto-approve"], Some(&tofu_dir));

        match result {
            Ok(_) => println!("✅ Test environment cleaned up successfully"),
            Err(e) => println!("⚠️  Warning: Cleanup failed: {e}"),
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.keep_env {
            // Try basic cleanup in case async cleanup failed
            let tofu_dir = self.build_dir.join("tofu/lxd");
            drop(
                Command::new("tofu")
                    .args(["destroy", "-auto-approve"])
                    .current_dir(&tofu_dir)
                    .output(),
            );
        }
    }
}

async fn run_full_deployment_test(env: &TestEnvironment) -> Result<()> {
    println!("🧪 Starting full deployment E2E test with template-based workflow");
    println!("   This will test the complete 4-stage template system:");
    println!("   Stage 1: Render static templates to build/");
    println!("   Stage 2: Provision VM with OpenTofu from build/");
    println!("   Stage 3: Render runtime templates with variables");
    println!("   Stage 4: Run Ansible playbooks from build/");
    println!();

    // Stage 1: Render static templates to build/tofu/
    env.render_static_templates().await?;

    // Stage 2: Provision infrastructure from build directory
    let container_ip = env.provision_infrastructure()?;

    // Wait for SSH connectivity
    env.wait_for_ssh_connectivity(&container_ip).await?;

    // Stage 3: Render ansible templates with runtime variables
    env.render_runtime_templates(&container_ip).await?;

    // Stage 4: Run Ansible playbooks from build directory
    println!("📋 Step 1: Waiting for cloud-init completion...");
    env.run_ansible_playbook("wait-cloud-init")?;

    // Validate cloud-init completion
    env.validate_cloud_init_completion(&container_ip)?;

    // Run the install-docker playbook
    println!("📋 Step 2: Installing Docker...");
    env.run_ansible_playbook("install-docker")?;

    // 7. Validate Docker installation
    env.validate_docker_installation(&container_ip)?;

    // 8. Run the install-docker-compose playbook
    println!("📋 Step 3: Installing Docker Compose...");
    env.run_ansible_playbook("install-docker-compose")?;

    // 9. Validate Docker Compose installation
    env.validate_docker_compose_installation(&container_ip)?;

    println!("🎉 Full deployment E2E test completed successfully!");
    println!("   ✅ Cloud-init setup completed");
    println!("   ✅ Docker installed and running");
    println!("   ✅ Docker Compose installed and functional");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🚀 Torrust Tracker Deploy E2E Tests");
    println!("===========================================");

    let env = TestEnvironment::new(cli.keep, cli.verbose)?;

    let test_start = Instant::now();

    let result = run_full_deployment_test(&env).await;

    env.cleanup();

    let test_duration = test_start.elapsed();
    println!("\n📊 Test execution time: {test_duration:?}");

    // Handle results
    match result {
        Ok(()) => {
            println!("✅ All tests passed and cleanup completed successfully");
            Ok(())
        }
        Err(test_err) => {
            println!("❌ Test failed: {test_err}");
            Err(test_err)
        }
    }
}
