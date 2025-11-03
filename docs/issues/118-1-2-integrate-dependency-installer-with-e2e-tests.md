# Integrate Dependency Installer with E2E Tests

**Type**: Sub-issue (Task)
**Issue**: [#118](https://github.com/torrust/torrust-tracker-deployer/issues/118)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Depends On**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests

## Summary

Integrate the `dependency-installer` package with E2E test binaries to automatically verify and install required dependencies before running tests. This ensures tests fail fast with clear error messages when dependencies are missing, and makes it easier for AI agents to run tests in fresh environments.

## Objectives

- [ ] Add dependency verification to all E2E test binaries
- [ ] Create helper functions for common dependency checking patterns
- [ ] Ensure clear error messages when dependencies are missing or fail to install
- [ ] Test the integration in a clean environment to verify it works as expected

## Background

### Current State

**E2E Test Files Overview**:

The project has 6 E2E test files total:

**Binary-based E2E tests** (in `src/bin/` - executed as standalone binaries):

- `e2e-tests-full.rs` - Comprehensive E2E tests (provisions + configures)
- `e2e-provision-tests.rs` - Infrastructure provisioning tests only
- `e2e-config-tests.rs` - Configuration tests only

**Integration test E2E tests** (in `tests/` - executed by `cargo test`):

- `e2e_create_command.rs` - Tests create command via CLI
- `e2e_destroy_command.rs` - Tests destroy command via CLI

**Additional tests** (not E2E, but related):

- Various integration tests in `tests/` directory

### System Dependencies by Test Type

**Binary-based E2E tests** require system dependencies:

- OpenTofu (for infrastructure provisioning)
- Ansible (for configuration management)
- LXD (for VM-based testing)
- cargo-machete (for pre-commit checks)

**Integration test E2E tests** do NOT currently require system dependencies:

- They only test CLI commands that don't interact with infrastructure
- They use temporary directories and mock data
- No OpenTofu, Ansible, or LXD needed (for now)

### Integration Strategy

**Binary-based E2E tests** (Issue Scope):

- ✅ Add dependency checking at startup
- ✅ Attempt to install missing dependencies automatically
- ✅ Fail with clear error if installation fails

**Integration test E2E tests** (Out of Scope):

- ❌ Do NOT modify these tests
- ❌ No dependency checking needed (they have no system dependencies currently)
- ✅ If they gain system dependencies in the future, only check (don't install)

### Current Problem

If required tools (OpenTofu, Ansible, LXD) are missing, binary-based E2E tests fail with cryptic errors deep in the execution, making it hard to diagnose the root cause.

### Desired State

Each E2E test binary should:

1. Check required dependencies before starting test execution
2. Display clear error messages if dependencies are missing
3. Optionally attempt to install missing dependencies (if appropriate)
4. Provide guidance on how to manually install dependencies if automatic installation fails

## Scope

This issue covers only the integration of the `dependency-installer` package with E2E test binaries. The package creation and CI workflow updates are handled in separate issues.

## Technical Approach

### Using the Library in E2E Tests

The `dependency-installer` package will be used as a library:

```rust
use dependency_installer::{DependencyManager, Dependency};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dependency manager
    let manager = DependencyManager::new();

    // Check required dependencies for this test
    let required_deps = vec![
        Dependency::OpenTofu,
        Dependency::Ansible,
        Dependency::Lxd,
    ];

    // Verify dependencies before running tests
    verify_dependencies(&manager, &required_deps)?;

    // Run actual tests
    run_tests()?;

    Ok(())
}

fn verify_dependencies(
    manager: &DependencyManager,
    deps: &[Dependency],
) -> Result<(), Box<dyn std::error::Error>> {
    for dep in deps {
        let detector = manager.get_detector(dep);

        if !detector.is_installed()? {
            eprintln!("Error: Required dependency '{}' is not installed", detector.name());
            eprintln!("\nTo install dependencies, run:");
            eprintln!("  cargo run --bin dependency-installer install");
            eprintln!("\nOr install manually:");
            // Add tool-specific installation instructions
            return Err(format!("Missing dependency: {}", detector.name()).into());
        }
    }

    println!("✅ All required dependencies are available");
    Ok(())
}
```

### Alternative: Helper Function Pattern

Create a reusable helper that can be shared across E2E tests:

```rust
// In src/testing/e2e_helpers.rs
use dependency_installer::{DependencyManager, Dependency};

/// Verify and optionally install required dependencies for E2E tests
pub fn ensure_e2e_dependencies(deps: &[Dependency]) -> Result<(), Box<dyn std::error::Error>> {
    let manager = DependencyManager::new();

    for dep in deps {
        let detector = manager.get_detector(dep);

        if !detector.is_installed()? {
            // Try to get installer (may not be available for all tools)
            if let Ok(installer) = manager.get_installer(dep) {
                println!("⚙️  Installing {}...", installer.name());
                installer.install()?;
            } else {
                return Err(format!(
                    "Dependency '{}' is not installed and automatic installation is not available",
                    detector.name()
                ).into());
            }
        }
    }

    println!("✅ All dependencies verified");
    Ok(())
}
```

## Implementation Plan

### Phase 1: Add Dependency to E2E Test Binaries (1 hour)

- [ ] Add `dependency-installer` as a dependency to the workspace
- [ ] Update `e2e-tests-full.rs` to add dependency verification at startup
- [ ] Update `e2e-provision-tests.rs` to add dependency verification at startup
- [ ] Update `e2e-config-tests.rs` to add dependency verification at startup

### Phase 2: Create Helper Functions (1-2 hours)

- [ ] Create `src/testing/e2e_helpers.rs` module
- [ ] Implement `ensure_e2e_dependencies()` helper function
- [ ] Add error handling and user-friendly error messages
- [ ] Add structured logging for dependency checks
- [ ] Document the helper function with rustdoc

### Phase 3: Integration Testing (1-2 hours)

- [ ] Test E2E binaries in a clean environment (Docker container)
- [ ] Verify error messages are clear when dependencies are missing
- [ ] Verify automatic installation works (if implemented)
- [ ] Document the integration in `docs/e2e-testing.md`

## Acceptance Criteria

### Functional Requirements

- [ ] All three E2E test binaries (`e2e-tests-full`, `e2e-provision-tests`, `e2e-config-tests`) check dependencies before running tests
- [ ] Tests fail fast with clear error messages if dependencies are missing
- [ ] Error messages include instructions on how to install missing dependencies
- [ ] Dependency verification is logged with appropriate log levels

### Code Quality

- [ ] Helper functions are well-documented with rustdoc
- [ ] Error handling follows project conventions (see `docs/contributing/error-handling.md`)
- [ ] Code follows the DDD layer placement guide (helpers in `src/testing/`)
- [ ] All linters pass (`cargo run --bin linter all`)

### Documentation

- [ ] Update `docs/e2e-testing.md` to document the dependency verification process
- [ ] Document how to run E2E tests in fresh environments
- [ ] Add troubleshooting section for common dependency issues

### Testing

- [ ] Test E2E binaries in a clean Docker environment
- [ ] Verify all three test binaries work with dependency verification
- [ ] Verify error messages are clear and actionable

## Dependencies

This sub-issue depends on:

- **#TBD** - Create Dependency Installation Package for E2E Tests (must be completed first)

## Related Documentation

- [docs/e2e-testing.md](../e2e-testing.md) - E2E testing documentation
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling guidelines
- [docs/contributing/testing/README.md](../contributing/testing/README.md) - Testing conventions

## Estimated Time

**3-5 hours** total:

- Phase 1: 1 hour (adding dependency checks to binaries)
- Phase 2: 1-2 hours (helper functions and error handling)
- Phase 3: 1-2 hours (testing and documentation)

## Notes

### Design Considerations

**Fail-fast approach**: Verifying dependencies at startup ensures clear error messages rather than cryptic failures deep in test execution.

**Helper function pattern**: Creating reusable helpers in `src/testing/` makes it easy to add dependency verification to new E2E tests in the future.

**Error message quality**: Following the project's error handling guidelines ensures users get actionable guidance when dependencies are missing.

### Future Enhancements

- Add automatic dependency installation for E2E tests (with confirmation prompts)
- Cache dependency detection results to avoid repeated checks
- Add `--skip-dependency-check` flag for cases where dependencies are guaranteed to be present
