use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use tempfile::TempDir;
use tracing_subscriber::fmt;

// Import command execution system
use torrust_tracker_deploy::ansible::AnsibleClient;
use torrust_tracker_deploy::lxd::LxdClient;
use torrust_tracker_deploy::opentofu::OpenTofuClient;
use torrust_tracker_deploy::ssh::SshClient;
use torrust_tracker_deploy::stages::ProvisionTemplateRenderer;
// Import template system
use torrust_tracker_deploy::template::file::File;
use torrust_tracker_deploy::template::wrappers::ansible::inventory::{
    AnsibleHost, InventoryContext, InventoryTemplate, SshPrivateKeyFile,
};
use torrust_tracker_deploy::template::TemplateManager;
// Import remote actions
use torrust_tracker_deploy::actions::{
    CloudInitValidator, DockerComposeValidator, DockerValidator, RemoteAction,
};

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

    /// Templates directory path (default: ./data/templates)
    #[arg(long, default_value = "./data/templates")]
    templates_dir: String,
}

struct TestEnvironment {
    #[allow(dead_code)] // Still used for SSH key fixtures and cleanup
    project_root: PathBuf,
    #[allow(dead_code)] // Will be used in template rendering
    build_dir: PathBuf,
    keep_env: bool,
    verbose: bool,
    ssh_key_path: PathBuf,
    template_manager: TemplateManager,
    opentofu_client: OpenTofuClient,
    provision_renderer: ProvisionTemplateRenderer,
    ssh_client: SshClient,
    lxd_client: LxdClient,
    ansible_client: AnsibleClient,
    #[allow(dead_code)] // Kept to maintain temp directory lifetime
    temp_dir: Option<tempfile::TempDir>,
    #[allow(dead_code)] // Used for cleanup but not directly accessed
    original_inventory: Option<String>,
}

impl TestEnvironment {
    fn new(keep_env: bool, verbose: bool, templates_dir: String) -> Result<Self> {
        // Get project root (current directory when running from root)
        let project_root = std::env::current_dir()?;

        // Create template manager
        let template_manager = TemplateManager::new(templates_dir);

        // Clean templates directory to ensure we use fresh templates from embedded resources
        if verbose {
            println!("🧹 Cleaning templates directory to ensure fresh embedded templates...");
        }
        template_manager.clean_templates_dir()?;

        template_manager.ensure_templates_dir()?;

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

        // Create SSH client with the configured key and username
        let ssh_client = SshClient::new(&temp_ssh_key, "torrust", verbose);

        // Create OpenTofu client pointing to build/tofu/lxd directory
        let opentofu_client = OpenTofuClient::new(project_root.join("build/tofu/lxd"), verbose);

        // Create LXD client for instance management
        let lxd_client = LxdClient::new(verbose);

        // Create Ansible client pointing to build/ansible directory
        let ansible_client = AnsibleClient::new(project_root.join("build/ansible"), verbose);

        // Create provision template renderer
        let provision_renderer =
            ProvisionTemplateRenderer::new(project_root.join("build"), verbose);

        if verbose {
            println!(
                "🔑 SSH key copied to temporary location: {}",
                temp_ssh_key.display()
            );
            println!("📁 Temporary directory: {}", temp_dir.path().display());
            println!(
                "📄 Templates directory: {}",
                template_manager.templates_dir().display()
            );
        }

        Ok(Self {
            build_dir: project_root.join("build"),
            project_root,
            keep_env,
            verbose,
            ssh_key_path: temp_ssh_key,
            template_manager,
            opentofu_client,
            provision_renderer,
            ssh_client,
            lxd_client,
            ansible_client,
            temp_dir: Some(temp_dir),
            original_inventory: None,
        })
    }

    /// Stage 1: Render provision templates (`OpenTofu`) to build/tofu/ directory
    async fn render_provision_templates(&self) -> Result<()> {
        self.provision_renderer
            .render(&self.template_manager)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Stage 3: Render configuration templates (`Ansible`) with runtime variables to build/ansible/
    async fn render_configuration_templates(&self, instance_ip: &str) -> Result<()> {
        println!("🎭 Stage 3: Rendering configuration templates with variables...");

        // Create build directory structure
        let build_ansible_dir = self.build_dir.join("ansible");
        tokio::fs::create_dir_all(&build_ansible_dir)
            .await
            .context("Failed to create build/ansible directory")?;

        // Render inventory.yml.tera with runtime variables
        let inventory_template_path = self
            .template_manager
            .get_template_path("ansible/inventory.yml.tera")?;
        let inventory_output_path = build_ansible_dir.join("inventory.yml");

        let inventory_template_content = std::fs::read_to_string(&inventory_template_path)
            .context("Failed to read inventory template file")?;

        let inventory_template_file = File::new("inventory.yml.tera", inventory_template_content)
            .context("Failed to create inventory template file")?;

        let inventory_context = {
            let host = AnsibleHost::from_str(instance_ip).context("Failed to parse instance IP")?;
            let ssh_key = SshPrivateKeyFile::new(self.ssh_key_path.to_string_lossy().as_ref())
                .context("Failed to parse SSH key path")?;

            InventoryContext::builder()
                .with_host(host)
                .with_ssh_priv_key_path(ssh_key)
                .build()
                .context("Failed to create InventoryContext")?
        };
        let inventory_template =
            InventoryTemplate::new(&inventory_template_file, inventory_context)
                .context("Failed to create InventoryTemplate")?;

        inventory_template
            .render(&inventory_output_path)
            .context("Failed to render inventory template")?;

        // Copy static ansible files
        // Copy ansible.cfg
        let source_cfg = self
            .template_manager
            .get_template_path("ansible/ansible.cfg")?;
        let dest_cfg = build_ansible_dir.join("ansible.cfg");
        tokio::fs::copy(&source_cfg, &dest_cfg)
            .await
            .context("Failed to copy ansible.cfg to build directory")?;

        // Copy playbooks
        for playbook in &[
            "update-apt-cache.yml",
            "install-docker.yml",
            "install-docker-compose.yml",
            "wait-cloud-init.yml",
        ] {
            let source_playbook = self
                .template_manager
                .get_template_path(&format!("ansible/{playbook}"))?;
            let dest_playbook = build_ansible_dir.join(playbook);
            tokio::fs::copy(&source_playbook, &dest_playbook)
                .await
                .with_context(|| format!("Failed to copy {playbook} to build directory"))?;
        }

        if self.verbose {
            println!(
                "   ✅ Configuration templates rendered to: {}",
                build_ansible_dir.display()
            );
            println!("   ✅ Inventory rendered with IP: {instance_ip}");
            println!(
                "   ✅ Inventory rendered with SSH key: {}",
                self.ssh_key_path.display()
            );
        }

        println!("✅ Stage 3 complete: Configuration templates ready");
        Ok(())
    }

    fn run_ansible_playbook(&self, playbook: &str) -> Result<()> {
        println!("🎭 Stage 4: Running Ansible playbook: {playbook}");

        self.ansible_client
            .run_playbook(playbook)
            .context(format!("Failed to run Ansible playbook: {playbook}"))?;

        println!("✅ Stage 4: Ansible playbook executed successfully");
        Ok(())
    }

    fn provision_infrastructure(&self) -> Result<String> {
        println!("🚀 Stage 2: Provisioning test infrastructure...");

        // Initialize OpenTofu
        println!("   Initializing OpenTofu...");
        self.opentofu_client
            .init()
            .map_err(anyhow::Error::from)
            .context("Failed to initialize OpenTofu")?;

        // Apply infrastructure
        println!("   Applying infrastructure...");
        self.opentofu_client
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
            .opentofu_client
            .get_instance_info()
            .map_err(anyhow::Error::from)
            .context("Failed to get container info from OpenTofu outputs")?;

        let opentofu_instance_ip = opentofu_instance_info.ip_address.to_string();

        // Get the instance IP from LXD client (keeping for comparison/validation)
        let lxd_instance_ip = self
            .get_instance_ip()
            .context("Failed to get instance IP from LXD client")?;

        println!("✅ Stage 2 complete: Infrastructure provisioned");
        println!("   Instance IP from OpenTofu: {opentofu_instance_ip}");
        println!("   Instance IP from LXD client: {lxd_instance_ip}");

        // Return the IP from OpenTofu as it's our preferred source
        Ok(opentofu_instance_ip)
    }

    fn get_instance_ip(&self) -> Result<String> {
        // For E2E tests, we should rely on OpenTofu outputs since they already wait for network
        // This is a secondary validation that the instance is accessible via LXD

        // First, check if the instance exists
        let instance = self
            .lxd_client
            .get_instance_by_name("torrust-vm")
            .context("Failed to query LXD for instance information")?
            .ok_or_else(|| anyhow::anyhow!("Instance 'torrust-vm' was not found in LXD"))?;

        // Then, check if the instance has an IP address
        let ip = instance.ip_address.ok_or_else(|| {
            anyhow::anyhow!("Instance 'torrust-vm' exists but has no IPv4 address assigned")
        })?;

        Ok(ip.to_string())
    }

    fn cleanup(&self) {
        if self.keep_env {
            println!("🔒 Keeping test environment as requested");
            println!("   Instance: torrust-vm");
            println!("   Connect with: lxc exec torrust-vm -- /bin/bash");
            return;
        }

        println!("🧹 Cleaning up test environment...");

        // Destroy infrastructure using OpenTofuClient
        let result = self
            .opentofu_client
            .destroy(true) // auto_approve = true
            .map_err(anyhow::Error::from);

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
            // Using emergency_destroy for consistent OpenTofu handling
            let tofu_dir = self.build_dir.join("tofu/lxd");

            drop(torrust_tracker_deploy::opentofu::emergency_destroy(
                &tofu_dir,
            ));
        }
    }
}

async fn run_full_deployment_test(env: &TestEnvironment) -> Result<()> {
    println!("🧪 Starting full deployment E2E test with template-based workflow");
    println!("   This will test the complete 4-stage template system:");
    println!("   Stage 1: Render provision templates to build/");
    println!("   Stage 2: Provision VM with OpenTofu from build/");
    println!("   Stage 3: Render configuration templates with variables");
    println!("   Stage 4: Run Ansible playbooks from build/");
    println!();

    // Stage 1: Render provision templates to build/tofu/
    env.render_provision_templates().await?;

    // Stage 2: Provision infrastructure from build directory
    let instance_ip = env.provision_infrastructure()?;

    // Wait for SSH connectivity
    env.ssh_client
        .wait_for_connectivity(&instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Stage 3: Render configuration templates with runtime variables
    env.render_configuration_templates(&instance_ip).await?;

    // Stage 4: Run Ansible playbooks from build directory
    println!("📋 Step 1: Waiting for cloud-init completion...");
    env.run_ansible_playbook("wait-cloud-init")?;

    // Validate cloud-init completion
    let cloud_init_validator = CloudInitValidator::new(&env.ssh_key_path, "torrust", env.verbose);
    cloud_init_validator
        .execute(&instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Run the install-docker playbook
    // NOTE: We skip the update-apt-cache playbook in E2E tests to avoid CI network issues
    // The install-docker playbook now assumes the cache is already updated or will handle stale cache gracefully
    println!("📋 Step 2: Installing Docker...");
    env.run_ansible_playbook("install-docker")?;

    // 7. Validate Docker installation
    let docker_validator = DockerValidator::new(&env.ssh_key_path, "torrust", env.verbose);
    docker_validator
        .execute(&instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // 8. Run the install-docker-compose playbook
    println!("📋 Step 3: Installing Docker Compose...");
    env.run_ansible_playbook("install-docker-compose")?;

    // 9. Validate Docker Compose installation
    let docker_compose_validator =
        DockerComposeValidator::new(&env.ssh_key_path, "torrust", env.verbose);
    docker_compose_validator
        .execute(&instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("🎉 Full deployment E2E test completed successfully!");
    println!("   ✅ Cloud-init setup completed");
    println!("   ✅ Ansible playbooks executed successfully");
    println!(
        "   ℹ️  Docker/Docker Compose installation status varies based on network connectivity"
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber to display logs from OpenTofuClient
    fmt::init();

    let cli = Cli::parse();

    println!("🚀 Torrust Tracker Deploy E2E Tests");
    println!("===========================================");

    let env = TestEnvironment::new(cli.keep, cli.verbose, cli.templates_dir)?;

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
