//! # AI Precommit Enforcement Tests
//!
//! This test suite is specifically designed to enforce pre-commit quality checks
//! for AI assistants (such as GitHub Copilot) working on this project.
//!
//! ## Purpose
//!
//! AI assistants often work in remote environments (like GitHub shared runners)
//! where they may not have access to local Git hooks or pre-commit scripts.
//! These integration tests ensure that all pre-commit validation steps are
//! executed and pass before any code changes are committed.
//!
//! ## What It Validates
//!
//! - **Dependencies**: Ensures no unused dependencies (`cargo-machete`)
//! - **Code Quality**: Runs comprehensive linting (`cargo run --bin linter all`)
//! - **Documentation**: Validates documentation builds (`cargo doc`)
//! - **End-to-End Tests**: Runs E2E tests compatible with shared runners
//!
//! ## Usage
//!
//! ```bash
//! # Default: Run all AI validation tests (recommended for AI assistants)
//! cargo test ai_enforcement
//!
//! # Development: Skip AI enforcement for faster iteration
//! SKIP_AI_ENFORCEMENT=1 cargo test ai_enforcement
//! ```
//!
//! ## Related Documentation
//!
//! For detailed testing conventions and guidelines, see:
//! - [`docs/contributing/testing/`](../docs/contributing/testing/)
//! - [`docs/contributing/commit-process.md`](../docs/contributing/commit-process.md)
//!
//! ## Environment Variable Control
//!
//! - **Default behavior**: All AI validation tests run when `SKIP_AI_ENFORCEMENT` is not set
//! - **Skip AI enforcement**: Set `SKIP_AI_ENFORCEMENT=1` to skip AI quality enforcement during development
//! - **Run AI enforcement**: Set `SKIP_AI_ENFORCEMENT=0` or any other value, or leave unset
//!
//! This allows rapid development cycles while ensuring AI assistants run full validation by default.

#[cfg(test)]
mod ai_enforcement_tests {
    use std::env;
    use std::process::Command;

    #[test]
    fn it_should_pass_dependency_check() {
        if env::var("SKIP_AI_ENFORCEMENT").unwrap_or_default() == "1" {
            println!("Skipping AI enforcement - set SKIP_AI_ENFORCEMENT=1 to skip or unset to run");
            return;
        }

        let workspace_root =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");

        // Try calling cargo-machete directly instead of through cargo
        let output = Command::new("cargo-machete")
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to run cargo-machete");

        if !output.status.success() {
            eprintln!(
                "cargo-machete stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "cargo-machete stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(output.status.success(), "cargo machete should pass");
    }

    #[test]
    fn it_should_pass_linting_checks() {
        if env::var("SKIP_AI_ENFORCEMENT").unwrap_or_default() == "1" {
            println!("Skipping AI enforcement - set SKIP_AI_ENFORCEMENT=1 to skip or unset to run");
            return;
        }

        let workspace_root =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");

        let output = Command::new("cargo")
            .args(["run", "--bin", "linter", "all"])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to run linter");

        if !output.status.success() {
            eprintln!("Linter stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("Linter stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        assert!(output.status.success(), "All linters should pass");
    }

    #[test]
    fn it_should_pass_documentation_build() {
        if env::var("SKIP_AI_ENFORCEMENT").unwrap_or_default() == "1" {
            println!("Skipping AI enforcement - set SKIP_AI_ENFORCEMENT=1 to skip or unset to run");
            return;
        }

        let workspace_root =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");

        let output = Command::new("cargo")
            .args([
                "doc",
                "--no-deps",
                "--bins",
                "--examples",
                "--workspace",
                "--all-features",
            ])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to run cargo doc");

        if !output.status.success() {
            eprintln!(
                "cargo doc stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "cargo doc stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(
            output.status.success(),
            "Documentation should build successfully"
        );
    }

    #[test]
    fn it_should_pass_e2e_config_tests() {
        if env::var("SKIP_AI_ENFORCEMENT").unwrap_or_default() == "1" {
            println!("Skipping AI enforcement - set SKIP_AI_ENFORCEMENT=1 to skip or unset to run");
            return;
        }

        let workspace_root =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");

        let output = Command::new("cargo")
            .args(["run", "--bin", "e2e-config-tests"])
            .current_dir(&workspace_root)
            .env("RUST_LOG", "warn")
            .output()
            .expect("Failed to run E2E config tests");

        if !output.status.success() {
            eprintln!(
                "E2E config tests stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "E2E config tests stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(output.status.success(), "E2E config tests should pass");
    }

    #[test]
    fn it_should_pass_e2e_provision_tests() {
        if env::var("SKIP_AI_ENFORCEMENT").unwrap_or_default() == "1" {
            println!("Skipping AI enforcement - set SKIP_AI_ENFORCEMENT=1 to skip or unset to run");
            return;
        }

        let workspace_root =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");

        let output = Command::new("cargo")
            .args(["run", "--bin", "e2e-provision-and-destroy-tests"])
            .current_dir(&workspace_root)
            .env("RUST_LOG", "warn")
            .output()
            .expect("Failed to run E2E provision tests");

        if !output.status.success() {
            eprintln!(
                "E2E provision tests stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "E2E provision tests stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(output.status.success(), "E2E provision tests should pass");
    }
}
