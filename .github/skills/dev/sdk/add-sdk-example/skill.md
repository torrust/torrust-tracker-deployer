---
name: add-sdk-example
description: Guide for adding a runnable example to the Deployer SDK, covering file creation in packages/sdk/examples/, the [[example]] entry in Cargo.toml, public-API-only imports, the sdk_ naming prefix, and CI verification. Triggers on "add sdk example", "sdk example", "new sdk example", "demonstrate sdk usage", "sdk usage example", "add example to sdk", or "show sdk in action".
metadata:
  author: torrust
  version: "1.0"
---

# Add an SDK Example

SDK examples are self-contained, runnable Rust programs that show SDK
consumers how to use the public API for a specific scenario. They serve as
living documentation: every example must compile cleanly, must use only public
SDK imports, and must be registered in `Cargo.toml`.

---

## Quick Decision Tree

- Demonstrating something **already possible** with the current SDK? → Follow
  this guide.
- Demonstrating a **not-yet-implemented** feature? → Implement the feature
  first (see `.github/skills/dev/sdk/add-sdk-method/skill.md`), then return here.

---

## Existing Examples (for reference)

| Cargo name                  | File                                | Scenario                   |
| --------------------------- | ----------------------------------- | -------------------------- |
| `sdk_basic_usage`           | `examples/basic_usage.rs`           | Create, list, show, purge  |
| `sdk_full_deployment`       | `examples/full_deployment.rs`       | Full provisioning workflow |
| `sdk_error_handling`        | `examples/error_handling.rs`        | Typed error matching       |
| `sdk_create_from_json_file` | `examples/create_from_json_file.rs` | Load env from JSON file    |
| `sdk_validate_config`       | `examples/validate_config.rs`       | Config file validation     |

Read at least one of these before writing a new example.

---

## Phase 1: Create the Example File

Create `packages/sdk/examples/{scenario_name}.rs`.

### Required structure

````rust
//! Short one-line description of the example.
//!
//! Longer explanation: what this example demonstrates, what the user will
//! learn, whether infrastructure is required, etc.
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_{scenario_name}
//! ```

use torrust_tracker_deployer_sdk::{Deployer, EnvironmentCreationConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example: {Title} ===\n");

    // Step 1: Initialize the deployer
    let deployer = Deployer::builder()
        .working_dir(std::env::current_dir()?)
        .build()?;

    // Step 2: … demonstrate the scenario …

    println!("[DONE]\n");
    Ok(())
}
````

### Mandatory rules

1. **Public API only** — import exclusively from `torrust_tracker_deployer_sdk`.
   Never import from `torrust_tracker_deployer_lib` or any internal crate.

2. **`sdk_` prefix** — the Cargo example `name` must start with `sdk_`
   (e.g., `sdk_my_scenario`). This keeps all SDK examples grouped in `cargo
run --example` completions and avoids collisions with binary targets.

3. **Print progress** — every meaningful step should print a status line so
   that running the example in a terminal gives clear feedback.

4. **Documented cleanup** — if the example creates environments or files that
   are not removed by the end of `main`, add a comment explaining why (e.g.,
   "intentionally left for the user to inspect").

5. **`no_run` doc tests inside `//!` comments** — if you show a code block in
   the module doc comment that cannot be run in isolation (e.g., requires a
   workspace path), mark it ` ```rust,no_run `.

---

## Phase 2: Register the Example in `Cargo.toml`

Open `packages/sdk/Cargo.toml` and add an `[[example]]` entry at the end of
the existing block, maintaining alphabetical or logical order:

```toml
[[example]]
name = "sdk_my_scenario"
path = "examples/my_scenario.rs"
```

The `name` field determines the `--example` argument. It **must** match the
`sdk_` prefix convention.

---

## Phase 3: Verify the Example Compiles and Runs

```bash
# Compile only
cargo build --example sdk_my_scenario -p torrust-tracker-deployer-sdk

# Run
cargo run --example sdk_my_scenario -p torrust-tracker-deployer-sdk
```

Expected: the example compiles and produces readable output with no panics.

---

## Phase 4: Verify CI Coverage

Run the CI verification command to confirm the example is picked up:

```bash
cargo run --examples -p torrust-tracker-deployer-sdk
```

This builds every `[[example]]` in the package. No new workflow changes are
needed — the existing `.github/workflows/test-sdk-examples.yml` already
builds and tests all registered SDK examples on every push.

---

## Phase 5: Run All Checks

```bash
cargo run --bin linter all
```

Fix any linting issues (markdownlint in example doc comments, clippy warnings,
missing `///` on public items) before committing.

---

## Commit

Follow the commit conventions in `.github/skills/dev/git-workflow/commit-changes/skill.md`.
Typical subject line:

```text
docs(sdk): add sdk_my_scenario example
```

Group in a single commit:

- `packages/sdk/examples/my_scenario.rs` (new file)
- `packages/sdk/Cargo.toml` (`[[example]]` entry)
