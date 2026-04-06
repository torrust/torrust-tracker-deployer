# Replace Local `packages/linting` with Published `torrust-linting` Crate

**Issue**: #416
**Parent Epic**: None
**Related**: None

## Overview

The `packages/linting` workspace package has been extracted and published to
[crates.io as `torrust-linting`](https://crates.io/crates/torrust-linting)
(v0.1.0). This makes it an independently versioned and reusable library for any
Torrust project.

This task replaces the local path dependency with the published crate and removes
the now-redundant workspace member.

## Goals

- [ ] Replace `torrust-linting = { path = "packages/linting" }` with `torrust-linting = "0.1.0"` in `Cargo.toml`
- [ ] Remove `"packages/linting"` from the workspace `members` list in `Cargo.toml`
- [ ] Delete the `packages/linting/` directory
- [ ] Update documentation that references `packages/linting/`
- [ ] Update `packages/README.md` to reflect that `torrust-linting` is now an external dependency
- [ ] Verify the build and all tests pass after the change

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (build infrastructure / workspace configuration)
**Module Path**: N/A
**Pattern**: External crate dependency substitution

### Architectural Constraints

- [ ] No changes to `src/` code — the public API of `torrust-linting` on crates.io matches the local package

### Anti-Patterns to Avoid

- ❌ Keeping the local package in the workspace after migrating (dead code / confusion)
- ❌ Updating `src/bin/linter.rs` imports — the API is identical and no source changes are needed

## Specifications

### Cargo.toml Changes

**Before**:

```toml
[workspace]
members = [
  "packages/linting",
  "packages/dependency-installer",
  "packages/sdk",
  "packages/deployer-types",
]

[dependencies]
torrust-linting = { path = "packages/linting" }
```

**After**:

```toml
[workspace]
members = [
  "packages/dependency-installer",
  "packages/sdk",
  "packages/deployer-types",
]

[dependencies]
torrust-linting = "0.1.0"
```

### Directory Removal

Delete `packages/linting/` in its entirety (the code is now maintained in the
upstream [torrust/torrust-linting](https://github.com/torrust/torrust-linting)
repository).

### Documentation Updates

Files that reference `packages/linting` and will need updating:

| File | Change |
|------|--------|
| `packages/README.md` | Remove `packages/linting` entry; add note that `torrust-linting` is an external crate |
| `docs/codebase-architecture.md` | Update linting section to reference the external crate |
| `.github/skills/dev/git-workflow/run-linters/skill.md` | Update link to linting framework |
| `.github/skills/dev/git-workflow/run-linters/references/linters.md` | Update package location description |

## Implementation Plan

### Phase 1: Update Cargo workspace and dependency (< 30 min)

- [ ] Task 1.1: Remove `"packages/linting"` from `[workspace] members` in `Cargo.toml`
- [ ] Task 1.2: Replace path dependency with `torrust-linting = "0.1.0"` in `[dependencies]`
- [ ] Task 1.3: Run `cargo build` and `cargo test` to confirm no compilation errors

### Phase 2: Remove local package (< 15 min)

- [ ] Task 2.1: Delete `packages/linting/` directory
- [ ] Task 2.2: Run `cargo build` and `cargo test` again to confirm clean build after deletion

### Phase 3: Update documentation (< 30 min)

- [ ] Task 3.1: Update `packages/README.md` — remove local package entry, document as external
- [ ] Task 3.2: Update `docs/codebase-architecture.md` — linting section
- [ ] Task 3.3: Update `.github/skills/` references to `packages/linting`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR to minimize
> back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `packages/linting/` directory no longer exists in the repository
- [ ] `Cargo.toml` workspace `members` does not include `"packages/linting"`
- [ ] `Cargo.toml` dependency is `torrust-linting = "0.1.0"` (crates.io)
- [ ] `cargo build` and `cargo test` pass with no errors
- [ ] No remaining references to `packages/linting` in source or documentation

## Related Documentation

- [crates.io: torrust-linting](https://crates.io/crates/torrust-linting)
- [GitHub: torrust/torrust-linting](https://github.com/torrust/torrust-linting)
- [packages/README.md](../../packages/README.md)
- [docs/codebase-architecture.md](../codebase-architecture.md)

## Notes

The published crate API is identical to the local package — `src/bin/linter.rs`
and all other callers require no changes. Only the Cargo configuration and
package directory are affected.
