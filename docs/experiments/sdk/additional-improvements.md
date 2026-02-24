# SDK — Additional Improvement Ideas

Beyond the [Phase 2 tasks](phase-2-improvements.md) (Tasks 1–11), here are
additional ideas identified while working on the SDK implementation.

## High Value / Low Effort

### Integration Tests for the SDK Facade

Today the only SDK test is the `it_should_be_clone_send_and_sync` unit test.
The `basic_usage` example exercises the happy path but is not an automated
test. A small `tests/sdk/` integration test that does the full local cycle
(create → list → show → exists → purge) against a temp workspace would
catch regressions and serve as living documentation.

### Configurable Lock Timeout in `DeployerBuilder`

The builder currently hard-codes a 30-second file-lock timeout. CI, tests,
and production have very different needs. Exposing a
`.lock_timeout(Duration)` method is a one-line addition that unblocks
callers with tight latency requirements or slow CI runners.

### `Deployer::create_environment_from_json(&str)` Convenience Method

`EnvironmentCreationConfig::from_json` exists, but there is no matching
wrapper on `Deployer` itself (unlike `create_environment_from_file`).
Adding a symmetric `create_environment_from_json` keeps the facade
consistent and makes one-liner usage easier for AI agents that already have
the JSON string in memory.

### `#[non_exhaustive]` on Public Enums Now

Task 11 mentions adding `#[non_exhaustive]` when extracting the SDK crate,
but adding it early to `SdkError`, `CreateEnvironmentFromFileError`,
`ConfigLoadError`, and `EnvironmentCreationConfigBuildError` prevents
breaking downstream consumers whenever a new variant is added. This is
cheaper to do now than later.

## Medium Value

### Typed State Transitions on the Facade

The domain already has phantom state types (`Created`, `Provisioned`,
`Configured`, `Released`, `Running`, `Destroyed`). Once the async operations
are added, the SDK could enforce the lifecycle at the type level:

```rust
// Only accepts Created, returns Provisioned
async fn provision(&self, env: &Environment<Created>) -> Result<Environment<Provisioned>, ...>
```

This makes illegal state transitions a compile error.

### `tracing` Instrumentation

Adding `#[instrument]` spans to SDK methods gives consumers automatic
structured logging without custom progress listeners. This complements (but
does not replace) the progress listener from Task 8.

### `DeployerBuilder::data_directory()` Override

Currently the data directory is always `working_dir.join("data")`. Some
users may want a custom path — for example a shared NFS location or a
temporary directory for tests.

### Render Operation on the SDK

The CLI has a `render` command (already async). It is useful for inspecting
generated configuration files without actually provisioning. This could be
added alongside the other async operations in Task 9.

### Idempotent Deploy Example

An `examples/sdk/idempotent_deploy.rs` that uses `exists()` + `show()` to
check environment state before each step, skipping already-completed stages.
Demonstrates how to build resilient automation that can resume after
interruptions — a more advanced pattern but very practical for production
use.

### Custom Progress Listener Example

An `examples/sdk/custom_progress.rs` showing a richer
`CommandProgressListener` implementation — for example writing to a log
file, updating a progress bar, or collecting step timings. The current
`PrintProgressListener` in `full_deployment.rs` is minimal; a more
realistic example would help users building UIs or dashboards on top of
the SDK.

## Lower Priority / Future

### SDK Changelog / Migration Guide

Once there are breaking changes between phases, a `CHANGELOG.md` or
migration document within `docs/experiments/sdk/` would help early adopters
track what changed.

### Feature Flag Gating

A `sdk` cargo feature that gates the `presentation::sdk` module, so users
who only need the library internals do not pull in the SDK surface area.
