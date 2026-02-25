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

## Approach: Integration Tests Against the Public API

SDK integration tests live in `packages/sdk/tests/local_operations/` and
use the SDK as an external consumer would — importing only from the public
API (`torrust_tracker_deployer_sdk::*`).

Tests exercise local-only operations (create, show, list, exists, validate,
destroy, purge) against a temporary workspace directory. No infrastructure
(LXD, Docker, SSH) is required.

### Structure

```text
packages/sdk/tests/
└── local_operations/
    ├── main.rs         ← entry point: shared helpers + mod declarations
    ├── builder.rs      ← DeployerBuilder error cases
    ├── create.rs       ← create_environment (typed + JSON file)
    ├── destroy.rs      ← destroy
    ├── exists.rs       ← exists before/after create
    ├── list.rs         ← list (populated + empty)
    ├── purge.rs        ← purge
    ├── show.rs         ← show + not-found
    ├── validate.rs     ← validate (valid + invalid config)
    └── workflow.rs     ← chained operations end-to-end
```

One module per SDK command, mirroring the CLI E2E test structure.

### What it catches

| Aspect                   | Covered |
| ------------------------ | ------- |
| Re-export regressions    | Yes     |
| Builder wiring bugs      | Yes     |
| Delegation errors        | Yes     |
| Error mapping bugs       | Yes     |
| CI-compatible            | Yes     |
| Fast (no infrastructure) | Yes     |

### Running

```bash
# All SDK tests
cargo test -p torrust-tracker-deployer-sdk

# A specific module
cargo test -p torrust-tracker-deployer-sdk create
```

For details on writing new tests, see the
[Write SDK Integration Test](../../../.github/skills/sdk/write-sdk-integration-test/skill.md)
skill.

---

## Compile-Time Verification via Examples

The SDK examples in `packages/sdk/examples/` serve as compile-time contract
tests. If a re-export is accidentally removed, the examples fail to compile.
The CI workflow `.github/workflows/test-sdk-examples.yml` builds and runs
all registered examples on every push.

This eliminates the need for dedicated `#[cfg(test)]` contract tests that
only assert type existence.

---

## Why Not Docker E2E Tests for the SDK?

The existing CLI E2E tests already cover the full infrastructure stack
(provision, configure, release, run, test). The SDK's infrastructure
methods are one-line delegations to the same handlers, so retesting them
through Docker adds cost without proportional value.

Revisit only if the SDK gains logic beyond delegation (e.g., retry
policies, orchestration sequences, rollback).
