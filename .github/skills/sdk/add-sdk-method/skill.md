---
name: add-sdk-method
description: >
  Step-by-step guide for adding a new public method to the Torrust Tracker
  Deployer SDK `Deployer` facade, covering the delegation pattern in
  `deployer.rs`, the required `SdkError` variant, public re-exports in
  `lib.rs`, and a matching integration-test module. Triggers on: "add sdk
  method", "new deployer method", "extend the sdk", "add method to deployer",
  "implement sdk operation", "expose new operation in sdk".
metadata:
  author: torrust
  version: "1.0"
---

# Add an SDK Method

Every new capability the SDK exposes follows the same four-step pattern:

1. Add the **delegation method** to `packages/sdk/src/deployer.rs`.
2. Add the **`SdkError` variant** to `packages/sdk/src/error.rs`.
3. **Re-export** every new public type through `packages/sdk/src/lib.rs`.
4. Add an **integration-test module** in
   `packages/sdk/tests/local_operations/`.

Read the existing method implementations before starting — they are the
authoritative reference for the patterns this skill describes.

---

## Quick Decision Tree

- Does an application-layer `CommandHandler` for this operation already exist
  in `torrust_tracker_deployer_lib`? → yes → follow this guide.
- Do you need a new application handler too? → create it first (see
  `.github/skills/add-new-command/skill.md`), then return here.

---

## Phase 1: Add the Delegation Method to `deployer.rs`

**Goal**: expose the operation as a `pub fn` on `Deployer` that wires up the
application handler using the deployer's internal dependencies.

### What to build

Open `packages/sdk/src/deployer.rs` and add your method following the existing
pattern. Every method:

- Creates a handler instance from `self`'s dependency fields.
- Calls `handler.execute(…)`.
- Maps the result to a public SDK type.

```rust
// packages/sdk/src/deployer.rs

use torrust_tracker_deployer_lib::application::command_handlers::my_command::{
    MyCommandHandler, MyCommandHandlerError,
};

// Inside `impl Deployer`:

/// One-sentence description of what this method does.
///
/// Equivalent to `torrust-tracker-deployer my-command [args]`.
///
/// # Errors
///
/// Returns [`MyCommandHandlerError`] if …
pub fn my_operation(
    &self,
    name: &EnvironmentName,
) -> Result<MyOutput, MyCommandHandlerError> {
    let handler = MyCommandHandler::new(
        self.repository.clone() as Arc<dyn EnvironmentRepository>,
    );
    handler.execute(name, &self.working_dir)
}
```

Infrastructure operations (provision, configure, release, run, test) receive
additional dependencies (`repository_factory`, `data_directory`, `listener`).
Compare with `provision()` or `configure()` in the same file for that pattern.

### Test

```bash
cargo build -p torrust-tracker-deployer-sdk
```

Expected: compiles without errors (the handler error type is not yet in
`SdkError`, so `cargo test` may still fail — that is addressed in Phase 2).

---

## Phase 2: Add the `SdkError` Variant and Re-exports

**Goal**: surface the new error type and any new public types through the SDK's
unified API.

### 2a — `SdkError` variant

Open `packages/sdk/src/error.rs` and add a variant:

```rust
// packages/sdk/src/error.rs

/// Error returned by [`Deployer::my_operation`].
#[error(transparent)]
MyOperation(#[from] MyCommandHandlerError),
```

Follow the `#[from]` pattern used by all other variants so `?` works
transparently in `deployer.rs`.

### 2b — Re-exports in `lib.rs`

Open `packages/sdk/src/lib.rs` and re-export every new type that appears in
method signatures (inputs **and** outputs). Do not export internal
implementation types.

```rust
// packages/sdk/src/lib.rs

pub use torrust_tracker_deployer_lib::application::command_handlers::my_command::{
    MyCommandHandlerError,
    MyOutput,
};
```

### Test

```bash
cargo build -p torrust-tracker-deployer-sdk
cargo test -p torrust-tracker-deployer-sdk --doc
```

Expected: library compiles; doc-tests in module-level examples pass.

---

## Phase 3: Add the Integration-Test Module

**Goal**: prove the new method works end-to-end using only the public SDK API.

For full guidance on the test structure and shared helpers, load the companion
skill: `.github/skills/sdk/write-sdk-integration-test/skill.md`.

### Quick summary

1. Create the file: `packages/sdk/tests/local_operations/my_command.rs`

2. Every test function uses the naming convention:
   `it_should_{behavior}_when_{condition}`.

3. Use the shared helpers from
   `packages/sdk/tests/local_operations/main.rs`:
   - `deployer_in_temp_dir()` — isolated workspace per test.
   - `minimal_config(name)` — minimal `EnvironmentCreationConfig`.
   - `create_environment(deployer, name)` — create and unwrap.
   - `assert_environment_exists(deployer, name)` /
     `assert_environment_not_exists(deployer, name)`.

4. Declare the module in `main.rs`:

   ```rust
   // packages/sdk/tests/local_operations/main.rs
   mod my_command;
   ```

Minimal example module:

```rust
// packages/sdk/tests/local_operations/my_command.rs

use crate::{create_environment, deployer_in_temp_dir};

#[test]
fn it_should_succeed_when_environment_exists() {
    let (deployer, _tmp) = deployer_in_temp_dir();
    let name = create_environment(&deployer, "my-op-test");

    let result = deployer.my_operation(&name);

    assert!(result.is_ok(), "my_operation failed: {:?}", result.err());
}
```

### Test

```bash
cargo test -p torrust-tracker-deployer-sdk
```

Expected: all existing tests plus the new ones pass.

---

## Phase 4: Run All Checks

```bash
cargo run --bin linter all
cargo test
```

Fix any linting issues before committing.

---

## Commit

Follow the commit conventions in `.github/skills/commit-changes/skill.md`.
Typical subject line:

```text
feat(sdk): add Deployer::my_operation method
```

Group in a single commit:

- `packages/sdk/src/deployer.rs`
- `packages/sdk/src/error.rs`
- `packages/sdk/src/lib.rs`
- `packages/sdk/tests/local_operations/main.rs`
- `packages/sdk/tests/local_operations/my_command.rs`
