//! Main binary entry point for Torrust Tracker Deploy
//!
//! This binary provides a simple information display about the deployment infrastructure.
//! For actual deployment functionality, use the dedicated binary tools:
//! - `e2e-tests` - Run end-to-end deployment tests
//! - `linter` - Run project linting checks
//!
//! The main deployment workflow is handled through the library's command system
//! rather than this binary entry point.

fn main() {
    println!("🏗️  Torrust Tracker Deploy");
    println!("=========================");
    println!();
    println!("This repository provides automated deployment infrastructure for Torrust tracker projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the deployment infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e-tests binary to run end-to-end tests:");
    println!("   cargo run --bin e2e-tests");
    println!();
    println!("📖 For detailed instructions, see: README.md");
}
