//! Main binary entry point for Torrust Tracker Deployer.
//!
//! This binary provides the main CLI interface for the deployment infrastructure.
//! All application logic is contained in the `app` module.

mod app;

fn main() {
    app::run();
}
