# Standardize JsonView Render API

## 📋 Overview

Standardize the `JsonView::render()` API across all command view modules to use a consistent
return type and enforce the contract through a shared trait. Currently there are two
incompatible patterns in use, and no mechanism to prevent future drift.

**Target Files:**

- `src/presentation/cli/views/commands/configure/views/json_view.rs`
- `src/presentation/cli/views/commands/destroy/views/json_view.rs`
- `src/presentation/cli/views/commands/list/views/json_view.rs`
- `src/presentation/cli/views/commands/purge/views/json_view.rs`
- `src/presentation/cli/views/commands/register/views/json_view.rs`
- `src/presentation/cli/views/commands/release/views/json_view.rs`
- `src/presentation/cli/views/commands/render/views/json_view.rs`
- `src/presentation/cli/views/commands/run/views/json_view.rs`
  _(path after [`standardize-command-view-folder-structure`](standardize-command-view-folder-structure.md) is applied)_
- `src/presentation/cli/views/commands/show/views/json_view.rs`
- `src/presentation/cli/views/commands/test/views/json_view.rs`
- `src/presentation/cli/views/commands/validate/views/json_view.rs`
- Command handler files that call `JsonView::render()` (one per command)

**Also touched (trait implementation):**

- All `text_view.rs` files (add `Render<T>` impl)
- New file: `src/presentation/cli/views/render.rs` (trait definition)

**Scope:**

- Introduce a `ViewRenderError` type in the presentation layer to decouple the trait from `serde_json`
- Introduce a `Render<T>` trait using `ViewRenderError` to enforce the signature at definition time
- Standardize `JsonView::render()` return type from `String` to `Result<String, ViewRenderError>`
  across all 11 inconsistent commands, matching the pattern of `create` and `provision`
- Propagate errors at call sites in all command handlers
- Out of scope: changing the DTO structures, view output format, or command behavior

## 📊 Progress Tracking

**Total Active Proposals**: 2
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 2
**In Progress**: 0
**Not Started**: 0

### Phase Summary

- **Phase 0 - Introduce ViewRenderError + Render Trait (High Impact, Low Effort)**: ✅ 1/1 completed (100%)
- **Phase 1 - Standardize Return Type (High Impact, Medium Effort)**: ✅ 1/1 completed (100%)

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. Inconsistent Return Types Across Identical Operations

`create` and `provision` return `Result`, while the other 11 commands return `String` with an
embedded fallback:

```rust
// create / provision (correct):
pub fn render(data: &EnvironmentDetailsData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(data)
}

// all other 11 commands (inconsistent):
pub fn render(data: &ConfigureDetailsData) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|e| {
        // fallback is dead code — serde_json never fails on #[derive(Serialize)]
        serde_json::to_string_pretty(&serde_json::json!({...})).unwrap_or_else(...)
    })
}
```

The fallback code is unreachable in practice: `serde_json::to_string_pretty` on a struct
with `#[derive(Serialize)]` and no custom serializers never fails. It is dead code that
obscures the real intent and was the source of the invalid-JSON bug fixed by Copilot review.

### 2. No Enforcement of the Signature Contract

There is no trait binding the `render()` method. Each view file can independently diverge
in return type, argument type, or method name without any compile-time check. This is what
allowed the inconsistency to accumulate across 11 files without being caught.

### 3. TextView Has No Shared Interface With JsonView

`TextView::render()` and `JsonView::render()` serve the same role (transform a DTO into a
displayable string) but share no interface. The `match output_format` dispatch in every
command handler is structural repetition that a trait could formalize.

## 🚀 Refactoring Phases

---

## Phase 0: Introduce ViewRenderError + Render Trait (Highest Priority)

Introduce the error type and trait together — they are a single design decision and should
be reviewed as one unit. Low effort since no existing call sites change yet.

### Proposal #1: Add `ViewRenderError` and `Render<T>` Trait

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None
**Completed**: 2026-02-27
**Commit**: `91798861`

#### Problem

All `JsonView` and `TextView` structs implement a `render()` static method, but there is
no shared trait and no defined error type. Two issues compound:

1. Future implementors can diverge silently:

   ```rust
   // Nothing stops someone from writing this:
   pub fn render(data: &MyData) -> Vec<u8> { ... } // wrong type, no compile error
   ```

2. If the trait error type is `serde_json::Error`, `TextView` (which does pure string
   formatting) is forced to return an error type that has no semantic meaning for it.
   This leaks `serde_json` as an implementation detail into `TextView` and into the trait
   definition itself.

#### Proposed Solution

Add `ViewRenderError` and `Render<T>` together in `src/presentation/cli/views/render.rs`:

```rust
/// Error produced by a [`Render`] implementation.
///
/// Using a dedicated error type decouples the presentation layer from the
/// serialization library. If the backend ever changes, only this type and
/// its `From` impls need updating — not every call site.
#[derive(Debug, thiserror::Error)]
pub enum ViewRenderError {
    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Trait for rendering command output data into a string.
///
/// Implementors transform a DTO (`T`) into a displayable or parseable string.
/// The `Result` return type is required even for infallible renderers (e.g., `TextView`)
/// so that all renderers share a uniform interface and callers can use `?` unconditionally.
pub trait Render<T> {
    /// Render `data` into a string representation.
    ///
    /// # Errors
    ///
    /// Returns a [`ViewRenderError`] if rendering fails. Text renderers always
    /// return `Ok`; JSON renderers return `Err` only if serialization fails
    /// (which is unreachable for plain `#[derive(Serialize)]` types).
    fn render(data: &T) -> Result<String, ViewRenderError>;
}
```

Implement it for every `JsonView` and `TextView`:

```rust
// json_view.rs
impl Render<ConfigureDetailsData> for JsonView {
    fn render(data: &ConfigureDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

// text_view.rs
impl Render<ConfigureDetailsData> for TextView {
    fn render(data: &ConfigureDetailsData) -> Result<String, ViewRenderError> {
        Ok(format!(
            "Configuration completed successfully for environment '{}'",
            data.environment_name
        ))
    }
}
```

#### Rationale

`ViewRenderError` and `Render<T>` must be decided together because the error type is part
of the trait's public API. Introducing them separately would require a breaking change to
the trait when the error type is later updated. Defining `ViewRenderError` upfront means:

- `TextView` returns a semantically correct error type (not `serde_json::Error`)
- The trait does not leak `serde_json` as a dependency of the presentation interface
- `#[from] serde_json::Error` on `ViewRenderError::Serialization` means `?` still works
  in `JsonView` with zero extra boilerplate

#### Benefits

- ✅ Compile-time enforcement: new views must match the signature
- ✅ `TextView` error type is semantically correct — no serde_json dependency
- ✅ Single place to update if the serialization backend changes
- ✅ Uniform `?` usage at all call sites regardless of view type
- ✅ Communicates intent: render is a potentially-fallible operation

#### Implementation Checklist

- [x] Create `src/presentation/cli/views/render.rs` with `ViewRenderError` and `Render<T>`
- [x] Add `thiserror` as a dependency if not already present (check `Cargo.toml`)
- [x] Re-export both from `src/presentation/cli/views/mod.rs`
- [x] Add `impl Render<...> for JsonView` to all 13 `json_view.rs` files
- [x] Add `impl Render<...> for TextView` to all 13 `text_view.rs` files
- [x] Verify all tests pass
- [x] Run `cargo run --bin linter all` and fix issues

#### Testing Strategy

The trait itself needs no tests. Existing unit tests in each view file cover the render
output. Add one test per view confirming `render()` via the trait interface compiles and
returns `Ok(...)`.

---

## Phase 1: Standardize JsonView Return Type

Align all 11 inconsistent `JsonView::render()` methods with the `create`/`provision`
pattern.

### Proposal #2: Remove `unwrap_or_else` Fallbacks and Return `Result`

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposal #1 + [`standardize-command-view-folder-structure`](standardize-command-view-folder-structure.md) (must be applied first so file paths are stable)
**Completed**: 2026-02-28
**Commit**: `80616e44`

#### Problem

Eleven `JsonView::render()` methods return `String` with an unreachable `unwrap_or_else`
fallback. The fallback was:

1. Originally the source of an invalid-JSON bug (fixed in commit `2319b28f`)
2. Still dead code — `serde_json::to_string_pretty` on `#[derive(Serialize)]` never fails
3. Inconsistent with `create` and `provision` which already return `Result`

```rust
// Current (11 commands):
pub fn render(data: &ConfigureDetailsData) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|e| {
        // This branch is never reached
        serde_json::to_string_pretty(&serde_json::json!({...})).unwrap_or_else(...)
    })
}
```

#### Proposed Solution

Remove the fallback entirely and return `Result<String, ViewRenderError>` via the trait:

```rust
impl Render<ConfigureDetailsData> for JsonView {
    fn render(data: &ConfigureDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}
```

Update call sites in each command handler to propagate the error with `?`:

```rust
// Before:
OutputFormat::Json => self.progress.result(&JsonView::render(&data))?,

// After:
OutputFormat::Json => self.progress.result(&JsonView::render(&data)?)?,
```

The handler error types that don't yet have `From<ViewRenderError>` need a conversion
impl added (or use `map_err`).

#### Rationale

The `Result` return type accurately models reality (even if the error path is currently
unreachable), is consistent with how `create` and `provision` already work, and removes
dead code. Propagating errors to the handler is correct: if serialization somehow failed,
silently emitting a fallback JSON string (or an invalid one) would be worse than surfacing
an error.

#### Benefits

- ✅ Removes 11 blocks of dead code
- ✅ Consistent API across all 13 command view modules
- ✅ Errors surface to callers instead of being silently swallowed
- ✅ Reduces lines of code

#### Implementation Checklist

- [x] Update all 11 `json_view.rs` standalone `render()` methods to the `Render<T>` trait impl
- [x] Update call sites in all 11 command handlers to use `?`
- [x] Add `From<ViewRenderError>` conversions where missing in handler error types
- [x] Also update `create` and `provision` (already return `Result`) to use the trait
- [x] Verify all tests pass
- [x] Run `cargo run --bin linter all` and fix issues

#### Testing Strategy

Existing unit tests in each `json_view.rs` file need to be updated: assertions that called
`render(&data)` directly now need `.expect("render failed")` or `?`. The observable
behavior (the rendered JSON string) does not change.

---

## 📈 Timeline

- **Start Date**: 2026-02-27
- **Actual Completion**: 2026-02-28

## 🔍 Review Process

### Approval Criteria

- [x] Technical feasibility validated
- [x] Aligns with [Development Principles](../development-principles.md)
- [x] Implementation plan is clear and actionable
- [x] Priorities are correct (high-impact/low-effort first)

### Completion Criteria

- [x] All active proposals implemented
- [x] All tests passing
- [x] All linters passing
- [x] Documentation updated
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../development-principles.md)
- [Contributing Guidelines](../contributing/README.md)
- [Output Handling Guide](../contributing/output-handling.md)
- [Module Organization](../contributing/module-organization.md)
- [Standardize Command View Folder Structure](standardize-command-view-folder-structure.md) — must be completed **before** Proposal #2 of this plan

## 💡 Notes

The inconsistency was introduced gradually as each command's JSON output was added in
separate PRs (issues #371–#396). The `create` and `provision` commands were the first
to be implemented (earlier PRs #351, #353) and happened to use `Result` from the start.
Subsequent commands followed a slightly different pattern that evolved independently.
The Copilot review on PR #397 was the first mention of the inconsistency.

---

**Created**: 2026-02-27
**Last Updated**: 2026-02-28 (both proposals completed)
**Status**: ✅ Completed
