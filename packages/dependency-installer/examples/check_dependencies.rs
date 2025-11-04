//! Example: Check all development dependencies
//!
//! This example demonstrates how to use the dependency installer package
//! to check if all required development dependencies are installed.
//!
//! Run with: `cargo run --example check_dependencies`

use torrust_dependency_installer::{init_tracing, DependencyManager};

fn main() {
    // Initialize tracing for structured logging
    init_tracing();

    println!("Checking development dependencies...\n");

    // Create dependency manager
    let manager = DependencyManager::new();

    // Check all dependencies
    match manager.check_all() {
        Ok(results) => {
            println!("Dependency Status:");
            println!("{}", "=".repeat(40));

            for result in &results {
                let detector = manager.get_detector(result.dependency);
                let name = detector.name();
                let status = if result.installed { "✓" } else { "✗" };
                let status_text = if result.installed {
                    "Installed"
                } else {
                    "Not Installed"
                };

                println!("{status} {name:20} {status_text}");
            }

            println!("\n{} dependencies checked", results.len());
        }
        Err(e) => {
            eprintln!("Error checking dependencies: {e}");
            std::process::exit(1);
        }
    }
}
