# Linter Auto-fix Feature Specification

## 📋 Overview

Add automatic fixing capability to the linter binary, allowing developers to automatically fix common linting issues before committing code.

## 🎯 Goals

- **Reduce friction**: Developers can fix most linting issues with a single command
- **Maintain quality**: Still report errors that cannot be auto-fixed
- **Improve workflow**: Integrate auto-fix into the pre-commit checklist
- **Simple feedback**: Report only remaining errors (developers use git to see what changed)

## 🚀 Feature Description

Add a `--fix` flag to the linter binary that will:

1. Attempt to automatically fix issues for linters that support auto-fix
2. Run the linter check after auto-fix to verify and report remaining issues
3. Report only remaining errors that need manual attention
4. Exit with non-zero code if any errors remain after auto-fix

**Note**: Developers can use `git diff` or `git status` to see what files were changed by the auto-fix.

## 💡 Decision: Option 3 - Single Flag Approach

We chose **Option 3** (add `--fix` flag to existing linter) over alternatives because:

### ✅ Selected: Option 3 - Add `--fix` flag to existing linter

**Rationale:**

- **Single workflow**: One command does both fix and check
- **Integrated experience**: Fix attempt happens automatically before showing errors
- **Industry standard**: Most linters work this way (prettier, eslint, rustfmt, etc.)
- **Pre-commit friendly**: Fits naturally into the pre-commit checklist
- **Less cognitive load**: Developers only need to remember one command

**Usage:**

```bash
# Check only (current behavior)
cargo run --bin linter all

# Try to fix, then check
cargo run --bin linter all --fix

# Individual linters with fix
cargo run --bin linter markdown --fix
cargo run --bin linter yaml --fix
```

### ❌ Discarded: Option 1 - Show command to user

**Why discarded:**

- Creates friction in the workflow
- Requires extra manual step
- Users might forget to run the fix command
- Breaks the "pre-commit must pass" principle
- Not aligned with modern linting tool UX

### ❌ Discarded: Option 2 - Create separate fix tool

**Why discarded:**

- More commands to remember (`linter` and `linter-fix`)
- Additional binary to maintain
- Splits related functionality
- Users might not discover the fix tool
- Duplicates command-line argument parsing logic

## 🔧 Linter Auto-fix Support Matrix

| Linter         | Auto-fix Support | Fix Command                                       | Notes                                   |
| -------------- | ---------------- | ------------------------------------------------- | --------------------------------------- |
| **markdown**   | ✅ Yes           | `npx markdownlint-cli --fix <file>`               | Fixes most formatting issues            |
| **yaml**       | ✅ Yes           | `yamlfmt <file>`                                  | Uses yamlfmt for YAML formatting        |
| **clippy**     | ✅ Yes           | `cargo clippy --fix --allow-dirty --allow-staged` | Fixes many clippy warnings              |
| **rustfmt**    | ✅ Yes           | `cargo fmt`                                       | Already auto-formats (no change needed) |
| **shellcheck** | ❌ No            | N/A                                               | No native auto-fix support              |
| **taplo**      | ✅ Yes           | `taplo fmt <file>`                                | TOML formatting                         |
| **cspell**     | ❌ No            | N/A                                               | Spelling requires human judgment        |

### Linters Without Auto-fix

For linters without auto-fix support (`shellcheck`, `cspell`):

- **Strategy**: Skip auto-fix phase, run check only
- **Output**: Report errors normally with manual fix guidance
- **No special handling**: Treat as if `--fix` wasn't specified for that linter

## 📊 Expected Behavior

### With `--fix` Flag

```text
1. For each linter:
   a. If auto-fix supported:
      - Run auto-fix command on relevant files
      - Log what was fixed
   b. Run linter check (always, even after fix)
   c. Report remaining errors (if any)

2. Exit code:
   - 0 if all errors fixed or no errors found
   - Non-zero if errors remain after auto-fix attempt
```

### Without `--fix` Flag (Current Behavior)

```text
1. For each linter:
   a. Run linter check
   b. Report errors

2. Exit code:
   - 0 if no errors
   - Non-zero if errors found
```

## 🎨 Output Format

**Note**: All output uses the `tracing` crate following the current logging pattern. Flat logging with targets is used (no tracing spans needed).

**Verbosity**: Minimal - show only a summary of files fixed per linter, not individual file details.

### Successful Auto-fix (No Remaining Errors)

```console
$ cargo run --bin linter all --fix

2025-10-02T10:30:45.123456Z  INFO linter: Running All Linters (with auto-fix)
2025-10-02T10:30:45.234567Z  INFO markdown: Fixed 3 files
2025-10-02T10:30:45.345678Z  INFO markdown: Scanning markdown files...
2025-10-02T10:30:45.456789Z  INFO markdown: All markdown files passed linting!
2025-10-02T10:30:45.567890Z  INFO yaml: Fixed 2 files
2025-10-02T10:30:45.678901Z  INFO yaml: Scanning YAML files...
2025-10-02T10:30:45.789012Z  INFO yaml: All YAML files passed linting!
2025-10-02T10:30:45.890123Z  INFO clippy: Fixed 1 file
2025-10-02T10:30:46.012345Z  INFO clippy: Running Rust clippy linter...
2025-10-02T10:30:47.123456Z  INFO clippy: Clippy check passed!
2025-10-02T10:30:47.234567Z  INFO rustfmt: Running Rust formatter check...
2025-10-02T10:30:47.345678Z  INFO rustfmt: All Rust code is properly formatted!
2025-10-02T10:30:47.456789Z  INFO toml: Fixed 1 file
2025-10-02T10:30:47.567890Z  INFO toml: Scanning TOML files...
2025-10-02T10:30:47.678901Z  INFO toml: All TOML files passed linting!
2025-10-02T10:30:47.789012Z  INFO shellcheck: Scanning shell scripts...
2025-10-02T10:30:47.890123Z  INFO shellcheck: All shell scripts passed linting!
2025-10-02T10:30:47.901234Z  INFO cspell: Running spell checker...
2025-10-02T10:30:48.012345Z  INFO cspell: Spell checking passed!

# Developers can check what was changed with:
$ git status
$ git diff
```

### Partial Fix (Some Errors Remain After Auto-fix)

```console
$ cargo run --bin linter all --fix

2025-10-02T10:30:45.123456Z  INFO linter: Running All Linters (with auto-fix)
2025-10-02T10:30:45.234567Z  INFO markdown: Fixed 2 files
2025-10-02T10:30:45.345678Z  INFO markdown: Scanning markdown files...
docs/deployment.md:42 MD001/heading-increment: Heading levels should only increment by one level at a time [Expected: h2; Actual: h3]
2025-10-02T10:30:45.456789Z ERROR markdown: Markdown linting failed. Please fix the issues above.
2025-10-02T10:30:45.567890Z  INFO yaml: Fixed 1 file
2025-10-02T10:30:45.678901Z  INFO yaml: Scanning YAML files...
ansible/inventory.yml:15:1: [error] found duplicate key (key-duplicates)
2025-10-02T10:30:45.789012Z ERROR yaml: YAML linting failed. Please fix the issues above.
2025-10-02T10:30:45.890123Z  INFO clippy: No files needed fixing
2025-10-02T10:30:46.012345Z  INFO clippy: Running Rust clippy linter...
2025-10-02T10:30:47.123456Z  INFO clippy: Clippy check passed!
2025-10-02T10:30:47.234567Z  INFO rustfmt: Running Rust formatter check...
2025-10-02T10:30:47.345678Z  INFO rustfmt: All Rust code is properly formatted!
2025-10-02T10:30:47.456789Z  INFO toml: No files needed fixing
2025-10-02T10:30:47.567890Z  INFO toml: Scanning TOML files...
2025-10-02T10:30:47.678901Z  INFO toml: All TOML files passed linting!
2025-10-02T10:30:47.789012Z  INFO shellcheck: Scanning shell scripts...
2025-10-02T10:30:47.890123Z  INFO shellcheck: All shell scripts passed linting!
2025-10-02T10:30:47.901234Z  INFO cspell: Running spell checker...
2025-10-02T10:30:48.012345Z  INFO cspell: Spell checking passed!
2025-10-02T10:30:48.123456Z ERROR linter: Some linters failed

# Developers can check what was auto-fixed with:
$ git status
$ git diff
```

**Key Principles**:

- Use `tracing` crate for all output (consistent with current implementation)
- Flat logging with targets (no tracing spans needed for simplicity)
- Minimal verbosity: show only summary of files fixed per linter
- Only show errors that still need attention after auto-fix
- Developers use git to see what was changed
- Maintain current logging format and targets

## �️ Error Handling

### Missing Linter Tools

**Approach**: Auto-install missing tools (matches current linter behavior)

When a linter tool is not found, the binary will:

1. Detect the missing tool
2. Automatically install it (npm packages or cargo install)
3. Continue with linting operation
4. Log the installation process for visibility

**Examples**:

```console
2025-10-02T10:30:45.123456Z  WARN markdown: markdownlint-cli not found, installing...
2025-10-02T10:30:47.234567Z  INFO markdown: markdownlint-cli installed successfully
2025-10-02T10:30:47.345678Z  INFO markdown: Fixed 2 files
```

This matches the current behavior where linters automatically install dependencies when missing.

### Auto-fix Command Failures

If an auto-fix command fails:

1. Log the error with context
2. Skip the fix phase for that linter
3. Continue to the check phase (to show remaining issues)
4. Exit with non-zero code if errors persist

**Example**:

```console
2025-10-02T10:30:45.123456Z ERROR yaml: Auto-fix command failed: yamlfmt returned exit code 1
2025-10-02T10:30:45.234567Z  INFO yaml: Scanning YAML files...
ansible/inventory.yml:15:1: [error] found duplicate key (key-duplicates)
```

## �🔒 Safety Considerations

1. **Non-destructive**: Auto-fix only modifies files in safe, reversible ways
2. **Git integration**: Changes are unstaged, allowing review before commit
3. **Verification**: Always run check after fix to ensure no issues introduced
4. **Minimal output**: Only show errors that need attention, rely on git for change visibility
5. **Tool isolation**: Auto-installed tools are local to the project (npm/cargo)

## ⚡ Performance Considerations

### Sequential Execution

**Current Implementation**: Linters run **sequentially** (one after another)

**Execution Time**: ~13 seconds for all linters

**Performance**: Acceptable for pre-commit workflow

### Interaction with Parallel Execution

**Note**: There is a separate feature for [parallel linter execution](../linter-parallel-execution/specification.md) that could reduce execution time by ~30% (13s → 9s).

**Auto-fix Compatibility**: Auto-fix works safely with parallel execution because linters operate on different file types:

| Linter     | File Types        | Auto-fix Support | Notes                              |
| ---------- | ----------------- | ---------------- | ---------------------------------- |
| markdown   | `*.md`            | ✅ Yes           | No conflicts                       |
| yaml       | `*.yml`, `*.yaml` | ✅ Yes           | No conflicts                       |
| toml       | `*.toml`          | ✅ Yes           | No conflicts                       |
| clippy     | `*.rs`            | ✅ Yes           | Conflicts with rustfmt (see below) |
| rustfmt    | `*.rs`            | ✅ Yes           | Conflicts with clippy (see below)  |
| shellcheck | `*.sh`            | ❌ No            | Read-only checker                  |
| cspell     | All text files    | ❌ No            | Read-only checker                  |

**Key Insights**:

- ✅ Most linters can auto-fix independently (different file types)
- ⚠️ `clippy --fix` and `rustfmt` both modify `.rs` files - must run sequentially to avoid conflicts
- ✅ Auto-fix is safe - no risk of file corruption or data loss

**Implementation**: Auto-fix will run linters sequentially (current approach), which naturally avoids any potential file conflicts.

**For parallel execution details**: See the separate [Parallel Linter Execution Feature](../linter-parallel-execution/specification.md) for analysis of running linters in parallel. This is a future optimization that is compatible with auto-fix but not required for auto-fix functionality.

## 🚫 Out of Scope (Current Implementation)

### No Dry-run Option

**Decision**: Do not implement `--dry-run` flag in initial version

**Rationale:**

- Adds complexity without clear immediate benefit
- Git already provides safety (changes are unstaged)
- Can be added later if users request it
- YAGNI principle - implement when needed

### No Interactive Mode

**Decision**: Do not implement interactive fix confirmation

**Rationale:**

- Would slow down the workflow
- Auto-fixes are safe and reviewable via git
- Can be added later if needed

### No Selective Linter Fix

**Decision**: `--fix` applies to all specified linters, no per-linter control

**Rationale:**

- Simplifies UX and implementation
- Users can run individual linters if selective fix needed
- Example: `cargo run --bin linter markdown --fix` (only markdown)

## 📝 Integration Points

### Pre-commit Workflow

Update `docs/contributing/commit-process.md`:

```bash
# Before committing: Run linters with auto-fix
cargo run --bin linter all --fix
```

### CI/CD

CI should run **without** `--fix` flag:

```bash
# CI - fail if code is not already formatted
cargo run --bin linter all
```

This ensures developers format code locally before pushing.

## 🔨 Implementation Approach

### Incremental Development

**Strategy**: Implement auto-fix for all linters, but develop and test **one linter at a time**

**Process**:

1. Add `--fix` flag to CLI (applies to all linters)
2. Implement auto-fix for one linter
3. Test thoroughly (unit + integration + E2E + manual)
4. Commit and push
5. Move to next linter
6. Repeat until all linters support auto-fix

**Benefits**:

- Easier to review and test changes
- Smaller, focused commits
- Can deploy partially completed feature (some linters work with `--fix`)
- Reduces risk of bugs
- Easier to debug issues

**Order** (suggested, based on complexity):

1. **rustfmt** - Already works, just add flag support
2. **toml** (taplo) - Simple formatting tool
3. **markdown** - Straightforward npm tool
4. **yaml** (yamlfmt) - New tool, needs verification
5. **clippy** - Most complex (requires `--allow-dirty --allow-staged` flags)

## ✅ Definition of Done

- [ ] `--fix` flag added to linter binary CLI
- [ ] Auto-fix implemented for supported linters (markdown, yaml, clippy, taplo)
- [ ] Rustfmt continues to work as before
- [ ] Linters without auto-fix (shellcheck, cspell) skip fix phase
- [ ] Clear output showing fixed vs manual issues
- [ ] Exit codes correct (0 if all pass, non-zero if errors remain)
- [ ] Works for both `all` and individual linter commands
- [ ] Documentation updated in `docs/contributing/commit-process.md`
- [ ] All existing tests pass
- [ ] Manual testing completed for each linter

## 🧪 Testing Strategy

### Unit Tests

- Test CLI argument parsing for `--fix` flag
- Test auto-fix logic for each supported linter
- Test skip behavior for unsupported linters

### Integration Tests

- Test end-to-end workflow with `--fix` flag
- Test that fixes are applied correctly
- Test that checks run after fix
- Test exit codes in various scenarios

### Manual Testing

- Test with actual files containing fixable issues
- Verify output messages are clear and helpful
- Confirm git shows unstaged changes after fix
- Test both `all` and individual linter modes

## 📚 Related Documentation

- [Error Handling Guide](../../contributing/error-handling.md)
- [Development Principles](../../development-principles.md)
- [Linting Guide](../../contributing/linting.md)

## 🔄 Future Enhancements (Not in Scope)

Potential future additions (implement only if needed):

1. **Parallel execution**: Run linters in parallel for better performance (~30% faster, 13s → 9s)

   - This is a separate feature with its own specification
   - See [Parallel Linter Execution Feature](../linter-parallel-execution/specification.md) for details
   - Compatible with auto-fix but not required for auto-fix functionality
   - Priority: Low (current performance is acceptable)

2. **Dry-run mode**: Preview what would be fixed without applying changes

3. **Interactive mode**: Confirm each fix before applying

4. **Fix statistics**: Detailed report of what was fixed

5. **Per-linter fix control**: `--fix-only=markdown,yaml`

6. **Configuration file**: Allow customizing auto-fix behavior
