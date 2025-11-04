//! Check command handler
//!
//! This module handles checking whether dependencies are installed.

use tracing::{error, info};

use crate::{Dependency, DependencyManager};

/// Handle the check command
///
/// # Errors
///
/// Returns an error if:
/// - Dependencies are missing
/// - Invalid tool name is provided
/// - Internal error occurs during dependency checking
pub fn handle_check(
    manager: &DependencyManager,
    tool: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        Some(tool_name) => check_specific_tool(manager, &tool_name),
        None => check_all_tools(manager),
    }
}

fn check_all_tools(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking all dependencies");
    println!("Checking dependencies...\n");

    let results = manager.check_all()?;
    let mut missing_count = 0;

    for result in &results {
        if result.installed {
            println!("✓ {}: installed", result.tool);
        } else {
            println!("✗ {}: not installed", result.tool);
            missing_count += 1;
        }
    }

    println!();
    if missing_count > 0 {
        let msg = format!(
            "Missing {missing_count} out of {} required dependencies",
            results.len()
        );
        error!("{}", msg);
        eprintln!("{msg}");
        Err(msg.into())
    } else {
        info!("All dependencies are installed");
        println!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_tool(
    manager: &DependencyManager,
    tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(tool = tool_name, "Checking specific tool");

    // Parse tool name to Dependency enum
    let dep = parse_tool_name(tool_name)?;
    let detector = manager.get_detector(dep);

    let installed = detector.is_installed()?;

    if installed {
        info!(tool = detector.name(), "Tool is installed");
        println!("✓ {}: installed", detector.name());
        Ok(())
    } else {
        let msg = format!("{}: not installed", detector.name());
        error!(tool = detector.name(), "Tool is not installed");
        eprintln!("✗ {msg}");
        Err(msg.into())
    }
}

fn parse_tool_name(name: &str) -> Result<Dependency, String> {
    match name.to_lowercase().as_str() {
        "cargo-machete" | "machete" => Ok(Dependency::CargoMachete),
        "opentofu" | "tofu" => Ok(Dependency::OpenTofu),
        "ansible" => Ok(Dependency::Ansible),
        "lxd" => Ok(Dependency::Lxd),
        _ => {
            // List of available tools - should be kept in sync with the match arms above
            const AVAILABLE_TOOLS: &str = "cargo-machete, opentofu, ansible, lxd";
            Err(format!(
                "Unknown tool: {name}. Available: {AVAILABLE_TOOLS}"
            ))
        }
    }
}
