use anyhow::{Context, Result};
use clap::Parser;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// Import command execution system
use torrust_tracker_deploy::config::{Config, SshConfig};
use torrust_tracker_deploy::container::Services;
// Import remote actions
use torrust_tracker_deploy::actions::{
    CloudInitValidator, DockerComposeValidator, DockerValidator, RemoteAction,
};
// Import steps
use torrust_tracker_deploy::steps::{
    ApplyInfrastructureStep, GetInstanceInfoStep, InitializeInfrastructureStep,
    PlanInfrastructureStep, RenderAnsibleTemplatesStep, RenderOpenTofuTemplatesStep,
    WaitForCloudInitStep, WaitForSSHConnectivityStep,
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

        // Create configuration
        // Note: Using placeholder IP since this config is used as a template - actual host IP will be set per connection
        let placeholder_ip = "0.0.0.0".parse().expect("Valid IP address");
        let ssh_config = SshConfig::new(
            temp_ssh_key,
            temp_ssh_pub_key,
            "torrust".to_string(),
            placeholder_ip,
        );
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

    /// Stage 1: Render provision templates (`OpenTofu`) to build/tofu/ directory
    async fn render_provision_templates(&self) -> Result<()> {
        let step =
            RenderOpenTofuTemplatesStep::new(Arc::clone(&self.services.tofu_template_renderer));
        step.execute()
            .await
            .with_context(|| "Failed to render provision templates")
    }

    fn run_ansible_playbook(&self, playbook: &str) -> Result<()> {
        info!(
            stage = "ansible_execution",
            playbook = playbook,
            "Running Ansible playbook"
        );

        self.services
            .ansible_client
            .run_playbook(playbook)
            .context(format!("Failed to run Ansible playbook: {playbook}"))?;

        info!(
            stage = "ansible_execution",
            playbook = playbook,
            status = "success",
            "Ansible playbook executed successfully"
        );
        Ok(())
    }

    fn provision_infrastructure(&self) -> Result<IpAddr> {
        info!(
            stage = "infrastructure_provisioning",
            "Provisioning test infrastructure"
        );

        // Initialize OpenTofu using the step
        let initialize_step =
            InitializeInfrastructureStep::new(Arc::clone(&self.services.opentofu_client));
        initialize_step
            .execute()
            .map_err(anyhow::Error::from)
            .context("Failed to initialize OpenTofu")?;

        // Plan infrastructure using the step
        let plan_step = PlanInfrastructureStep::new(Arc::clone(&self.services.opentofu_client));
        plan_step
            .execute()
            .map_err(anyhow::Error::from)
            .context("Failed to plan OpenTofu configuration")?;

        // Apply infrastructure using the step
        let apply_step = ApplyInfrastructureStep::new(Arc::clone(&self.services.opentofu_client));
        apply_step
            .execute()
            .map_err(anyhow::Error::from)
            .context("Failed to apply OpenTofu configuration")?;

        // Get instance info using the step
        let get_instance_info_step =
            GetInstanceInfoStep::new(Arc::clone(&self.services.opentofu_client));
        let opentofu_instance_info = get_instance_info_step
            .execute()
            .map_err(anyhow::Error::from)
            .context("Failed to get container info from OpenTofu outputs")?;

        let opentofu_instance_ip = opentofu_instance_info.ip_address;

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
    info!(
        stage = "validation",
        component = "cloud_init",
        "Validating cloud-init completion"
    );
    let cloud_init_ssh_config = SshConfig::new(
        env.config.ssh_config.ssh_priv_key_path.clone(),
        env.config.ssh_config.ssh_pub_key_path.clone(),
        env.config.ssh_config.ssh_username.clone(),
        *instance_ip,
    );
    let cloud_init_validator = CloudInitValidator::new(cloud_init_ssh_config);
    cloud_init_validator
        .execute(instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker installation
    info!(
        stage = "validation",
        component = "docker",
        "Validating Docker installation"
    );
    let docker_ssh_config = SshConfig::new(
        env.config.ssh_config.ssh_priv_key_path.clone(),
        env.config.ssh_config.ssh_pub_key_path.clone(),
        env.config.ssh_config.ssh_username.clone(),
        *instance_ip,
    );
    let docker_validator = DockerValidator::new(docker_ssh_config);
    docker_validator
        .execute(instance_ip)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Validate Docker Compose installation
    info!(
        stage = "validation",
        component = "docker_compose",
        "Validating Docker Compose installation"
    );
    let docker_compose_ssh_config = SshConfig::new(
        env.config.ssh_config.ssh_priv_key_path.clone(),
        env.config.ssh_config.ssh_pub_key_path.clone(),
        env.config.ssh_config.ssh_username.clone(),
        *instance_ip,
    );
    let docker_compose_validator = DockerComposeValidator::new(docker_compose_ssh_config);
    docker_compose_validator
        .execute(instance_ip)
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
        stages = 4,
        "Starting full deployment E2E test"
    );

    // Stage 1: Render provision templates to build/tofu/
    env.render_provision_templates().await?;

    // Stage 2: Provision infrastructure from build directory
    let instance_ip = env.provision_infrastructure()?;

    // Wait for SSH connectivity
    let wait_ssh_config = SshConfig::new(
        env.config.ssh_config.ssh_priv_key_path.clone(),
        env.config.ssh_config.ssh_pub_key_path.clone(),
        env.config.ssh_config.ssh_username.clone(),
        instance_ip,
    );
    let wait_ssh_step = WaitForSSHConnectivityStep::new(wait_ssh_config);
    wait_ssh_step
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Stage 3: Render configuration templates with runtime variables
    let step = RenderAnsibleTemplatesStep::new(
        Arc::clone(&env.services.ansible_template_renderer),
        env.config.ssh_config.ssh_priv_key_path.clone(),
        instance_ip,
    );
    step.execute()
        .await
        .with_context(|| "Failed to render configuration templates")?;

    // Stage 4: Run Ansible playbooks from build directory
    let wait_cloud_init_step = WaitForCloudInitStep::new(env.services.ansible_client.clone());
    wait_cloud_init_step
        .execute()
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to wait for cloud-init completion")?;

    // Run the install-docker playbook
    // NOTE: We skip the update-apt-cache playbook in E2E tests to avoid CI network issues
    // The install-docker playbook now assumes the cache is already updated or will handle stale cache gracefully
    info!(step = 2, action = "install_docker", "Installing Docker");
    env.run_ansible_playbook("install-docker")?;

    // Run the install-docker-compose playbook
    info!(
        step = 3,
        action = "install_docker_compose",
        "Installing Docker Compose"
    );
    env.run_ansible_playbook("install-docker-compose")?;

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
