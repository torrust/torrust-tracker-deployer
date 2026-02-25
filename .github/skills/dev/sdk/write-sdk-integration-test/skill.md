---
name: write-sdk-integration-test
description: Guide for writing SDK integration tests using the module-per-command structure in packages/sdk/tests/local_operations/, covering shared helpers, the it_should_* naming convention, test isolation, and module registration. Triggers on "sdk integration test", "write sdk test", "test sdk method", "add sdk test", "integration test for sdk", "test deployer method", or "sdk test module".
metadata:
  author: torrust
  version: "1.0"
---

# Write an SDK Integration Test

SDK integration tests live in `packages/sdk/tests/local_operations/` and are
the only tests written entirely against the **public** SDK API — with no
access to internal implementation details. They exercise the SDK exactly as an
external consumer would.

No infrastructure (LXD, Docker, SSH) is required for these tests; they only
cover local operations.

---

## Quick Decision Tree

- Adding tests for a **new method**? → Follow the full guide below to create a
  new module file and register it.
- Adding an **extra test case** to an existing method? → Skip to
  [Phase 3 — Write the Test Function](#phase-3-write-the-test-function).

---

## Directory Structure

```text
packages/sdk/tests/
└── local_operations/
    ├── main.rs         ← entry point: shared helpers + `mod` declarations
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

One module per SDK command. Mirror this structure when adding a new command.

---

## Phase 1: Create the Module File

Create `packages/sdk/tests/local_operations/{command}.rs`.

Use the file module template below; replace `my_command` with the command name
and write real test cases instead of the placeholder.

```rust
// packages/sdk/tests/local_operations/my_command.rs

use crate::{create_environment, deployer_in_temp_dir, minimal_config};

#[test]
fn it_should_succeed_when_environment_exists() {
    // Arrange
    let (deployer, _tmp) = deployer_in_temp_dir();
    let name = create_environment(&deployer, "my-cmd-test");

    // Act
    let result = deployer.my_command(&name);

    // Assert
    assert!(result.is_ok(), "my_command failed: {:?}", result.err());
}

#[test]
fn it_should_return_error_when_environment_does_not_exist() {
    // Arrange
    let (deployer, _tmp) = deployer_in_temp_dir();
    let name = minimal_config("nonexistent").name;

    // Act
    let result = deployer.my_command(&name);

    // Assert
    assert!(result.is_err(), "expected error for unknown environment");
}
```

### Naming convention

All test functions follow the pattern:

```text
it_should_{behavior}_when_{condition}
```

Examples:

- `it_should_return_empty_list_when_no_environments_exist`
- `it_should_return_error_when_environment_not_found`
- `it_should_succeed_when_environment_was_previously_created`

---

## Phase 2: Register the Module in `main.rs`

Add a `mod` declaration at the top of
`packages/sdk/tests/local_operations/main.rs` alongside the other module
declarations:

```rust
// packages/sdk/tests/local_operations/main.rs

mod builder;
mod create;
mod destroy;
mod exists;
mod list;
mod my_command;   // ← add this line, in alphabetical order
mod purge;
mod show;
mod validate;
mod workflow;
```

---

## Phase 3: Write the Test Function

### Shared helpers (from `main.rs` — import via `crate::`)

All helpers are `pub(crate)` and resolved through the `crate::` prefix inside
any module file.

| Helper                                                                     | Purpose                                                                                  |
| -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `deployer_in_temp_dir() -> (Deployer, TempDir)`                            | Fresh isolated workspace per test. **Always bind `_tmp`** — dropping it deletes the dir. |
| `minimal_config(name: &str) -> EnvironmentCreationConfig`                  | Minimal valid config using `fixtures/` SSH keys.                                         |
| `create_environment(deployer: &Deployer, name: &str) -> EnvironmentName`   | Create environment and unwrap; panics with context on failure.                           |
| `write_config_json(dir: &Path, filename: &str, env_name: &str) -> PathBuf` | Write a minimal valid JSON config to disk.                                               |
| `fixture_ssh_keys() -> (PathBuf, PathBuf)`                                 | Absolute paths to `fixtures/testing_rsa` keypair.                                        |
| `repo_root() -> PathBuf`                                                   | Absolute path to the repository root.                                                    |
| `assert_environment_exists(deployer, name)`                                | Asserts `deployer.exists(name)` is `true`.                                               |
| `assert_environment_not_exists(deployer, name)`                            | Asserts `deployer.exists(name)` is `false`.                                              |

### Isolation pattern

Each test must get its own `deployer_in_temp_dir()`. Tests **must not** share
state or depend on the execution order of other tests.

```rust
#[test]
fn it_should_return_true_when_environment_was_created() {
    // Each test gets a fresh, independent workspace
    let (deployer, _tmp) = deployer_in_temp_dir();
    let name = create_environment(&deployer, "exists-test");

    assert!(deployer.exists(&name).unwrap());
}
```

### Error-path tests

Import error types from the public SDK surface only:

```rust
use torrust_tracker_deployer_sdk::CreateCommandHandlerError;

#[test]
fn it_should_return_already_exists_when_created_twice() {
    let (deployer, _tmp) = deployer_in_temp_dir();
    let config = crate::minimal_config("dup");
    deployer.create_environment(config.clone()).unwrap();

    let result = deployer.create_environment(config);

    assert!(
        matches!(result, Err(CreateCommandHandlerError::AlreadyExists(_))),
        "expected AlreadyExists, got: {result:?}"
    );
}
```

### Workflow / multi-step tests

For tests that chain multiple operations, place them in `workflow.rs`:

```rust
#[test]
fn it_should_allow_full_local_lifecycle() {
    let (deployer, _tmp) = deployer_in_temp_dir();

    // create
    let name = create_environment(&deployer, "lifecycle");
    assert_environment_exists(&deployer, &name);

    // list
    let list = deployer.list().unwrap();
    assert_eq!(list.len(), 1);

    // show
    let info = deployer.show(&name).unwrap();
    assert_eq!(info.name, name);

    // destroy
    deployer.destroy(&name).unwrap();
    assert_environment_not_exists(&deployer, &name);
}
```

---

## Phase 4: Run the Tests

```bash
# All SDK tests
cargo test -p torrust-tracker-deployer-sdk

# Only the new module
cargo test -p torrust-tracker-deployer-sdk my_command
```

Expected: the new tests appear in the output and pass.

---

## Phase 5: Run All Checks

```bash
cargo run --bin linter all
```

Fix any linting issues (unused imports, missing doc-comments on public items)
before committing.

---

## Commit

Follow the commit conventions in `.github/skills/dev/git-workflow/commit-changes/skill.md`.
Typical subject line:

```text
test(sdk): add integration tests for my_command
```

Group in a single commit:

- `packages/sdk/tests/local_operations/my_command.rs` (new file)
- `packages/sdk/tests/local_operations/main.rs` (module declaration)
