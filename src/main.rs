fn main() {
    println!("🏗️  Torrust Testing Infrastructure");
    println!("=================================");
    println!();
    println!("This repository provides automated testing infrastructure for Torrust projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the testing infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e-tests binary to run end-to-end tests:");
    println!("   cargo run --bin e2e-tests -- wait-cloud-init");
    println!();
    println!("📖 For detailed instructions, see: README.md");
}
