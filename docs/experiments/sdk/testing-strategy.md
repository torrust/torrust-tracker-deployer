# SDK Testing Strategy

## Context

The SDK (`packages/sdk/`) wraps the same application-layer command handlers
that the CLI uses. The CLI already has comprehensive testing at multiple
levels:

- **Unit tests** — 2240+ tests in `src/` covering domain, application,
  infrastructure, and presentation layers
- **E2E binary tests** — `e2e_complete_workflow_tests`,
  `e2e_deployment_workflow_tests`, `e2e_infrastructure_lifecycle_tests`
  exercise the full CLI as external processes
- **E2E integration tests** — `tests/e2e/` exercises individual commands
  (create, destroy, list, show, purge, render, validate) as black-box
  process invocations

Since the CLI and SDK share the same application/domain/infrastructure
layers, and those layers are already well-tested, the question is: **what
additional testing does the SDK layer itself need?**

The SDK layer is thin — it delegates to command handlers and translates
results. Its own logic is limited to:

1. Wiring (builder creates the right dependencies)
2. Delegation (each method calls the right handler with the right args)
3. Error mapping (`SdkError` wraps per-command errors)
4. Public API surface guarantees (re-exports, trait bounds)

---

## Proposals

### Proposal 1: SDK Integration Tests in `packages/sdk/tests/` (Recommended)

**What**: Create `packages/sdk/tests/` with integration tests that use the
SDK as an external consumer would — importing only from the public API
(`torrust_tracker_deployer_sdk::*`). Tests exercise the local-only
operations (create, show, list, exists, validate, destroy, purge) against
a temporary workspace directory.

**Scope**: Only local operations (no LXD/Docker/SSH needed). These tests
verify that the SDK public API works end-to-end through the full stack —
from the `Deployer` facade down to the file-based repository — but without
any infrastructure provisioning.

**Example test outline:**

```rust
// packages/sdk/tests/local_operations.rs
use torrust_tracker_deployer_sdk::{
    Deployer, EnvironmentCreationConfigBuilder, EnvironmentName,
};
use tempfile::TempDir;

#[test]
fn it_should_create_and_show_an_environment() {
    let workspace = TempDir::new().unwrap();
    let deployer = Deployer::builder()
        .working_dir(workspace.path())
        .build()
        .unwrap();

    let config = EnvironmentCreationConfigBuilder::default()
        .tracker_domain("example.com")
        // ... minimal required fields ...
        .build()
        .unwrap();

    let name = deployer.create_environment(config).unwrap();
    let info = deployer.show(&name).unwrap();
    assert_eq!(info.name, name.as_str());
}

#[test]
fn it_should_return_error_when_creating_duplicate_environment() {
    // Create once, create again with same name → error
}

#[test]
fn it_should_list_environments_in_workspace() {
    // Create 2 environments → list → assert both present
}

#[test]
fn it_should_report_exists_correctly() {
    // exists before create → false; after create → true
}

#[test]
fn it_should_purge_environment_completely() {
    // Create → purge → exists returns false
}

#[test]
fn it_should_validate_config_file() {
    // Write valid JSON → validate → assert success
}
```

**Pros:**

- Tests the SDK exactly as consumers see it — only public API, no internal
  imports
- Catches regressions in re-exports, builder wiring, and error mapping
- Runs fast (no infrastructure, no Docker, no network)
- CI-compatible — runs on every push/PR
- Follows the same pattern as `tests/e2e/` but through the SDK instead of
  CLI process invocations
- Validates that the `Deployer` facade correctly delegates to handlers
  and that results flow back through the SDK types

**Cons:**

- Does not test infrastructure operations (provision, configure, release,
  run, test) — those require LXD or Docker
- Some overlap with CLI E2E tests for local operations (both exercise the
  same underlying handlers)
- Needs `tempfile` as a dev-dependency in the SDK package

**Effort**: Low — 1-2 files, ~200-400 lines of test code.

---

### Proposal 2: SDK Compile-Time Contract Tests (Complementary)

**What**: Add `#[cfg(test)]` unit tests in `packages/sdk/src/lib.rs` that
verify compile-time guarantees about the SDK's public API. These are not
behavioral tests — they assert that the SDK exports the right types and
that those types implement the right traits.

**Example test outline:**

```rust
// packages/sdk/src/lib.rs
#[cfg(test)]
mod api_contract_tests {
    use super::*;

    #[test]
    fn it_should_export_all_command_error_types() {
        // Instantiate each error type to prove it's exported
        fn assert_error<T: std::error::Error>() {}
        assert_error::<CreateCommandHandlerError>();
        assert_error::<DestroyCommandHandlerError>();
        assert_error::<ProvisionCommandHandlerError>();
        // ... all error types ...
    }

    #[test]
    fn it_should_export_result_types() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<EnvironmentList>();
        assert_debug::<EnvironmentInfo>();
        assert_debug::<TestResult>();
        assert_debug::<ValidationResult>();
    }

    #[test]
    fn it_should_export_application_layer_error_wrappers() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<PersistenceError>();
        assert_error::<InvalidStateError>();
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<ReleaseWorkflowStep>();
    }

    #[test]
    fn sdk_error_should_convert_from_all_command_errors() {
        fn assert_from<F, T: From<F>>() {}
        assert_from::<CreateCommandHandlerError, SdkError>();
        assert_from::<DestroyCommandHandlerError, SdkError>();
        // ... etc ...
    }

    #[test]
    fn deployer_builder_should_fail_without_working_dir() {
        let result = Deployer::builder().build();
        assert!(matches!(
            result,
            Err(DeployerBuildError::MissingWorkingDir)
        ));
    }
}
```

**Pros:**

- Catches accidental removal of re-exports or trait implementations
- Zero runtime cost — compiler-only assertions
- Very low maintenance — changes only when the public API changes
- Complements the existing `Clone + Send + Sync` test in `deployer.rs`
- Fast — no file I/O, no temporary directories

**Cons:**

- Does not test behavior — only that types exist and implement traits
- Cannot catch logic bugs in delegation or error mapping
- Limited value on its own (best paired with Proposal 1)

**Effort**: Very low — ~50-100 lines in `lib.rs`.

---

### Proposal 3: SDK E2E Tests with Docker (Full Stack)

**What**: Create an E2E test binary (or integration test) that exercises
the full SDK lifecycle including infrastructure operations, using the same
Docker-based approach as `e2e_deployment_workflow_tests.rs`. The test would
use the SDK API instead of CLI process invocations.

**Example test outline:**

```rust
// packages/sdk/tests/full_deployment.rs
// or src/bin/e2e_sdk_deployment_tests.rs

#[tokio::test]
async fn it_should_complete_full_deployment_workflow_via_sdk() {
    let workspace = setup_workspace_with_docker_container().await;
    let deployer = Deployer::builder()
        .working_dir(&workspace.path)
        .build()
        .unwrap();

    // Create
    let name = deployer.create_environment(config).unwrap();

    // Register pre-existing container
    deployer.register(&name, container_ip).unwrap();

    // Configure → Release → Run → Test
    deployer.configure(&name).unwrap();
    deployer.release(&name).await.unwrap();
    deployer.run_services(&name).unwrap();
    let result = deployer.test(&name).await.unwrap();
    assert!(result.passed);

    // Cleanup
    deployer.destroy(&name).unwrap();
    deployer.purge(&name).unwrap();
}
```

**Pros:**

- Full confidence that the SDK works end-to-end for infrastructure
  operations
- Tests the exact workflow AI agents would use
- Catches integration issues between SDK wiring and infrastructure steps

**Cons:**

- Heavy — requires Docker, SSH containers, network setup
- Slow — minutes per run (same as existing deployment E2E tests)
- Largely duplicates `e2e_deployment_workflow_tests` (same handlers,
  same Docker setup, same Ansible playbooks — just called via SDK instead
  of CLI)
- The SDK's own logic for infrastructure operations is trivial —
  one-line delegation. The value of retesting the same infrastructure
  stack through a different entry point is low
- Maintenance burden — two parallel E2E suites that test the same stack
- The SDK does not currently expose a `register` command, so this would
  require either adding it or using LXD provisioning (which is
  CI-incompatible)

**Effort**: High — requires Docker infrastructure setup, testcontainers
integration, async test runtime, and potentially new SDK methods.

---

### Proposal 4: SDK Example Compilation as Tests

**What**: Run the existing SDK examples as CI tests. The examples already
demonstrate real usage patterns and cover error handling. Making them part
of the test suite ensures they stay compilable and correct.

This is partially done already — `.github/workflows/test-sdk-examples.yml`
runs 4 of the 5 examples. This proposal would additionally:

- Add `packages/sdk/tests/examples_compile.rs` that imports key types
  from the examples to ensure they keep working
- Or simply rely on the existing CI workflow (already in place)

**Pros:**

- Zero new test code needed — examples are already written
- Examples serve double duty as documentation and tests
- Already partially implemented via the CI workflow

**Cons:**

- Examples are not assertions — they demonstrate usage but don't assert
  invariants. If an operation silently returns wrong data, the example
  won't catch it
- The existing CI workflow already does this, so a separate test file
  would be redundant
- Examples may need environment-specific paths that break outside their
  expected context

**Effort**: Minimal — the CI workflow already exists. No code changes
needed unless we want to formalize it differently.

---

## Recommendation

**Implement Proposal 1 only.** It provides the best value for the effort:

| Aspect                        | Proposal 1 (Integration) |
| ----------------------------- | ------------------------ |
| Catches re-export regressions | ✅                       |
| Catches wiring bugs           | ✅                       |
| Catches delegation errors     | ✅                       |
| Catches error mapping bugs    | ✅                       |
| CI-compatible                 | ✅                       |
| Fast                          | ✅                       |
| Low maintenance               | ✅                       |

**Skip Proposal 2** (compile-time contract tests) — the SDK examples in
`packages/sdk/examples/` already serve as compile-time contract tests.
If a re-export is accidentally removed, the examples will fail to compile,
providing the same safety net that Proposal 2 would offer. Adding
dedicated contract tests on top of that would be redundant.

**Defer Proposal 3** (Docker E2E) — the existing CLI E2E tests already
cover the infrastructure stack. The SDK's infrastructure methods are
one-line delegations, so retesting them through Docker adds cost without
proportional value. Revisit only if the SDK gains logic beyond delegation
(e.g., retry policies, orchestration sequences, rollback).

**Proposal 4 is already done** — the CI workflow runs the examples. No
further action needed unless the workflow breaks.

## Decision

TBD — pending review and discussion.
