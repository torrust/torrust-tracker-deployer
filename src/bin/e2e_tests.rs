use anyhow::Result;
use clap::Parser;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deploy::e2e::environment::TestEnvironment;
use torrust_tracker_deploy::e2e::tasks::{
    configure_infrastructure::configure_infrastructure,
    provision_infrastructure::{cleanup_infrastructure, provision_infrastructure},
    validate_deployment::validate_deployment,
};
use torrust_tracker_deploy::logging::{self, LogFormat};

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

    /// Logging format to use
    #[arg(
        long,
        default_value = "pretty",
        help = "Logging format: pretty, json, or compact"
    )]
    log_format: LogFormat,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on the chosen format
    logging::init_with_format(&cli.log_format);

    info!(
        application = "torrust_tracker_deploy",
        test_suite = "e2e_tests",
        log_format = ?cli.log_format,
        "Starting E2E tests"
    );

    let env = TestEnvironment::new(cli.keep, cli.templates_dir)?;

    let test_start = Instant::now();

    let deployment_result = run_full_deployment_test(&env).await;

    let validation_result = match &deployment_result {
        Ok(instance_ip) => validate_deployment(&env, instance_ip).await,
        Err(_) => Ok(()), // Skip validation if deployment failed
    };

    cleanup_infrastructure(&env);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Test execution completed"
    );

    // Handle all 4 combinations of deployment and validation results
    match (deployment_result, validation_result) {
        (Ok(_), Ok(())) => {
            info!(
                test_suite = "e2e_tests",
                status = "success",
                "All tests passed and cleanup completed successfully"
            );
            Ok(())
        }
        (Ok(_), Err(validation_err)) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %validation_err,
                "Deployment succeeded but validation failed"
            );
            Err(validation_err)
        }
        (Err(deployment_err), Ok(())) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %deployment_err,
                "Deployment failed"
            );
            Err(deployment_err)
        }
        (Err(deployment_err), Err(_)) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %deployment_err,
                "Deployment failed (validation skipped)"
            );
            Err(deployment_err)
        }
    }
}

async fn run_full_deployment_test(env: &TestEnvironment) -> Result<IpAddr> {
    info!(
        test_type = "full_deployment",
        workflow = "template_based",
        "Starting full deployment E2E test"
    );

    let instance_ip = provision_infrastructure(env).await?;

    configure_infrastructure(env)?;

    info!(status = "success", "Deployment completed successfully");

    info!(
        test_type = "full_deployment",
        status = "success",
        note = "Docker/Docker Compose installation status varies based on network connectivity",
        "Full deployment E2E test completed successfully"
    );

    // Return the instance IP for validation in main
    Ok(instance_ip)
}
