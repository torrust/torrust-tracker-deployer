# Switch Default Output Format from `text` to `json`

**Issue**: #398
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/roadmap.md` section 12
- Roadmap section 12.14: `docs/roadmap.md`
- `OutputFormat` definition: `src/presentation/cli/input/cli/output_format.rs`
- CLI args: `src/presentation/cli/input/cli/args.rs`

## Overview

Switch the default output format from `text` to `json` (roadmap task 12.14). This is the final task in epic #348 and requires all previous commands (12.1–12.13) to have JSON output implemented; otherwise the application panics for unimplemented commands.

**Prerequisite**: All commands must have JSON output implemented ✅ (12.1–12.13 complete).

Once merged, `--output-format json` will no longer be needed for automation workflows — JSON will be the out-of-the-box experience.

## Goals

- [ ] Move `#[default]` attribute from `OutputFormat::Text` to `OutputFormat::Json`
- [ ] Update `default_value = "text"` to `default_value = "json"` in CLI args
- [ ] Update doc comments referring to `text` as the default
- [ ] Update all doctests and documentation examples that assert `OutputFormat::default()` is `Text`
- [ ] Verify no integration tests or snapshots hardcode the old default

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Files to change**:

- `src/presentation/cli/input/cli/output_format.rs` — move `#[default]` from `Text` to `Json`
- `src/presentation/cli/input/cli/args.rs` — update `default_value` and doc comment

### Key Code Changes

#### `output_format.rs`

Before:

```rust
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    #[default]
    Text,
    /// JSON output for automation and programmatic parsing
    Json,
}
```

After:

```rust
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for automation and programmatic parsing (default)
    #[default]
    Json,
}
```

#### `args.rs`

Before:

```rust
/// Output format for command results (default: text)
// ...
#[arg(long, value_enum, default_value = "text", global = true)]
pub output_format: OutputFormat,
```

After:

```rust
/// Output format for command results (default: json)
// ...
#[arg(long, value_enum, default_value = "json", global = true)]
pub output_format: OutputFormat,
```

### Doctest Updates

The existing doctest in `output_format.rs` asserts the old default:

```rust
// Before — must be updated:
let format = OutputFormat::default();
assert!(matches!(format, OutputFormat::Text));

// After:
let format = OutputFormat::default();
assert!(matches!(format, OutputFormat::Json));
```

Any other tests or examples that construct `Args::default()` and expect `output_format: OutputFormat::Text` must also be updated to `OutputFormat::Json`.

## Specifications

### Expected Behaviour After Change

Running any command **without `--output-format`** produces JSON output:

```bash
$ torrust-tracker-deployer list
{
  "environments": []
}
```

Human-readable text output still available via explicit flag:

```bash
$ torrust-tracker-deployer list --output-format text
No environments found.
```

### Breaking Change Notice

This is a **breaking change** for users relying on unformatted text output without the `--output-format` flag. Users who want text output must now explicitly pass `--output-format text`.

## Acceptance Criteria

- [ ] `OutputFormat::default()` returns `OutputFormat::Json`
- [ ] Running any command without `--output-format` produces valid JSON to stdout
- [ ] Running with `--output-format text` still produces human-readable text
- [ ] All existing unit tests and doctests pass
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] `cargo machete` reports no unused dependencies

## Testing

- Update the doctest in `output_format.rs`
- Update any `Args::default()` usages that assert `OutputFormat::Text`
- Run `cargo test` and verify all 430+ tests pass
- Manual smoke test: run `torrust-tracker-deployer list` (no flag) and confirm JSON output
