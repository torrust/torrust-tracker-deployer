# Linter Parallel Execution Feature Specification

## 📋 Overview

Optimize linter execution time by running linters in parallel instead of sequentially. This would reduce total execution time from ~13 seconds to ~9 seconds (~30% faster).

## 🎯 Goals

- **Improve performance**: Reduce total linter execution time by running compatible linters in parallel
- **Maintain clean output**: Ensure error messages remain readable and properly grouped by linter
- **Safe execution**: Prevent file conflicts between linters that modify the same files
- **Preserve functionality**: Work correctly with both check-only and auto-fix modes

## 📊 Current State

**Sequential Execution** (current implementation):

```rust
// From packages/linting/src/cli.rs
pub fn run_all_linters() -> Result<()> {
    info!("Running All Linters");

    let mut failed = false;

    match run_markdown_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Markdown linting failed: {e}");
            failed = true;
        }
    }

    match run_yaml_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("YAML linting failed: {e}");
            failed = true;
        }
    }
    // ... continues sequentially for all linters
}
```

**Execution Time**:

- markdown: ~1s
- yaml: ~1s
- toml: ~0.5s
- clippy: ~5s
- rustfmt: ~2s
- shellcheck: ~0.5s
- cspell: ~2s
- **Total: ~13s** (sequential)

## 🚀 Proposed Solution

### Parallel Execution Strategy

Run linters in groups based on file type conflicts:

**Group 1 (Parallel)**: Non-conflicting linters

- markdown (`*.md`)
- yaml (`*.yml`, `*.yaml`)
- toml (`*.toml`)
- shellcheck (`*.sh`)
- rustfmt (`*.rs`) - Can run in parallel since only modifies Rust files

**Group 2 (Sequential, if auto-fix enabled)**: Clippy

- clippy (`*.rs`) - Must run after Group 1 if auto-fix is enabled to avoid conflicts with rustfmt

**cspell (Separate group)**: Read-only checker

- cspell (all text files) - Can run separately since it doesn't modify any files

### Why Parallel Execution is Safe

| Linter     | File Types        | Modifies Files? | Can Run in Parallel?               |
| ---------- | ----------------- | --------------- | ---------------------------------- |
| markdown   | `*.md`            | ✅ Yes          | ✅ Yes - unique file type          |
| yaml       | `*.yml`, `*.yaml` | ✅ Yes          | ✅ Yes - unique file type          |
| toml       | `*.toml`          | ✅ Yes          | ✅ Yes - unique file type          |
| rustfmt    | `*.rs`            | ✅ Yes          | ✅ Yes - only modifies Rust files  |
| clippy     | `*.rs`            | ✅ Yes          | ⚠️ Run after rustfmt if auto-fix   |
| shellcheck | `*.sh`            | ❌ No           | ✅ Yes - read-only                 |
| cspell     | All text files    | ❌ No           | ✅ Yes - read-only, separate group |

**Key Insight**: Different linters modify different file extensions, so they can safely run in parallel without file conflicts.

**Updated Strategy**: rustfmt can run in parallel with other linters in Group 1. Only clippy needs to run sequentially after rustfmt if auto-fix is enabled, to avoid conflicts.

### Expected Performance

**Parallel execution time**:

- Group 1 (parallel): max(1s markdown, 1s yaml, 0.5s toml, 0.5s shellcheck, 2s rustfmt) = **~2s**
- cspell (separate): **~2s** (can run concurrently with Group 1)
- Group 2 (sequential, if auto-fix): 5s clippy = **~5s**
- **Total: ~7s** (46% faster than current 13s)

**Note**: Performance gain is even better than initially estimated due to updated grouping strategy.

## 🚧 Implementation Challenges

### Challenge 1: Output Handling

**Problem**: Current linters print errors **immediately** using `println!()` and `eprintln!()`

```rust
// Current implementation (from markdown.rs)
if output.status.success() {
    info!(target: "markdown", "All markdown files passed linting!");
    Ok(())
} else {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Prints immediately - would be interleaved in parallel execution!
    if !stdout.is_empty() {
        println!("{stdout}");
    }
    if !stderr.is_empty() {
        eprintln!("{stderr}");
    }

    error!(target: "markdown", "Markdown linting failed. Please fix the issues above.");
    Err(anyhow::anyhow!("Markdown linting failed"))
}
```

**Impact**: If linters run in parallel and print immediately, output would be **interleaved and messy**:

```console
# Bad: Mixed output from parallel linters
docs/README.md:42 MD001/heading-increment...
ansible/inventory.yml:15: [error] duplicate key...
docs/deployment.md:23 MD022/blanks-around...
Cargo.toml:5: expected newline...
src/main.rs:15: unused import...
```

Users wouldn't be able to tell which errors belong to which linter!

**Solution**: Refactor linters to capture output and display sequentially:

```rust
// Proposed implementation
struct LinterResult {
    linter_name: String,
    success: bool,
    stdout: String,
    stderr: String,
    error: Option<anyhow::Error>,
}

async fn run_linter_capturing_output(
    linter_name: &str,
    linter_fn: impl Fn() -> Result<()>
) -> LinterResult {
    // Capture output instead of printing immediately
    let result = linter_fn();

    LinterResult {
        linter_name: linter_name.to_string(),
        success: result.is_ok(),
        // ... capture output
    }
}

pub async fn run_all_linters_parallel() -> Result<()> {
    // Run Group 1 in parallel
    let group1_handles = vec![
        tokio::spawn(async { run_linter_capturing_output("markdown", run_markdown_linter) }),
        tokio::spawn(async { run_linter_capturing_output("yaml", run_yaml_linter) }),
        tokio::spawn(async { run_linter_capturing_output("toml", run_toml_linter) }),
        tokio::spawn(async { run_linter_capturing_output("shellcheck", run_shellcheck_linter) }),
        tokio::spawn(async { run_linter_capturing_output("cspell", run_cspell_linter) }),
    ];

    // Collect results
    let mut results = Vec::new();
    for handle in group1_handles {
        results.push(handle.await?);
    }

    // Display results sequentially for clean output
    for result in results {
        display_linter_result(result);
    }

    // Run Group 2 sequentially
    let clippy_result = run_linter_capturing_output("clippy", run_clippy_linter).await;
    display_linter_result(clippy_result);

    let rustfmt_result = run_linter_capturing_output("rustfmt", run_rustfmt_linter).await;
    display_linter_result(rustfmt_result);

    // Return overall success/failure
    // ...
}
```

### Challenge 2: Refactoring Effort

**Required Changes**:

1. **Refactor all 7 linters** to return results instead of printing immediately
2. **Create output buffering system** to capture stdout/stderr
3. **Implement result display** to show output sequentially after parallel execution
4. **Add async runtime** (`tokio` or similar) for parallel execution
5. **Update error handling** to work with captured results
6. **Comprehensive testing** for parallel scenarios

**Estimated Effort**: Medium to high - touches all linter modules and core execution logic

### Challenge 3: Compatibility with Auto-fix

**Consideration**: This feature interacts with the [linter auto-fix feature](../linter-auto-fix/specification.md).

**Auto-fix Safety**:

- ✅ Group 1 linters can auto-fix in parallel (different file types)
- ⚠️ Group 2 linters must auto-fix sequentially (same file types)
- ✅ No additional concerns beyond file conflicts already handled

**Integration**:

```rust
pub async fn run_all_linters_parallel(fix: bool) -> Result<()> {
    // Group 1: Parallel execution with optional auto-fix
    let group1_handles = vec![
        tokio::spawn(async move { run_markdown_linter_with_fix(fix) }),
        tokio::spawn(async move { run_yaml_linter_with_fix(fix) }),
        tokio::spawn(async move { run_toml_linter_with_fix(fix) }),
        // shellcheck and cspell don't support auto-fix
        tokio::spawn(async { run_shellcheck_linter() }),
        tokio::spawn(async { run_cspell_linter() }),
    ];

    // ... collect and display results

    // Group 2: Sequential execution (both modify .rs files)
    if fix {
        run_clippy_linter_with_fix(true)?;
    } else {
        run_clippy_linter()?;
    }
    run_rustfmt_linter()?;
}
```

## ⚖️ Pros and Cons

### ✅ Pros

1. **Performance improvement**: ~30% faster (13s → 9s)
2. **Better user experience**: Faster pre-commit workflow
3. **Scalable**: If we add more linters, parallel execution becomes more valuable
4. **No breaking changes**: CLI interface remains the same
5. **Safe**: No file conflicts due to careful grouping

### ❌ Cons

1. **Implementation complexity**: Requires refactoring all linter modules
2. **Output handling**: Need to buffer and display results sequentially
3. **Additional dependency**: Requires async runtime (tokio)
4. **Harder debugging**: Parallel execution can complicate troubleshooting
5. **Testing overhead**: Need to test parallel scenarios
6. **Modest gains**: Only 4 seconds saved (~30% improvement)

## 🔍 Cost-Benefit Analysis

### Performance Gain

- **Time saved**: 4 seconds per linter run
- **Percentage**: ~30% faster
- **User impact**: Moderate - noticeable but not game-changing
- **Frequency**: Every pre-commit (could be multiple times per hour for active development)

### Implementation Cost

- **Refactoring**: 7 linter modules need changes
- **New infrastructure**: Output buffering, result collection, async runtime
- **Testing**: Additional test scenarios for parallel execution
- **Maintenance**: More complex code to maintain
- **Risk**: Potential for new bugs during refactoring

### Verdict

#### Not a priority for initial implementation

**Rationale**:

- Current performance (13s) is acceptable for pre-commit workflow
- Implementation effort is significant
- Risk of introducing bugs during refactoring
- YAGNI principle: Implement only if performance becomes a real bottleneck
- Other features (like auto-fix) provide more value

## 🎯 When to Implement

Consider implementing parallel execution when:

1. **Linter count increases**: Adding more linters makes parallel execution more valuable
2. **Performance complaints**: Users report that linting is too slow
3. **CI/CD optimization**: Parallel execution becomes important for CI pipeline speed
4. **Auto-fix is stable**: After auto-fix feature is implemented and stable
5. **Time permits**: When there are no higher-priority features

## 🔄 Implementation Plan (Future)

If/when this feature is implemented:

### Phase 1: Output Refactoring

1. Create `LinterResult` struct to capture output
2. Refactor one linter as proof-of-concept (e.g., markdown)
3. Test output capture and display
4. Apply to all linters

### Phase 2: Parallel Execution

1. Add `tokio` dependency
2. Implement parallel execution for Group 1 linters
3. Keep Group 2 linters sequential
4. Test parallel scenarios

### Phase 3: Integration

1. Integrate with auto-fix feature (if implemented)
2. Update documentation
3. Comprehensive testing
4. Performance benchmarking

### Phase 4: Optimization

1. Fine-tune grouping strategy
2. Optimize result collection
3. Monitor performance in real usage

## ✅ Definition of Done (If Implemented)

- [ ] All linters refactored to capture output instead of immediate printing
- [ ] Output buffering system implemented
- [ ] Parallel execution working for Group 1 linters
- [ ] Group 2 linters run sequentially
- [ ] Clean, grouped output maintained
- [ ] Compatible with auto-fix feature
- [ ] All existing tests pass
- [ ] New tests for parallel scenarios
- [ ] Performance improvement verified (~30% faster)
- [ ] Documentation updated

## 📚 Related Documentation

- [Linter Auto-fix Feature](../linter-auto-fix/specification.md) - May interact with parallel execution
- [Development Principles](../../development-principles.md)
- [Linting Guide](../../contributing/linting.md)

## 📊 Priority

**Priority**: Low (Future Enhancement)

**Reason**: Current performance is acceptable. Focus on higher-value features first (like auto-fix).

**Decision**: Defer implementation until there's clear evidence it's needed.

## 🎯 Key Decisions (Based on Answered Questions)

The following decisions were made based on answers in [questions.md](./questions.md):

### 1. Performance Threshold

- **Current state**: ~13s is acceptable for pre-commit workflow
- **Trigger point**: Reconsider when execution time exceeds **25 seconds**
- **Justification**: More linters added in the future would make parallel execution more valuable

### 2. Output Handling Strategy

- **Chosen approach**: **Option B - Synchronized output mechanism** (mutex-protected stdout)
- **Rationale**: Keep current output format, collect output from each linter and print sequentially after all finish
- **Acceptable trade-off**: Mixing metadata is fine; problem is mixing error reporting from different linters

### 3. Grouping Strategy (Updated)

Based on file conflict analysis:

- **Group 1 (Parallel)**: markdown, yaml, toml, shellcheck, **rustfmt**
  - All operate on different file types
  - rustfmt only modifies `*.rs` files, can run in parallel
- **Group 2 (Sequential)**: clippy
  - Must run after Group 1 if auto-fix is enabled
  - Can run in parallel with Group 1 if no auto-fix
- **cspell**: Can be in its own group (read-only, checks all files)

### 4. Async Runtime

- **Choice**: **tokio** (most popular and full-featured)
- **Rationale**: Ecosystem support and flexibility
- **Trade-off**: More complex than rayon, but provides better long-term flexibility

### 5. Refactoring Approach

- **Strategy**: **Incremental** - refactor one linter at a time
- **Process**: Implement → Test → Commit → Next linter
- **Proof-of-concept**: Start with 1-2 linters to validate approach
- **Timeline**: One day for complete implementation

### 6. Auto-fix Compatibility

- **Requirement**: Support auto-fix from the start
- **Priority**: Auto-fix feature must be implemented **first** (higher value)
- **Integration**: Parallel execution should work seamlessly with `--fix` flag

### 7. Configuration

- **Default behavior**: Parallel execution by default once implemented
- **Debugging support**: Provide flag to disable parallelization for debugging (`--sequential`)
- **Grouping**: Not configurable - hardcoded grouping strategy is sufficient

### 8. Testing Strategy

- **Approach**: Manual and visual verification of output
- **Sequential option**: Add option to run linters sequentially for easier testing
- **Cross-platform**: Not necessary to test on different machines/OSes
- **Focus**: Verify output is clean and not interleaved

### 9. Error Handling

- **Failure strategy**: Continue with other linters if one fails
- **Panic handling**: Catch panics and report as errors without crashing
- **Fallback**: If output buffering fails, fall back to sequential execution

### 10. Implementation Prerequisites

- **Must complete first**: Auto-fix feature
- **Reconsider when**: Execution time exceeds 25 seconds or more linters are added
