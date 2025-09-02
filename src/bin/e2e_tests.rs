use anyhow::{anyhow, Context, Result};
use clap::Parser;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "e2e-tests")]
#[command(about = "E2E tests for Torrust Testing Infrastructure")]
struct Cli {
    /// Test to run
    #[arg(value_enum)]
    test: TestType,
    
    /// Keep the test environment after completion
    #[arg(long)]
    keep: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum TestType {
    WaitCloudInit,
}

struct TestEnvironment {
    project_root: PathBuf,
    keep_env: bool,
    verbose: bool,
}

impl TestEnvironment {
    fn new(keep_env: bool, verbose: bool) -> Result<Self> {
        // Get project root (current directory when running from root)
        let project_root = std::env::current_dir()?;
        
        Ok(Self {
            project_root,
            keep_env,
            verbose,
        })
    }
    
    async fn run_command(&self, cmd: &str, args: &[&str], working_dir: Option<&Path>) -> Result<String> {
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
    
    async fn provision_infrastructure(&self) -> Result<String> {
        println!("🚀 Provisioning test infrastructure...");
        
        // First, we need to update the container name in the OpenTofu config
        // For now, we'll use the existing config but this means tests might conflict
        // TODO: Make this configurable with variables later
        
        let tofu_dir = self.project_root.join("config/tofu/lxd");
        
        // Initialize OpenTofu
        println!("   Initializing OpenTofu...");
        self.run_command("tofu", &["init"], Some(&tofu_dir)).await
            .context("Failed to initialize OpenTofu")?;
        
        // Apply infrastructure
        println!("   Applying infrastructure...");
        self.run_command("tofu", &["apply", "-auto-approve"], Some(&tofu_dir)).await
            .context("Failed to apply OpenTofu configuration")?;
        
        // Get the container IP
        let container_ip = self.get_container_ip().await
            .context("Failed to get container IP after provisioning")?;
        
        println!("✅ Infrastructure provisioned successfully");
        println!("   Container IP: {}", container_ip);
        
        Ok(container_ip)
    }
    
    async fn get_container_ip(&self) -> Result<String> {
        // Get container information
        let output = self.run_command("lxc", &["list", "torrust-vm", "--format=json"], None).await
            .context("Failed to list LXC containers")?;
        
        let containers: Value = serde_json::from_str(&output)
            .context("Failed to parse LXC list output")?;
        
        let container = containers.as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow!("No container found"))?;
        
        let ip = container["state"]["network"]["eth0"]["addresses"]
            .as_array()
            .and_then(|addresses| {
                addresses.iter().find(|addr| {
                    addr["family"].as_str() == Some("inet")
                })
            })
            .and_then(|addr| addr["address"].as_str())
            .ok_or_else(|| anyhow!("Could not find IPv4 address for container"))?;
        
        Ok(ip.to_string())
    }
    
    async fn wait_for_ssh_connectivity(&self, ip: &str) -> Result<()> {
        println!("🔌 Waiting for SSH connectivity...");
        
        let ssh_key = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?
            .join(".ssh/testing_rsa");
        
        let max_attempts = 30;
        let mut attempt = 0;
        
        while attempt < max_attempts {
            let result = Command::new("ssh")
                .args(&[
                    "-i", ssh_key.to_str().unwrap(),
                    "-o", "StrictHostKeyChecking=no",
                    "-o", "UserKnownHostsFile=/dev/null",
                    "-o", "ConnectTimeout=5",
                    &format!("torrust@{}", ip),
                    "echo 'SSH connected'"
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
                println!("   Still waiting for SSH... (attempt {}/{})", attempt + 1, max_attempts);
            }
            
            sleep(Duration::from_secs(2)).await;
            attempt += 1;
        }
        
        Err(anyhow!("SSH connectivity could not be established after {} attempts", max_attempts))
    }
    
    async fn update_ansible_inventory(&self, container_ip: &str) -> Result<()> {
        println!("📝 Updating Ansible inventory...");
        
        let inventory_path = self.project_root.join("config/ansible/inventory.yml");
        let inventory_content = tokio::fs::read_to_string(&inventory_path).await
            .context("Failed to read inventory file")?;
        
        // Replace the IP address in the inventory
        let ip_regex = Regex::new(r"ansible_host: \d+\.\d+\.\d+\.\d+")
            .context("Failed to create IP regex")?;
        
        let updated_content = ip_regex.replace(
            &inventory_content,
            &format!("ansible_host: {}", container_ip)
        );
        
        tokio::fs::write(&inventory_path, updated_content.as_ref()).await
            .context("Failed to write updated inventory file")?;
        
        println!("✅ Ansible inventory updated with IP: {}", container_ip);
        Ok(())
    }
    
    async fn run_ansible_playbook(&self, playbook: &str) -> Result<()> {
        println!("🎭 Running Ansible playbook: {}", playbook);
        
        let ansible_dir = self.project_root.join("config/ansible");
        let playbook_path = format!("{}.yml", playbook);
        
        let mut args = vec!["ansible-playbook", &playbook_path];
        if self.verbose {
            args.push("-vvv");
        }
        
        self.run_command("ansible-playbook", &[&playbook_path], Some(&ansible_dir)).await
            .context(format!("Failed to run Ansible playbook: {}", playbook))?;
        
        println!("✅ Ansible playbook executed successfully");
        Ok(())
    }
    
    async fn validate_cloud_init_completion(&self, container_ip: &str) -> Result<()> {
        println!("🔍 Validating cloud-init completion...");
        
        let ssh_key = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?
            .join(".ssh/testing_rsa");
        
        // Check cloud-init status
        let output = Command::new("ssh")
            .args(&[
                "-i", ssh_key.to_str().unwrap(),
                "-o", "StrictHostKeyChecking=no",
                "-o", "UserKnownHostsFile=/dev/null",
                &format!("torrust@{}", container_ip),
                "cloud-init status"
            ])
            .output()
            .context("Failed to check cloud-init status")?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to execute cloud-init status command"));
        }
        
        let status_output = String::from_utf8_lossy(&output.stdout);
        if !status_output.contains("status: done") {
            return Err(anyhow!("Cloud-init status is not 'done': {}", status_output));
        }
        
        // Check for completion marker file
        let marker_check = Command::new("ssh")
            .args(&[
                "-i", ssh_key.to_str().unwrap(),
                "-o", "StrictHostKeyChecking=no",
                "-o", "UserKnownHostsFile=/dev/null",
                &format!("torrust@{}", container_ip),
                "test -f /var/lib/cloud/instance/boot-finished"
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
    
    async fn cleanup(&self) -> Result<()> {
        if self.keep_env {
            println!("🔒 Keeping test environment as requested");
            println!("   Container: torrust-vm");
            println!("   Connect with: lxc exec torrust-vm -- /bin/bash");
            return Ok(());
        }
        
        println!("🧹 Cleaning up test environment...");
        
        let tofu_dir = self.project_root.join("config/tofu/lxd");
        
        // Destroy infrastructure
        let result = self.run_command("tofu", &["destroy", "-auto-approve"], Some(&tofu_dir)).await;
        
        match result {
            Ok(_) => println!("✅ Test environment cleaned up successfully"),
            Err(e) => println!("⚠️  Warning: Cleanup failed: {}", e),
        }
        
        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.keep_env {
            // Try basic cleanup in case async cleanup failed
            let tofu_dir = self.project_root.join("config/tofu/lxd");
            let _ = Command::new("tofu")
                .args(&["destroy", "-auto-approve"])
                .current_dir(&tofu_dir)
                .output();
        }
    }
}

async fn test_wait_cloud_init(env: &TestEnvironment) -> Result<()> {
    println!("🧪 Starting wait-cloud-init E2E test");
    
    // 1. Provision infrastructure
    let container_ip = env.provision_infrastructure().await?;
    
    // 2. Wait for SSH connectivity
    env.wait_for_ssh_connectivity(&container_ip).await?;
    
    // 3. Update Ansible inventory
    env.update_ansible_inventory(&container_ip).await?;
    
    // 4. Run the wait-cloud-init playbook
    env.run_ansible_playbook("wait-cloud-init").await?;
    
    // 5. Validate cloud-init completion
    env.validate_cloud_init_completion(&container_ip).await?;
    
    println!("🎉 wait-cloud-init E2E test completed successfully!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("🚀 Torrust Testing Infrastructure E2E Tests");
    println!("===========================================");
    
    let env = TestEnvironment::new(cli.keep, cli.verbose)?;
    
    let test_start = Instant::now();
    
    let result = match cli.test {
        TestType::WaitCloudInit => test_wait_cloud_init(&env).await,
    };
    
    let cleanup_result = env.cleanup().await;
    
    let test_duration = test_start.elapsed();
    println!("\n📊 Test execution time: {:?}", test_duration);
    
    // Handle results
    match (result, cleanup_result) {
        (Ok(()), Ok(())) => {
            println!("✅ All tests passed and cleanup completed successfully");
            Ok(())
        }
        (Ok(()), Err(cleanup_err)) => {
            println!("✅ Tests passed but cleanup failed: {}", cleanup_err);
            Ok(()) // Don't fail the test due to cleanup issues
        }
        (Err(test_err), _) => {
            println!("❌ Test failed: {}", test_err);
            Err(test_err)
        }
    }
}
