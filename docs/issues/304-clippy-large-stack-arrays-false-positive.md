# Issue #304: Clippy `large_stack_arrays` False Positive with `vec![]` Macro

**GitHub Issue**: <https://github.com/torrust/torrust-tracker-deployer/issues/304>

## Problem

Clippy reports a false positive `large_stack_arrays` lint when using the `vec![]` macro
to create vectors of `ServiceTopology` items in tests.

### Error Message

```text
error: allocating a local array larger than 16384 bytes
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#large_stack_arrays
  = note: `-D clippy::large-stack-arrays` implied by `-D clippy::pedantic`
  = help: to override `-D clippy::pedantic` add `#[allow(clippy::large_stack_arrays)]`
```

### Root Cause

This is a known clippy bug where the `vec![]` macro is incorrectly flagged for creating
a large stack array. The `vec![]` macro creates a `Vec<T>` which allocates on the heap,
not the stack. The lint is a false positive.

**Upstream Issue**: <https://github.com/rust-lang/rust-clippy/issues/12586>

### Affected Code

Tests in `src/domain/topology/aggregate.rs` that create vectors of `ServiceTopology`:

```rust
let topology = DockerComposeTopology::new(vec![
    ServiceTopology::with_networks(Service::Tracker, vec![Network::Database]),
    ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
])
.unwrap();
```

### Workarounds Attempted (Did Not Work)

1. **Outer attribute on module**: `#[allow(clippy::large_stack_arrays)]` on `mod tests`
2. **Inner attribute on submodule**: `#![allow(clippy::large_stack_arrays)]` inside submodules
3. **Outer attribute on test function**: `#[allow(clippy::large_stack_arrays)]` on `#[test]` functions
4. **Inner attribute in function body**: `#![allow(clippy::large_stack_arrays)]` inside function

None of these local suppression methods worked because the lint fires during macro
expansion before the allow attributes are processed.

## Solution

Add a **crate-level allow attribute** in `src/lib.rs`:

```rust
// False positive: clippy reports large_stack_arrays for vec![] macro with ServiceTopology
// This is a known issue: https://github.com/rust-lang/rust-clippy/issues/12586
#![allow(clippy::large_stack_arrays)]
```

This is the only approach that successfully suppresses the false positive.

### Trade-offs

- **Downside**: Suppresses the lint crate-wide, potentially hiding legitimate issues
- **Mitigation**: This lint is specifically for stack allocations. Since we use `Vec<T>`
  (heap-allocated) throughout the codebase, legitimate triggers are unlikely
- **Future**: Remove this allow once the upstream clippy issue is fixed

## Affected Rust Versions

- Rust 1.93.0 stable (confirmed on GitHub Actions CI)
- Local nightly 1.95.0 does **not** reproduce the issue

### Fix Timeline

| Event                                                                                          | Date           | Version                            |
| ---------------------------------------------------------------------------------------------- | -------------- | ---------------------------------- |
| Fix merged to clippy master ([PR #12624](https://github.com/rust-lang/rust-clippy/pull/12624)) | April 27, 2024 | -                                  |
| Rust 1.79.0 released                                                                           | June 13, 2024  | Nightly at merge time              |
| Rust 1.80.0 released                                                                           | July 25, 2024  | **Expected first stable with fix** |

### Potential Regression

The fix was merged in April 2024 and should be in Rust 1.80.0+. However, we're still
seeing this error on Rust 1.93.0 (January 2026). This suggests either:

1. **Regression**: The bug may have regressed in a later clippy version
2. **Different code pattern**: Our `vec![]` with `ServiceTopology` (large struct with
   `EnumSet` fields) might trigger a variant not covered by the original fix
3. **CI environment**: Some discrepancy in the clippy version used in CI

Consider reporting this as a potential regression if the issue persists after verifying
the clippy version matches the expected behavior.

## Related Clippy Issues

Other `large_stack_arrays` issues (not directly related to our problem):

- [#13774](https://github.com/rust-lang/rust-clippy/issues/13774) - No span for error (closed)
- [#13529](https://github.com/rust-lang/rust-clippy/issues/13529) - Nested const items (closed)
- [#9460](https://github.com/rust-lang/rust-clippy/issues/9460) - Static struct (closed)
- [#4520](https://github.com/rust-lang/rust-clippy/issues/4520) - Original lint creation (open)

## References

- Upstream clippy issue: <https://github.com/rust-lang/rust-clippy/issues/12586>
- Related PR: #303 (Phase 4 Service Topology DDD Alignment)

## Labels

- `bug`
- `workaround`
- `clippy`
- `technical-debt`
