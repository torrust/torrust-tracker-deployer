use anyhow::{Context, Result};
use clap::Parser;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

// Import command execution system
use torrust_tracker_deploy::command_wrappers::ssh::SshClient;
use torrust_tracker_deploy::config::{Config, SshConfig};
use torrust_tracker_deploy::container::Services;
// Import template system
use torrust_tracker_deploy::template::wrappers::ansible::inventory::{
    AnsibleHost, InventoryContext, SshPrivateKeyFile,
};
// Import remote actions
use torrust_tracker_deploy::actions::{
    CloudInitValidator, DockerComposeValidator, DockerValidator, RemoteAction,
};
// Import steps
use torrust_tracker_deploy::steps::RenderOpenTofuTemplatesStep;

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
        Self::setup_ssh_key(&project_root, &temp_dir, &temp_ssh_key)?;

        // Create configuration
        let ssh_config = SshConfig::new(temp_ssh_key, "torrust".to_string());
        let config = Config::new(
            keep_env,
            ssh_config,
            templates_dir,
            project_root.clone(),
            project_root.join("build"),
        );

        // Create services using the configuration
        let services = Services::new(&config);

        // Clean and prepare templates directory
        Self::clean_and_prepare_templates(&services)?;

        info!("📁 Temporary directory: {}", temp_dir.path().display());
        info!(
            "📄 Templates directory: {}",
            services.template_manager.templates_dir().display()
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
        ssh_key_path: &std::path::Path,
    ) -> Result<()> {
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

        info!(
            "🔑 SSH key copied to temporary location: {}",
            ssh_key_path.display()
        );

        Ok(())
    }

    /// Clean and prepare templates directory to ensure fresh embedded templates
    fn clean_and_prepare_templates(services: &Services) -> Result<()> {
        // Clean templates directory to ensure we use fresh templates from embedded resources
        info!("🧹 Cleaning templates directory to ensure fresh embedded templates...");
        services
            .template_manager
            .reset_templates_dir()
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    /// Stage 1: Render provision templates (`OpenTofu`) to build/tofu/ directory
    async fn render_provision_templates(&self) -> Result<()> {
        let step =
            RenderOpenTofuTemplatesStep::new(Arc::clone(&self.services.tofu_template_renderer));
        step.execute()
            .await
            .with_context(|| "Failed to render provision templates")
    }

    /// Stage 3: Render configuration templates (`Ansible`) with runtime variables to build/ansible/
    async fn render_configuration_templates(&self, instance_ip: &IpAddr) -> Result<()> {
        // Create inventory context with runtime variables
        let inventory_context = {
            let host = AnsibleHost::from(*instance_ip);
            let ssh_key = SshPrivateKeyFile::new(
                self.config
                    .ssh_config
                    .ssh_key_path
                    .to_string_lossy()
                    .as_ref(),
            )
            .context("Failed to parse SSH key path")?;

            InventoryContext::builder()
                .with_host(host)
                .with_ssh_priv_key_path(ssh_key)
                .build()
                .context("Failed to create InventoryContext")?
        };

        // Use the configuration renderer to handle all template rendering
        self.services
            .ansible_template_renderer
            .render(&self.services.template_manager, &inventory_context)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn run_ansible_playbook(&self, playbook: &str) -> Result<()> {
        info!("🎭 Stage 4: Running Ansible playbook: {playbook}");

        self.services
            .ansible_client
            .run_playbook(playbook)
            .context(format!("Failed to run Ansible playbook: {playbook}"))?;

        info!("✅ Stage 4: Ansible playbook executed successfully");
        Ok(())
    }

    fn provision_infrastructure(&self) -> Result<IpAddr> {
        info!("🚀 Stage 2: Provisioning test infrastructure...");

        // Initialize OpenTofu
        info!("   Initializing OpenTofu...");
        self.services
            .opentofu_client
            .init()
            .map_err(anyhow::Error::from)
            .context("Failed to initialize OpenTofu")?;

        // Apply infrastructure
        info!("   Applying infrastructure...");
        self.services
            .opentofu_client
            .apply(true) // auto_approve = true
            .map_err(anyhow::Error::from)
            .context("Failed to apply OpenTofu configuration")?;

        // Get the instance IP from OpenTofu outputs
        // NOTE: We prefer OpenTofu outputs over provider-specific methods because:
        // - If we add more providers (different than LXD) in the future, we have two options:
        //   1. Use the method that each provider provides to get the IP
        //   2. Use OpenTofu for all of them, so the OpenTofu output has a contract with this app.
        //      It has to return always the instance info we expect.
        // Using OpenTofu outputs provides a consistent interface across all providers.
        let opentofu_instance_info = self
            .services
            .opentofu_client
            .get_instance_info()
            .map_err(anyhow::Error::from)
            .context("Failed to get container info from OpenTofu outputs")?;

        let opentofu_instance_ip = opentofu_instance_info.ip_address;

        // Get the instance IP from LXD client (keeping for comparison/validation)
        let lxd_instance_ip = self
            .get_instance_ip()
            .context("Failed to get instance IP from LXD client")?;

        info!("✅ Stage 2 complete: Infrastructure provisioned");
        info!("   Instance IP from OpenTofu: {opentofu_instance_ip}");
        info!("   Instance IP from LXD client: {lxd_instance_ip}");

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
            info!("🔒 Keeping test environment as requested");
            info!("   Instance: torrust-vm");
            info!("   Connect with: lxc exec torrust-vm -- /bin/bash");
            return;
        }

        info!("🧹 Cleaning up test environment...");

        // Destroy infrastructure using OpenTofuClient
        let result = self
            .services
            .opentofu_client
            .destroy(true) // auto_approve = true
            .map_err(anyhow::Error::from);

        match result {
            Ok(_) => info!("✅ Test environment cleaned up successfully"),
            Err(e) => warn!("⚠️  Warning: Cleanup failed: {e}"),
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
    info!("🔍 Starting deployment validation...");

    // Validate cloud-init completion
    info!("   Validating cloud-init completion...");
    let cloud_init_validator = CloudInitValidator::new(
        &env.config.ssh_config.ssh_key_path,
        &env.config.ssh_config.ssh_username,
        *instance_ip,
    );
    cloud_init_validator
        .execute(instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker installation
    info!("   Validating Docker installation...");
    let docker_validator = DockerValidator::new(
        &env.config.ssh_config.ssh_key_path,
        &env.config.ssh_config.ssh_username,
        *instance_ip,
    );
    docker_validator
        .execute(instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker Compose installation
    info!("   Validating Docker Compose installation...");
    let docker_compose_validator = DockerComposeValidator::new(
        &env.config.ssh_config.ssh_key_path,
        &env.config.ssh_config.ssh_username,
        *instance_ip,
    );
    docker_compose_validator
        .execute(instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    info!("✅ All deployment validations passed!");
    Ok(())
}

async fn run_full_deployment_test(env: &TestEnvironment) -> Result<IpAddr> {
    info!("🧪 Starting full deployment E2E test with template-based workflow");
    info!("   This will test the complete 4-stage template system:");
    info!("   Stage 1: Render provision templates to build/");
    info!("   Stage 2: Provision VM with OpenTofu from build/");
    info!("   Stage 3: Render configuration templates with variables");
    info!("   Stage 4: Run Ansible playbooks from build/");
    info!("");

    // Stage 1: Render provision templates to build/tofu/
    env.render_provision_templates().await?;

    // Stage 2: Provision infrastructure from build directory
    let instance_ip = env.provision_infrastructure()?;

    // Wait for SSH connectivity
    let ssh_client = SshClient::new(
        &env.config.ssh_config.ssh_key_path,
        &env.config.ssh_config.ssh_username,
        instance_ip,
    );
    ssh_client
        .wait_for_connectivity()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Stage 3: Render configuration templates with runtime variables
    env.render_configuration_templates(&instance_ip).await?;

    // Stage 4: Run Ansible playbooks from build directory
    info!("📋 Step 1: Waiting for cloud-init completion...");
    env.run_ansible_playbook("wait-cloud-init")?;

    // Run the install-docker playbook
    // NOTE: We skip the update-apt-cache playbook in E2E tests to avoid CI network issues
    // The install-docker playbook now assumes the cache is already updated or will handle stale cache gracefully
    info!("📋 Step 2: Installing Docker...");
    env.run_ansible_playbook("install-docker")?;

    // Run the install-docker-compose playbook
    info!("📋 Step 3: Installing Docker Compose...");
    env.run_ansible_playbook("install-docker-compose")?;

    info!("✅ Deployment stages completed successfully!");
    info!("   ✅ Infrastructure provisioned with OpenTofu");
    info!("   ✅ Configuration rendered with Ansible templates");
    info!("   ✅ Ansible playbooks executed successfully");

    info!("🎉 Full deployment E2E test completed successfully!");
    info!("   ℹ️  Docker/Docker Compose installation status varies based on network connectivity");

    // Return the instance IP for validation in main
    Ok(instance_ip)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber to display logs from OpenTofuClient
    fmt::init();

    let cli = Cli::parse();

    info!("🚀 Torrust Tracker Deploy E2E Tests");
    info!("===========================================");

    let env = TestEnvironment::new(cli.keep, cli.templates_dir)?;

    let test_start = Instant::now();

    let result = run_full_deployment_test(&env).await;

    // Handle deployment results and run validation if deployment succeeded
    let validation_result = match result {
        Ok(instance_ip) => {
            info!("");
            validate_deployment(&env, &instance_ip).await
        }
        Err(deployment_err) => {
            error!("❌ Deployment failed: {deployment_err}");
            Err(deployment_err)
        }
    };

    env.cleanup();

    let test_duration = test_start.elapsed();
    info!("📊 Test execution time: {test_duration:?}");

    // Handle final results
    match validation_result {
        Ok(()) => {
            info!("✅ All tests passed and cleanup completed successfully");
            Ok(())
        }
        Err(test_err) => {
            error!("❌ Test failed: {test_err}");
            Err(test_err)
        }
    }
}
