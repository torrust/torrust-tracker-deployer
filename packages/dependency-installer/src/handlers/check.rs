//! Check command handler
//!
//! This module handles checking whether dependencies are installed.

use thiserror::Error;
use tracing::{error, info};

use crate::detector::DetectionError;
use crate::{Dependency, DependencyManager};

/// Errors that can occur when handling the check command
#[derive(Debug, Error)]
pub enum CheckError {
    /// Failed to check all tools
    ///
    /// This occurs when checking all dependencies at once.
    #[error("Failed to check all tools: {source}")]
    CheckAllFailed {
        #[source]
        source: CheckAllToolsError,
    },

    /// Failed to check a specific tool
    ///
    /// This occurs when checking a single specified tool.
    #[error("Failed to check specific tool: {source}")]
    CheckSpecificFailed {
        #[source]
        source: CheckSpecificToolError,
    },
}

/// Errors that can occur when checking all tools
#[derive(Debug, Error)]
pub enum CheckAllToolsError {
    /// Failed to check dependencies
    ///
    /// This occurs when the dependency detection system fails to check
    /// the status of installed tools.
    #[error("Failed to check dependencies: {source}")]
    DependencyCheckFailed {
        #[source]
        source: DetectionError,
    },

    /// One or more dependencies are missing
    ///
    /// This occurs when required tools are not installed on the system.
    #[error("Missing {missing_count} out of {total_count} required dependencies")]
    MissingDependencies {
        /// Number of missing dependencies
        missing_count: usize,
        /// Total number of dependencies checked
        total_count: usize,
    },
}

/// Errors that can occur when checking a specific tool
#[derive(Debug, Error)]
pub enum CheckSpecificToolError {
    /// Failed to parse the tool name
    ///
    /// This occurs when the user provides an unrecognized tool name.
    #[error("Failed to parse tool name: {source}")]
    ParseFailed {
        #[source]
        source: ParseToolNameError,
    },

    /// Failed to detect if the tool is installed
    ///
    /// This occurs when the dependency detection system fails to check
    /// whether a specific tool is installed.
    #[error("Failed to detect tool installation: {source}")]
    DetectionFailed {
        #[source]
        source: DetectionError,
    },

    /// Tool is not installed
    ///
    /// This occurs when the specified tool is not found on the system.
    #[error("{tool}: not installed")]
    ToolNotInstalled {
        /// Name of the tool that is not installed
        tool: String,
    },
}

/// Errors that can occur when parsing tool names
#[derive(Debug, Error)]
pub enum ParseToolNameError {
    /// Unknown tool name provided
    ///
    /// This occurs when the user specifies a tool name that is not recognized.
    /// The error includes the invalid name and a list of available tools.
    #[error("Unknown tool: {name}. Available: {available_tools}")]
    UnknownTool {
        /// The tool name that was not recognized
        name: String,
        /// Comma-separated list of available tool names
        available_tools: String,
    },
}

/// Handle the check command
///
/// # Errors
///
/// Returns an error if:
/// - Dependencies are missing
/// - Invalid tool name is provided
/// - Internal error occurs during dependency checking
pub fn handle_check(manager: &DependencyManager, tool: Option<String>) -> Result<(), CheckError> {
    match tool {
        Some(tool_name) => check_specific_tool(manager, &tool_name)
            .map_err(|source| CheckError::CheckSpecificFailed { source }),
        None => check_all_tools(manager).map_err(|source| CheckError::CheckAllFailed { source }),
    }
}

fn check_all_tools(manager: &DependencyManager) -> Result<(), CheckAllToolsError> {
    info!("Checking all dependencies");
    println!("Checking dependencies...\n");

    let results = manager
        .check_all()
        .map_err(|source| CheckAllToolsError::DependencyCheckFailed { source })?;

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
        error!(
            "Missing {} out of {} required dependencies",
            missing_count,
            results.len()
        );
        eprintln!(
            "Missing {missing_count} out of {} required dependencies",
            results.len()
        );
        Err(CheckAllToolsError::MissingDependencies {
            missing_count,
            total_count: results.len(),
        })
    } else {
        info!("All dependencies are installed");
        println!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_tool(
    manager: &DependencyManager,
    tool_name: &str,
) -> Result<(), CheckSpecificToolError> {
    info!(tool = tool_name, "Checking specific tool");

    // Parse tool name to Dependency enum
    let dep = parse_tool_name(tool_name)
        .map_err(|source| CheckSpecificToolError::ParseFailed { source })?;

    let detector = manager.get_detector(dep);

    let installed = detector
        .is_installed()
        .map_err(|source| CheckSpecificToolError::DetectionFailed { source })?;

    if installed {
        info!(tool = detector.name(), "Tool is installed");
        println!("✓ {}: installed", detector.name());
        Ok(())
    } else {
        error!(tool = detector.name(), "Tool is not installed");
        eprintln!("✗ {}: not installed", detector.name());
        Err(CheckSpecificToolError::ToolNotInstalled {
            tool: detector.name().to_string(),
        })
    }
}

fn parse_tool_name(name: &str) -> Result<Dependency, ParseToolNameError> {
    match name.to_lowercase().as_str() {
        "cargo-machete" | "machete" => Ok(Dependency::CargoMachete),
        "opentofu" | "tofu" => Ok(Dependency::OpenTofu),
        "ansible" => Ok(Dependency::Ansible),
        "lxd" => Ok(Dependency::Lxd),
        _ => {
            // List of available tools - should be kept in sync with the match arms above
            const AVAILABLE_TOOLS: &str = "cargo-machete, opentofu, ansible, lxd";
            Err(ParseToolNameError::UnknownTool {
                name: name.to_string(),
                available_tools: AVAILABLE_TOOLS.to_string(),
            })
        }
    }
}
