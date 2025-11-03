# Create CLI Binary with Check Command

**Issue**: [#115](https://github.com/torrust/torrust-tracker-deployer/issues/115)
**Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests  
**Depends On**: [#114](https://github.com/torrust/torrust-tracker-deployer/issues/114) - Create Detection Logic Package (Issue 1-1-1)
**Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Overview

Build a command-line interface binary that wraps the detection functionality from Phase 1 (Issue 1-1-1). The CLI will provide `check` and `list` subcommands, making the detection logic accessible to users and testable in Docker containers (Phase 3).

## Objectives

- [ ] Create CLI binary with clap for argument parsing
- [ ] Implement `check` subcommand to verify tool installation status
- [ ] Implement `list` subcommand to show all available tools
- [ ] Add verbose output flag and tool-specific checking
- [ ] Provide clear output formatting and appropriate exit codes
- [ ] Add comprehensive help messages and usage examples

## Context

This is **Phase 2** of creating the dependency installation package. It makes the detection logic from Issue 1-1-1 accessible via a user-friendly command-line interface.

### Why CLI Before Docker Testing

Having a working binary enables:

1. **Direct Docker testing** in Phase 3 (Issue 1-1-3) - just run the binary in containers
2. **Manual testing** during development - developers can test immediately
3. **Realistic testing** - tests how users will actually interact with the tool
4. **Simpler test code** - no complex library integration in tests

The `install` subcommand will be added in Phase 4 (Issue 1-1-4) after Docker testing infrastructure is ready.

### Dependencies

- **Requires**: Issue 1-1-1 (detection logic package) must be completed first
- **Enables**: Issue 1-1-3 (Docker testing) can test the binary directly

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (CLI interface)
**Module Path**: `packages/dependency-installer/src/bin/dependency-installer.rs`
**Pattern**: Binary using clap for CLI + library for logic

### Binary Location

```text
packages/dependency-installer/
├── src/
│   ├── bin/
│   │   └── dependency-installer.rs    # CLI binary (this phase)
│   ├── lib.rs
│   ├── manager.rs
│   └── detector/
```

## Specifications

### CLI Structure with Clap

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dependency-installer")]
#[command(version)]
#[command(about = "Manage development dependencies for E2E tests", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if dependencies are installed
    Check {
        /// Specific tool to check (if omitted, checks all)
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// List all available tools and their status
    List,
}
```

### Main Binary Implementation

```rust
use dependency_installer::{DependencyManager, init_tracing};
use tracing::{info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing based on verbose flag
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    init_tracing();

    let manager = DependencyManager::new();

    match cli.command {
        Commands::Check { tool } => handle_check(&manager, tool),
        Commands::List => handle_list(&manager),
    }
}

fn handle_check(manager: &DependencyManager, tool: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        Some(tool_name) => check_specific_tool(manager, &tool_name),
        None => check_all_tools(manager),
    }
}

fn check_all_tools(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
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
        println!("Missing {} out of {} required dependencies", missing_count, results.len());
        std::process::exit(1);
    } else {
        println!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_tool(manager: &DependencyManager, tool_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse tool name to Dependency enum
    let dep = parse_tool_name(tool_name)?;
    let detector = manager.get_detector(dep);

    let installed = detector.is_installed()?;

    if installed {
        println!("✓ {}: installed", detector.name());
        Ok(())
    } else {
        println!("✗ {}: not installed", detector.name());
        std::process::exit(1);
    }
}

fn handle_list(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("Available tools:\n");

    let results = manager.check_all()?;
    for result in results {
        let status = if result.installed { "installed" } else { "not installed" };
        println!("- {} ({})", result.tool, status);
    }

    Ok(())
}

fn parse_tool_name(name: &str) -> Result<Dependency, String> {
    match name.to_lowercase().as_str() {
        "cargo-machete" | "machete" => Ok(Dependency::CargoMachete),
        "opentofu" | "tofu" => Ok(Dependency::OpenTofu),
        "ansible" => Ok(Dependency::Ansible),
        "lxd" => Ok(Dependency::Lxd),
        _ => Err(format!("Unknown tool: {}. Available: cargo-machete, opentofu, ansible, lxd", name)),
    }
}
```

### Exit Code Specification

The binary must use consistent exit codes:

- **Exit Code 0 (Success)**:

  - `check` command: All checked tools are installed
  - `list` command: Always returns 0 (informational command)

- **Exit Code 1 (Missing Dependencies)**:

  - `check` command without `--tool`: One or more tools are missing
  - `check --tool <name>`: The specified tool is not installed

- **Exit Code 2 (Invalid Arguments)**:

  - Unknown subcommand
  - Invalid tool name passed to `--tool`
  - Missing required arguments

- **Exit Code 3 (Internal Error)**:
  - Detector execution failed
  - System command failed unexpectedly
  - Other runtime errors

**Example Implementation**:

```rust
fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {}", e);

            // Determine exit code based on error type
            if e.to_string().contains("not installed") {
                1  // Missing dependency
            } else if e.to_string().contains("Unknown tool") {
                2  // Invalid argument
            } else {
                3  // Internal error
            }
        }
    };

    std::process::exit(exit_code);
}
```

## Implementation Tasks

### Binary Setup

- [ ] Create `src/bin/dependency-installer.rs`
- [ ] Add `clap` dependency to `Cargo.toml` with `derive` feature
- [ ] Define CLI structure with `Parser` and `Subcommand` derives
- [ ] Add binary target to `Cargo.toml`:

  ```toml
  [[bin]]
  name = "dependency-installer"
  path = "src/bin/dependency-installer.rs"
  ```

### Initialize Tracing

- [ ] Import `init_tracing()` from library
- [ ] Set up tracing based on `--verbose` flag
- [ ] Test that logging works correctly:
  - [ ] Normal mode shows info logs
  - [ ] Verbose mode shows debug logs

### Implement Check Command

- [ ] Create `handle_check()` function
- [ ] Implement `check_all_tools()`:
  - [ ] Call `manager.check_all()`
  - [ ] Format output with ✓ and ✗ symbols
  - [ ] Count missing dependencies
  - [ ] Exit with code 0 if all installed, 1 if any missing
- [ ] Implement `check_specific_tool()`:
  - [ ] Parse tool name to `Dependency` enum
  - [ ] Get specific detector
  - [ ] Check if installed
  - [ ] Format output
  - [ ] Exit with appropriate code

### Implement List Command

- [ ] Create `handle_list()` function
- [ ] Check all tools and show status
- [ ] Format output clearly:

  ```text
  Available tools:

  - cargo-machete (installed)
  - OpenTofu (not installed)
  - Ansible (not installed)
  - LXD (not installed)
  ```

### Error Handling

- [ ] Implement `parse_tool_name()` with clear error messages
- [ ] Handle detection errors gracefully
- [ ] Provide actionable error messages
- [ ] Ensure errors include context

### Help Messages

- [ ] Add command descriptions with `about` attribute
- [ ] Add argument descriptions with `help` attribute
- [ ] Add examples to long help text
- [ ] Test help output:
  - [ ] `dependency-installer --help`
  - [ ] `dependency-installer check --help`
  - [ ] `dependency-installer list --help`

### Manual Testing

- [ ] Build binary: `cargo build --bin dependency-installer`
- [ ] Test check command:
  - [ ] `./target/debug/dependency-installer check`
  - [ ] `./target/debug/dependency-installer check --tool opentofu`
  - [ ] `./target/debug/dependency-installer check --verbose`
- [ ] Test list command:
  - [ ] `./target/debug/dependency-installer list`
- [ ] Verify exit codes:
  - [ ] 0 when all dependencies installed
  - [ ] 1 when any dependencies missing
- [ ] Verify logging output is clear
- [ ] Test with `--help` and `--version` flags

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Binary compiles without warnings
- [ ] Manual testing completed successfully

**Binary Functionality**:

- [ ] Binary can be built with `cargo build --bin dependency-installer`
- [ ] Binary has `check` and `list` subcommands
- [ ] `check` command works for all tools
- [ ] `check --tool <name>` works for specific tools
- [ ] `list` command shows all tools with status
- [ ] `--verbose` flag enables detailed logging
- [ ] Exit codes are correct (0 = success, 1 = missing tools)

**Output Quality**:

- [ ] Output is clear and user-friendly
- [ ] Uses visual indicators (✓ and ✗)
- [ ] Provides summary information (e.g., "Missing 3 out of 4")
- [ ] Error messages are actionable
- [ ] Help text is comprehensive and includes examples

**Documentation**:

- [ ] Binary help text explains all commands
- [ ] Each command has clear description
- [ ] Arguments are documented
- [ ] Examples are provided in help text
- [ ] README updated with CLI usage examples

## Example Usage After Completion

```bash
# Check all dependencies
$ dependency-installer check
Checking dependencies...

✓ cargo-machete: installed
✗ OpenTofu: not installed
✗ Ansible: not installed
✗ LXD: not installed

Missing 3 out of 4 required dependencies

# Check specific tool
$ dependency-installer check --tool opentofu
✗ OpenTofu: not installed

# List all tools
$ dependency-installer list
Available tools:

- cargo-machete (installed)
- OpenTofu (not installed)
- Ansible (not installed)
- LXD (not installed)

# Verbose output
$ dependency-installer check --verbose
2025-11-03T16:00:00.000Z  INFO dependency_installer::detector::cargo_machete: Checking if cargo-machete is installed
2025-11-03T16:00:00.001Z  INFO dependency_installer::detector::cargo_machete: cargo-machete is installed
...

# Help
$ dependency-installer --help
$ dependency-installer check --help
```

## Related Documentation

- [Issue 1-1-1](./114-1-1-1-create-detection-logic-package.md) - Detection logic that this binary uses
- [Parent Issue 1-1](./create-dependency-installation-package-for-e2e-tests.md) - Overall package specification
- [docs/contributing/module-organization.md](../contributing/module-organization.md) - Module organization patterns
- [Clap Documentation](https://docs.rs/clap/) - CLI framework reference

## Notes

### Estimated Time

**2-3 hours** total for this phase.

### Next Steps

After completing this phase:

1. **Issue 1-1-3**: Create Docker testing infrastructure that runs this binary
2. **Issue 1-1-4**: Add `install` subcommand to the binary

### Design Decisions

**Single binary with subcommands**: Following the recommendation from the parent issue, we use a single binary with subcommands rather than separate binaries per tool. This provides better discoverability and a consistent interface.

**No install command yet**: The `install` subcommand will be added in Phase 4 (Issue 1-1-4) after Docker testing infrastructure is in place.

**Exit codes**: Following Unix conventions:

- Exit code 0: Success (all tools installed or command succeeded)
- Exit code 1: Failure (missing tools or error occurred)

**Visual indicators**: Using ✓ and ✗ symbols makes output scannable and user-friendly.

### CLI Design Notes

**Tool name aliases**: The `parse_tool_name()` function accepts multiple names for the same tool (e.g., both "cargo-machete" and "machete" work). This improves user experience.

**Global verbose flag**: The `--verbose` flag is marked as `global = true` in clap, making it available to all subcommands.

**Tracing integration**: The binary uses the library's `init_tracing()` function and adjusts the log level based on the verbose flag.
