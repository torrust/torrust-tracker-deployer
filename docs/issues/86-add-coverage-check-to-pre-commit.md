# Add Coverage Check to Pre-commit Script

**Issue**: #86
**Parent Epic**: #85 - Coverage & Reporting EPIC
**Related**: [85-epic-coverage-and-reporting.md](./85-epic-coverage-and-reporting.md)

## Overview

Add code coverage validation to the pre-commit script (`scripts/pre-commit.sh`) as the last check in the steps array. The check uses the `cargo cov-check` alias which validates that coverage meets the 85% threshold and will fail the pre-commit if coverage drops below this level.

## Goals

- [ ] Make coverage validation part of the pre-commit process
- [ ] Provide developers immediate feedback about coverage impact
- [ ] Run coverage check last to ensure all other checks pass first (saves time)
- [ ] Use existing step infrastructure for consistency

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (Tooling/Scripts)
**Module Path**: `scripts/pre-commit.sh`
**Pattern**: Shell script validation tool

### Module Structure Requirements

- [ ] Follow existing script structure and patterns in `scripts/pre-commit.sh`
- [ ] Use consistent formatting and messaging style
- [ ] Maintain the step-based execution model

### Architectural Constraints

- [ ] Must run as the last step in the `STEPS` array
- [ ] Should use existing `cargo cov-check` alias from `.cargo/config.toml`
- [ ] Must leverage existing step execution infrastructure (timing, error handling, formatting)
- [ ] Must provide clear, actionable messaging

### Anti-Patterns to Avoid

- ❌ Adding special-case logic instead of using the existing step array
- ❌ Running coverage before other checks (wastes time if linting fails)
- ❌ Duplicating timing/formatting logic that already exists

## Specifications

### Current Pre-commit Structure

The `scripts/pre-commit.sh` script uses a step-based execution model with an array of checks:

```bash
declare -a STEPS=(
    "Checking for unused dependencies (cargo machete)|No unused dependencies found|||cargo machete"
    "Running linters|All linters passed|||cargo run --bin linter all"
    "Running tests|All tests passed|||cargo test"
    "Testing cargo documentation|Documentation builds successfully|||cargo doc --no-deps --bins --examples --workspace --all-features"
    "Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
)
```

Each step follows the format:
`"description|success_message|optional_pre_message|optional_env_var|command"`

All steps fail fast on errors due to `set -euo pipefail`.

### Proposed Change

Add coverage check as the **last step** in the `STEPS` array:

```bash
declare -a STEPS=(
    "Checking for unused dependencies (cargo machete)|No unused dependencies found|||cargo machete"
    "Running linters|All linters passed|||cargo run --bin linter all"
    "Running tests|All tests passed|||cargo test"
    "Testing cargo documentation|Documentation builds successfully|||cargo doc --no-deps --bins --examples --workspace --all-features"
    "Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
    "Running code coverage check|Coverage meets 85% threshold|(Informational only - does not block commits)||cargo cov-check"
)
```

### Why This Approach?

**Advantages:**

- ✅ **Consistent**: Follows existing script patterns
- ✅ **Simple**: Just one line added to the array
- ✅ **Maintainable**: No special-case logic needed
- ✅ **Clear**: Uses same messaging format as other steps
- ✅ **Automatic timing**: Built-in timing display like other steps

**Behavior:**

- Runs last after all other checks pass
- Still fails fast if coverage is below 85% (exits with non-zero code)
- Shows clear error message from `cargo cov-check`
- Uses existing error handling and timing infrastructure

### Alternative: Non-blocking Implementation

**Current decision**: Treat coverage like other mandatory checks (blocking).

**If non-blocking behavior is needed later**, the script could be modified to:

1. Add a separate section after the STEPS loop
2. Use conditional execution (`if cargo cov-check; then ... else ... fi`)
3. Exit with code 0 regardless of coverage result

However, the simpler approach is to start with blocking behavior and only add complexity if feedback shows it's needed.

## Implementation Plan

### Phase 1: Add Coverage Step (5 minutes)

- [ ] Open `scripts/pre-commit.sh`
- [ ] Locate the `declare -a STEPS=()` array
- [ ] Add new line at the end of the array (before the closing parenthesis)
- [ ] The new line should be:

```bash
"Running code coverage check|Coverage meets 85% threshold|(Informational only - does not block commits)||cargo cov-check"
```

- [ ] Save the file

### Phase 2: Testing (10 minutes)

- [ ] Run `./scripts/pre-commit.sh` on current codebase (coverage should pass at 85.75%)
- [ ] Verify coverage check runs last (after E2E tests)
- [ ] Verify timing is displayed correctly (uses existing timing infrastructure)
- [ ] Verify success message appears when coverage passes
- [ ] Verify script shows clear output from `cargo cov-check`

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Coverage check is added to `scripts/pre-commit.sh`
      **Task-Specific Criteria**:

- [ ] Coverage check added as last step in `STEPS` array in `scripts/pre-commit.sh`
- [ ] Uses `cargo cov-check` alias (already exists in `.cargo/config.toml`)
- [ ] Success message reads: "Coverage meets 85% threshold"
- [ ] Pre-message reads: "(Informational only - does not block commits)"
- [ ] Script exits with non-zero code if coverage is below 85%
- [ ] Timing information is automatically displayed (inherited from step infrastructure)
- [ ] All existing checks still work correctly

## Related Documentation

- [Pre-commit Process](../contributing/commit-process.md) - Pre-commit workflow
- [Development Principles](../development-principles.md) - Quality standards
- [Testing Conventions](../contributing/testing.md) - Testing guidelines

## Notes

- The coverage check is intentionally non-blocking to support urgent fixes and patches
- Running coverage last ensures developers don't waste time waiting for coverage if linting fails
- The `tail -5` filter keeps output concise while showing the essential coverage summary
- Exit code 0 is important for CI/CD pipelines that may use this script
