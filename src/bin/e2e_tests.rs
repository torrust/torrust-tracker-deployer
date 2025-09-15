use anyhow::{Context, Result};
use clap::Parser;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// Import command execution system
use torrust_tracker_deploy::commands::ProvisionCommand;
use torrust_tracker_deploy::config::{Config, SshCredentials};
use torrust_tracker_deploy::container::Services;
// Import steps
use torrust_tracker_deploy::steps::{
    InstallDockerComposeStep, InstallDockerStep, ValidateCloudInitCompletionStep,
    ValidateDockerComposeInstallationStep, ValidateDockerInstallationStep, WaitForCloudInitStep,
};

#[derive(Parser)]
#[command(name = "e2e-tests")]
#[command(about = "E2E tests for Torrust Tracker Deploy")]
struct Cli {
    /// Keep the test environment after completion
    #[arg(long)]
    keep: bool,

    /// Templates directory path (default: ./data/templates)
    #[arg(long, default_value = "./data/templates")]
    templates_dir: String,
}

/// Main test environment combining configuration and services
struct TestEnvironment {
    config: Config,
    services: Services,
    #[allow(dead_code)] // Kept to maintain temp directory lifetime for tests
    temp_dir: Option<tempfile::TempDir>,
}

impl TestEnvironment {
    fn new(keep_env: bool, templates_dir: String) -> Result<Self> {
        // Get project root (current directory when running from root)
        let project_root = std::env::current_dir()?;

        // Create temporary directory for SSH keys
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        // Setup SSH key
        let temp_ssh_key = temp_dir.path().join("testing_rsa");
        let temp_ssh_pub_key = temp_dir.path().join("testing_rsa.pub");
        Self::setup_ssh_key(&project_root, &temp_dir, &temp_ssh_key)?;

        // Create SSH credentials (no host IP needed yet)
        let ssh_credentials =
            SshCredentials::new(temp_ssh_key, temp_ssh_pub_key, "torrust".to_string());

        // Create main configuration
        let config = Config::new(
            keep_env,
            ssh_credentials,
            templates_dir,
            project_root.clone(),
            project_root.join("build"),
        );

        // Create services using the configuration
        let services = Services::new(&config);

        // Clean and prepare templates directory
        Self::clean_and_prepare_templates(&services)?;

        info!(
            environment = "temporary_directory",
            path = %temp_dir.path().display(),
            "Temporary directory created"
        );
        info!(
            environment = "templates_directory",
            path = %services.template_manager.templates_dir().display(),
            "Templates directory configured"
        );

        Ok(Self {
            config,
            services,
            temp_dir: Some(temp_dir),
        })
    }

    /// Setup SSH key by copying from fixtures to temporary directory with proper permissions
    fn setup_ssh_key(
        project_root: &std::path::Path,
        temp_dir: &TempDir,
        _ssh_key_path: &std::path::Path,
    ) -> Result<()> {
        // Copy SSH private key from fixtures to temp directory
        let fixtures_ssh_key = project_root.join("fixtures/testing_rsa");
        let temp_ssh_key = temp_dir.path().join("testing_rsa");

        std::fs::copy(&fixtures_ssh_key, &temp_ssh_key)
            .context("Failed to copy SSH private key to temporary directory")?;

        // Copy SSH public key from fixtures to temp directory
        let fixtures_ssh_pub_key = project_root.join("fixtures/testing_rsa.pub");
        let temp_ssh_pub_key = temp_dir.path().join("testing_rsa.pub");

        std::fs::copy(&fixtures_ssh_pub_key, &temp_ssh_pub_key)
            .context("Failed to copy SSH public key to temporary directory")?;

        // Set proper permissions on the SSH key (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_ssh_key)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&temp_ssh_key, perms)?;
        }

        info!(
            operation = "ssh_key_setup",
            private_location = %temp_ssh_key.display(),
            public_location = %temp_ssh_pub_key.display(),
            "SSH keys copied to temporary location"
        );

        Ok(())
    }

    /// Clean and prepare templates directory to ensure fresh embedded templates
    fn clean_and_prepare_templates(services: &Services) -> Result<()> {
        // Clean templates directory to ensure we use fresh templates from embedded resources
        info!(
            operation = "clean_templates",
            "Cleaning templates directory to ensure fresh embedded templates"
        );
        services
            .template_manager
            .reset_templates_dir()
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    async fn provision_infrastructure(&self) -> Result<IpAddr> {
        info!(
            stage = "infrastructure_provisioning",
            "Provisioning test infrastructure"
        );

        // Use the new ProvisionCommand to handle all infrastructure provisioning steps
        let provision_command = ProvisionCommand::new(
            Arc::clone(&self.services.tofu_template_renderer),
            Arc::clone(&self.services.ansible_template_renderer),
            Arc::clone(&self.services.ansible_client),
            Arc::clone(&self.services.opentofu_client),
            self.config.ssh_credentials.clone(),
        );

        let opentofu_instance_ip = provision_command
            .execute()
            .await
            .map_err(anyhow::Error::from)
            .context("Failed to provision infrastructure")?;

        // Get the instance IP from LXD client (keeping for comparison/validation)
        let lxd_instance_ip = self
            .get_instance_ip()
            .context("Failed to get instance IP from LXD client")?;

        info!(
            stage = "infrastructure_provisioning",
            status = "complete",
            opentofu_ip = %opentofu_instance_ip,
            lxd_ip = %lxd_instance_ip,
            "Infrastructure provisioned successfully"
        );

        // Return the IP from OpenTofu as it's our preferred source
        Ok(opentofu_instance_ip)
    }

    fn get_instance_ip(&self) -> Result<IpAddr> {
        // For E2E tests, we should rely on OpenTofu outputs since they already wait for network
        // This is a secondary validation that the instance is accessible via LXD

        // First, check if the instance exists
        let instance = self
            .services
            .lxd_client
            .get_instance_by_name("torrust-vm")
            .context("Failed to query LXD for instance information")?
            .ok_or_else(|| anyhow::anyhow!("Instance 'torrust-vm' was not found in LXD"))?;

        // Then, check if the instance has an IP address
        let ip = instance.ip_address.ok_or_else(|| {
            anyhow::anyhow!("Instance 'torrust-vm' exists but has no IPv4 address assigned")
        })?;

        Ok(ip)
    }

    fn cleanup(&self) {
        if self.config.keep_env {
            info!(
                operation = "cleanup",
                action = "keep_environment",
                instance = "torrust-vm",
                connect_command = "lxc exec torrust-vm -- /bin/bash",
                "Keeping test environment as requested"
            );
            return;
        }

        info!(operation = "cleanup", "Cleaning up test environment");

        // Destroy infrastructure using OpenTofuClient
        let result = self
            .services
            .opentofu_client
            .destroy(true) // auto_approve = true
            .map_err(anyhow::Error::from);

        match result {
            Ok(_) => info!(
                operation = "cleanup",
                status = "success",
                "Test environment cleaned up successfully"
            ),
            Err(e) => warn!(
                operation = "cleanup",
                status = "failed",
                error = %e,
                "Cleanup failed"
            ),
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.config.keep_env {
            // Try basic cleanup in case async cleanup failed
            // Using emergency_destroy for consistent OpenTofu handling
            let tofu_dir = self.config.build_dir.join(&self.config.opentofu_subfolder);

            drop(torrust_tracker_deploy::command_wrappers::opentofu::emergency_destroy(&tofu_dir));
        }
    }
}

async fn validate_deployment(env: &TestEnvironment, instance_ip: &IpAddr) -> Result<()> {
    info!(stage = "validation", "Starting deployment validation");

    // Validate cloud-init completion
    let validate_cloud_init_step = ValidateCloudInitCompletionStep::new(
        env.config.ssh_credentials.clone().with_host(*instance_ip),
    );
    validate_cloud_init_step
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker installation
    let validate_docker_step = ValidateDockerInstallationStep::new(
        env.config.ssh_credentials.clone().with_host(*instance_ip),
    );
    validate_docker_step
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker Compose installation
    let validate_docker_compose_step = ValidateDockerComposeInstallationStep::new(
        env.config.ssh_credentials.clone().with_host(*instance_ip),
    );
    validate_docker_compose_step
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    info!(
        stage = "validation",
        status = "success",
        "All deployment validations passed"
    );
    Ok(())
}

async fn run_full_deployment_test(env: &TestEnvironment) -> Result<IpAddr> {
    info!(
        test_type = "full_deployment",
        workflow = "template_based",
        stages = 3,
        "Starting full deployment E2E test"
    );

    // Stage 1: Provision infrastructure (includes template rendering, infrastructure creation, SSH wait, and Ansible template rendering)
    let instance_ip = env.provision_infrastructure().await?;

    // Stage 2: Wait for cloud-init completion (now that Ansible inventory has correct IP)
    let wait_cloud_init_step = WaitForCloudInitStep::new(env.services.ansible_client.clone());
    wait_cloud_init_step
        .execute()
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to wait for cloud-init completion")?;

    // Stage 3: Run Ansible playbooks from build directory
    // Install Docker using the step
    let install_docker_step = InstallDockerStep::new(env.services.ansible_client.clone());
    install_docker_step
        .execute()
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to install Docker")?;

    // Run the install-docker-compose playbook
    info!(
        step = 3,
        action = "install_docker_compose",
        "Installing Docker Compose"
    );
    InstallDockerComposeStep::new(Arc::clone(&env.services.ansible_client))
        .execute()
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to install Docker Compose")?;

    info!(
        stage = "deployment",
        status = "success",
        "Deployment stages completed successfully"
    );

    info!(
        test_type = "full_deployment",
        status = "success",
        note = "Docker/Docker Compose installation status varies based on network connectivity",
        "Full deployment E2E test completed successfully"
    );

    // Return the instance IP for validation in main
    Ok(instance_ip)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber with proper configuration for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    info!(
        application = "torrust_tracker_deploy",
        test_suite = "e2e_tests",
        "Starting E2E tests"
    );

    let env = TestEnvironment::new(cli.keep, cli.templates_dir)?;

    let test_start = Instant::now();

    let result = run_full_deployment_test(&env).await;

    // Handle deployment results and run validation if deployment succeeded
    let validation_result = match result {
        Ok(instance_ip) => validate_deployment(&env, &instance_ip).await,
        Err(deployment_err) => {
            error!(
                stage = "deployment",
                status = "failed",
                error = %deployment_err,
                "Deployment failed"
            );
            Err(deployment_err)
        }
    };

    env.cleanup();

    let test_duration = test_start.elapsed();
    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Test execution completed"
    );

    // Handle final results
    match validation_result {
        Ok(()) => {
            info!(
                test_suite = "e2e_tests",
                status = "success",
                "All tests passed and cleanup completed successfully"
            );
            Ok(())
        }
        Err(test_err) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %test_err,
                "Test failed"
            );
            Err(test_err)
        }
    }
}
