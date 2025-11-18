//! Main binary entry point for Torrust Tracker Deployer.
//!
//! This binary provides the main CLI interface for the deployment infrastructure.
//! All application logic is contained in the `bootstrap::app` module.

use torrust_tracker_deployer_lib::bootstrap;

#[tokio::main]
async fn main() {
    bootstrap::app::run().await;
}
