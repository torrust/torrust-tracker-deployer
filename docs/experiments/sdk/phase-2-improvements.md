# SDK Next Steps: Implementation Plan

> **Status**: Post-PoC planning. The initial proof of concept is complete
> (see [phase-1-poc.md](phase-1-poc.md)). This document
> describes the next improvements, ordered from simplest to most complex.

## Context

The PoC delivered a working `Deployer` facade with 6 operations
(`create_environment`, `show`, `list`, `validate`, `destroy`, `purge`),
a `DeployerBuilder`, curated re-exports, and a running example.

This plan addresses the gaps discovered during the PoC, prioritized for
AI agent use cases — the primary motivation for the SDK.

## Task List

### Task 1: Make `Deployer` Clone + Send + Sync ✅

**Complexity**: Trivial
**Why**: AI agents managing multiple environments concurrently need to
share the `Deployer` across threads. All inner fields are already `Arc`,
so this is a one-line change.

**Work**:

- Derive `Clone` on `Deployer`
- Add compile-time assertions for `Send + Sync`
- Add a unit test confirming the bounds

### Task 2: Add `EnvironmentCreationConfig::from_json` helper ✅

**Complexity**: Trivial
**Why**: The example currently requires consumers to depend on `serde_json`
directly just to deserialize a config string. A convenience method hides
that transitive dependency.

**Work**:

- Add `EnvironmentCreationConfig::from_json(s: &str) -> Result<Self, ...>`
- Add `EnvironmentCreationConfig::from_file(path: &Path) -> Result<Self, ...>`
- Re-export both from the SDK module
- Update the example to use `from_json` instead of `serde_json::from_str`

### Task 3: Add `create_environment_from_file` to `Deployer` ✅

**Complexity**: Simple
**Why**: The CLI's primary flow is `--env-file <path>`. The SDK should
mirror it. Currently consumers must read the file, parse JSON, and call
`create_environment` — three steps that should be one.

**Work**:

- Add `Deployer::create_environment_from_file(&self, path: &Path)` method
- Internally: read file, deserialize, delegate to `create_environment`
- Return a clear error if the file doesn't exist or is malformed
- Add a second example or extend the existing one

### Task 4: Improve name reuse after `create_environment` ✅

**Complexity**: Simple
**Why**: After `create_environment` returns `Environment<Created>`, the
consumer must manually reconstruct `EnvironmentName::new("sdk-example")`
to call `show` or `purge`. This is redundant — the name is already inside
the returned environment.

**Work**:

- Ensure `Environment<Created>` exposes `.name() -> &EnvironmentName`
  (it may already — verify and document)
- Update the example to reuse the returned name:

  ```rust
  let env = deployer.create_environment(config)?;
  let info = deployer.show(env.name())?;
  deployer.purge(env.name())?;
  ```

- If `show`/`purge`/`destroy` accept `&EnvironmentName` but
  `Environment::name()` returns `&EnvironmentName`, this should
  already work — just update the example

### Task 5: Add `Deployer::exists` method ✅

**Complexity**: Simple
**Why**: AI agents frequently need to check whether an environment exists
before deciding to create or destroy it. Currently they must call `show`
and match on the error, which is awkward.

**Work**:

- Add `Deployer::exists(&self, name: &EnvironmentName) -> Result<bool, ...>`
- Internally: delegate to `show`, map `NotFound` to `Ok(false)`, `Ok(_)`
  to `Ok(true)`, propagate other errors
- Update the AI agent workflow example in the docs

### Task 6: Introduce a unified `SdkError` enum ✅

**Complexity**: Medium
**Why**: Each `Deployer` method returns a different error type. Consumers
using `Box<dyn Error>` lose the ability to match on variants. A unified
enum enables programmatic error recovery without type erasure.

**Work**:

- Create `src/presentation/sdk/error.rs`
- Define `SdkError` with one variant per operation:

  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum SdkError {
      #[error(transparent)]
      Build(#[from] DeployerBuildError),
      #[error(transparent)]
      Create(#[from] CreateCommandHandlerError),
      #[error(transparent)]
      Show(#[from] ShowCommandHandlerError),
      #[error(transparent)]
      List(#[from] ListCommandHandlerError),
      #[error(transparent)]
      Validate(#[from] ValidateCommandHandlerError),
      #[error(transparent)]
      Destroy(#[from] DestroyCommandHandlerError),
      #[error(transparent)]
      Purge(#[from] PurgeCommandHandlerError),
  }
  ```

- **Do not change** existing method signatures (that would be breaking).
  Instead, provide `SdkError` as an opt-in unified type that consumers
  can convert into via `.into()` or `?` with a `From` impl
- Re-export from `sdk/mod.rs`

### Task 7: Add `EnvironmentCreationConfigBuilder` ✅

**Complexity**: Medium
**Why**: The example builds config by hand-crafting a JSON string — fragile,
verbose, and error-prone. A typed builder eliminates JSON string
manipulation entirely, which is the single biggest ergonomic improvement
for AI agent consumers.

**Work**:

- Create a builder for `EnvironmentCreationConfig` with methods like:

  ```rust
  EnvironmentCreationConfig::builder()
      .name("my-tracker")
      .ssh_keys("/path/to/key", "/path/to/key.pub")
      .provider_lxd("profile-name")
      .sqlite("tracker.db")
      .udp("0.0.0.0:6969")
      .http("0.0.0.0:7070")
      .api("0.0.0.0:1212", "MyToken")
      .build()?
  ```

- Support both LXD and Hetzner providers
- Support both SQLite and MySQL database drivers
- Validate required fields at build time
- Re-export the builder from `sdk/mod.rs`
- Update the example to use the builder instead of raw JSON

### Task 8: Add progress listener support to `DeployerBuilder`

**Complexity**: Medium
**Why**: Long-running operations (provision, configure, release) produce
step-by-step progress events. AI agents need to react to these in
real-time — e.g., logging, retrying, or aborting. The extension point
(`CommandProgressListener`) is already re-exported but the `Deployer` has
no way to wire one in.

**Work**:

- Add `.progress_listener(listener)` to `DeployerBuilder`
- Store `Arc<dyn CommandProgressListener>` in `Deployer`
  (default: `NullProgressListener`)
- Pass the listener to command handlers that accept one
- Add an example showing a custom listener that prints progress

### Task 9: Add async operations (provision, configure, release, run) ✅

**Complexity**: High
**Why**: These are the core deployment operations that make the SDK useful
for real workflows. They require real infrastructure (LXD/SSH), are
long-running, and some command handlers are already async.

**Work**:

- Add `async fn provision(&self, name: &EnvironmentName) -> Result<Environment<Provisioned>, ...>`
- Add `async fn configure(&self, name: &EnvironmentName) -> Result<Environment<Configured>, ...>`
- Add `async fn release(&self, name: &EnvironmentName) -> Result<Environment<Released>, ...>`
- Add `async fn run(&self, name: &EnvironmentName) -> Result<Environment<Running>, ...>`
- Add `async fn test(&self, name: &EnvironmentName) -> Result<..., ...>`
- Decide: should the `Deployer` carry a tokio runtime handle, or should
  all methods be async and let the consumer own the runtime?
- Wire progress listener into each handler
- Add an example: `examples/sdk/full_deployment.rs` (requires LXD)

### Task 10: Scoped environment guard (create + auto-purge on drop)

**Complexity**: Medium
**Why**: Examples and test harnesses that create temporary environments
need cleanup guarantees. A RAII guard prevents leaked environments.

**Work**:

- Create `ScopedEnvironment` struct that wraps `Environment<Created>`
  and holds a reference to `Deployer`
- On `Drop`, call `deployer.purge(name)`
- Add `Deployer::scoped_environment(config) -> Result<ScopedEnvironment, ...>`
- Handle drop errors gracefully (log, don't panic)
- Add an example demonstrating the guard pattern

### Task 11: Extract into a separate crate

**Complexity**: High
**Why**: Once the API stabilizes, a separate `torrust-tracker-deployer-sdk`
crate enables independent versioning, smaller dependency footprint for
SDK-only consumers, and cleaner public API boundaries.

**Work**:

- Create `packages/sdk/` as a workspace member
- Move SDK types from `src/presentation/sdk/` to the new crate
- Re-export domain types via the SDK crate's public API
- Keep the deployer lib crate as a dependency of the SDK crate
- Add `#[non_exhaustive]` to public enums and structs
- Set up independent semver versioning
- Publish to crates.io (or keep private, depending on project policy)

## Summary

| #   | Task                              | Complexity | Improves           | Status |
| --- | --------------------------------- | ---------- | ------------------ | ------ |
| 1   | Make Deployer Clone+Send+Sync     | Trivial    | Concurrency        | Done   |
| 2   | Config from_json / from_file      | Trivial    | Ergonomics         | Done   |
| 3   | create_environment_from_file      | Simple     | Ergonomics         | Done   |
| 4   | Improve name reuse in example     | Simple     | Ergonomics         | Done   |
| 5   | Add exists() method               | Simple     | AI agent workflows | Done   |
| 6   | Unified SdkError enum             | Medium     | Error handling     | Done   |
| 7   | EnvironmentCreationConfigBuilder  | Medium     | Ergonomics         | Done   |
| 8   | Progress listener in builder      | Medium     | Observability      |        |
| 9   | Async operations (provision, etc) | High       | Full workflow      | Done   |
| 10  | Scoped environment guard          | Medium     | Cleanup safety     |        |
| 11  | Extract into separate crate       | High       | API stability      |        |
