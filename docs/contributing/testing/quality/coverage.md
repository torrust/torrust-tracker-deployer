# Code Coverage Guide

This guide explains our code coverage practices, expectations, and how to work with coverage reports in the Torrust Tracker Deployer project.

## 📋 Overview

Code coverage is a metric that measures which lines of code are executed during tests. It helps us:

- **Identify Untested Code**: Find areas that lack test coverage
- **Maintain Quality**: Ensure new features include adequate tests
- **Track Progress**: Monitor testing improvements over time
- **Support Refactoring**: Give confidence when changing code

**Important**: Coverage is a **tool**, not a **goal**. High coverage doesn't guarantee bug-free code, but it does indicate that code has been exercised by tests. We use coverage as one of many indicators of code quality.

## 📊 Coverage Targets

### Project-Wide Goals

- **Overall Coverage Target**: ≥ 70% (lines)
- **Critical Business Logic**: ≥ 90% (domain layer, commands, steps)
- **Shared Utilities**: ≥ 95% (clock, username, command executor)

These are **targets**, not strict requirements. PRs may be merged below these thresholds with proper justification.

### What We DON'T Require Coverage For

The following modules are **intentionally excluded** from strict coverage requirements:

#### 1. Binary Entry Points

- **Location**: `src/bin/`, `src/main.rs`
- **Reason**: These are executables tested through actual execution
- **Coverage**: Not measured
- **Testing**: Validated through E2E tests and manual execution

#### 2. E2E Test Infrastructure

- **Location**: `src/testing/e2e/tasks/`
- **Reason**: Testing utilities that support E2E tests
- **Coverage**: Not required
- **Testing**: Validated through E2E test execution

#### 3. Infrastructure Adapters

When mocking adds no value or requires real infrastructure:

- **`src/adapters/lxd/`** - Requires real LXD
- **`src/adapters/tofu/`** - Requires real OpenTofu
- **`src/infrastructure/remote_actions/`** - Requires real remote infrastructure
- **Coverage**: Tested via E2E tests
- **Reason**: These interact with external systems that cannot be easily mocked

#### 4. Linting Package

- **Location**: `packages/linting/`
- **Reason**: Primarily executed as binary, wraps external tools
- **Coverage**: 30-40% is acceptable
- **Testing**: Validated through actual execution

#### 5. Error Types

- **Reason**: Some error variants only occur in real infrastructure failures
- **Coverage**: Partial coverage is acceptable
- **Testing**: Critical error paths should be tested; rare edge cases may remain uncovered

## 🧪 Running Coverage Locally

### Prerequisites

Install `cargo-llvm-cov`:

```bash
cargo install cargo-llvm-cov
```

### Quick Coverage Check

Validate that coverage meets the threshold:

```bash
cargo cov-check
```

This command:

- Runs tests with coverage instrumentation
- Calculates line coverage percentage
- **Fails** if coverage is below the threshold
- Shows a summary of coverage by file

**Example Output (Passing)**:

```text
Finished test [unoptimized + debuginfo] target(s) in 34.56s
     Running unittests src/lib.rs (target/llvm-cov-target/debug/deps/torrust_tracker_deployer_lib-abc123)
...
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
src/application/commands/...     85.67%       ...             87.23%      ...
...
TOTAL                           ...             ...          87.23%      ...              ...       ...         ...              ...        87.23%        ...              ...       ...
```

**Example Output (Failing)**:

```text
...
TOTAL                           ...             ...          82.45%      ...              ...       ...         ...              ...        82.45%        ...              ...       ...
error: coverage is below 85%
```

### Detailed Coverage Reports

#### Generate LCOV Format

Useful for integration with coverage tools and IDEs:

```bash
cargo cov-lcov
```

Output: `.coverage/lcov.info`

Use this format with:

- IDE plugins (VS Code, IntelliJ)
- Coverage visualization tools
- CI/CD integrations

#### Generate Codecov JSON Format

For Codecov service integration:

```bash
cargo cov-codecov
```

Output: `.coverage/codecov.json`

#### Generate HTML Report

For human-readable, detailed coverage analysis:

```bash
cargo cov-html
```

Output: `target/llvm-cov/html/index.html`

Open in browser:

```bash
open target/llvm-cov/html/index.html  # macOS
xdg-open target/llvm-cov/html/index.html  # Linux
```

The HTML report provides:

- **Line-by-line coverage**: See exactly which lines are covered
- **Function coverage**: Identify untested functions
- **Branch coverage**: Understand conditional logic coverage
- **Color coding**: Green (covered), red (not covered), yellow (partially covered)

#### Basic Coverage Report

For a quick terminal-based summary:

```bash
cargo cov
```

This shows coverage statistics in the terminal without generating files.

## 🔄 Coverage Aliases Reference

All coverage commands use cargo aliases defined in `.cargo/config.toml`:

| Alias               | Full Command                                                      | Purpose                           |
| ------------------- | ----------------------------------------------------------------- | --------------------------------- |
| `cargo cov`         | `cargo llvm-cov`                                                  | Basic coverage report in terminal |
| `cargo cov-check`   | `cargo llvm-cov --all-features --workspace --fail-under-lines 70` | Validate coverage threshold            |
| `cargo cov-lcov`    | `cargo llvm-cov --lcov --output-path=./.coverage/lcov.info`       | Generate LCOV format              |
| `cargo cov-codecov` | `cargo llvm-cov --codecov --output-path=./.coverage/codecov.json` | Generate Codecov JSON             |
| `cargo cov-html`    | `cargo llvm-cov --html`                                           | Generate HTML report              |

## 🚨 Coverage Enforcement Strategy

Code coverage is **not checked in pre-commit** to keep local development fast and focused on core quality checks.

### Why No Pre-commit Coverage?

Coverage is excluded from pre-commit because:

- **Speed**: Coverage analysis is slow (1-2 minutes) and would slow down local commits
- **Reliability**: Coverage tools can fail due to missing binaries or tool issues
- **Developer Experience**: Fast feedback loop for core quality checks (linting, tests)
- **CI Enforcement**: Coverage threshold is enforced where it matters most - in CI

### Local Coverage (Optional)

Developers can still check coverage locally when needed:

```bash
# Check current coverage (fast)
cargo cov-check

# Generate detailed HTML report
cargo cov-html
```

**Use cases for local coverage**:

- Before submitting a PR with new features
- Investigating coverage gaps in specific modules
- Understanding which code paths need more testing

## 🔄 CI/CD Coverage Workflow

Code coverage is automatically generated in GitHub Actions for every push and pull request.

### Workflow Configuration

**File**: `.github/workflows/coverage.yml`

The workflow generates coverage in multiple formats:

1. **Text Summary** - Terminal output for quick review
2. **HTML Report** - Detailed, browsable coverage report
3. **Coverage Artifacts** - Uploaded for download and review

### What the Workflow Does

```yaml
- Generate text coverage summary (cargo cov)
- Generate HTML coverage report (cargo cov-html)
- Upload HTML report as GitHub Actions artifact
```

### Accessing Coverage Reports

#### From GitHub Actions UI

1. Navigate to your PR or commit
2. Click on **"Checks"** tab
3. Select **"Coverage Report"** workflow
4. Scroll to **"Artifacts"** section
5. Download **"coverage-html-report"**
6. Extract and open `index.html` in browser

#### Coverage Report Contents

The HTML report includes:

- Overall coverage percentages
- Per-file coverage breakdown
- Line-by-line coverage visualization
- Function and branch coverage details

### Non-blocking Nature

The coverage workflow:

- **Does NOT block merges** if coverage is low
- Provides **visibility** into coverage changes
- Helps **reviewers assess** test quality
- Generates **artifacts** for detailed analysis

**Why?** Same reasons as pre-commit: security patches, refactoring, and WIP commits should not be blocked by coverage metrics.

## 📝 Coverage Expectations for PRs

### For New Features

When adding new features, aim for:

- **New domain logic**: ≥ 90% coverage
- **New commands/steps**: ≥ 70% coverage
- **New utilities**: ≥ 95% coverage
- **Infrastructure adapters**: E2E tests + reasonable unit tests

**Note**: These are **targets**, not blockers. PRs may be merged below these thresholds with proper justification.

### For Bug Fixes

When fixing bugs:

1. **Add a test** that reproduces the bug
2. **Verify** the test fails before the fix
3. **Ensure** the test passes after the fix
4. **Maintain or improve** existing coverage

This ensures the bug won't regress in the future.

### For Refactoring

When refactoring code:

1. **Maintain or improve** existing coverage
2. **Prefer** adding tests over decreasing project coverage
3. **Avoid** decreasing overall project coverage below 70%
4. **Document** any intentional coverage reductions
5. **Update tests** to reflect new structure

### For Documentation Changes

Documentation-only changes:

- **No coverage requirements** - tests are not needed
- **Coverage is only checked in CI** - no local coverage overhead
- **Focus** on markdown linting and link validation

### When Coverage Drops

If your PR reduces coverage:

1. **Explain why** in the PR description
2. **Justify** the change (e.g., "Removed dead code", "Refactored untestable adapter")
3. **Plan** when/how coverage will be restored (if applicable)
4. **Reviewers** will evaluate on a case-by-case basis

**Acceptable reasons** for coverage drops:

- Removing untested legacy code
- Refactoring to move code to E2E-only adapters
- Adding infrastructure code that requires real systems
- Moving code to excluded modules (binaries, linting package)

## 📊 Interpreting Coverage Results

### Understanding Coverage Percentages

Coverage types:

- **Line Coverage**: Percentage of lines executed
- **Function Coverage**: Percentage of functions called
- **Branch Coverage**: Percentage of conditional branches taken

We primarily track **line coverage** with the 70% target.

### Reading HTML Reports

**Color Coding**:

- **Green**: Line was executed by tests ✅
- **Red**: Line was never executed ❌
- **Yellow**: Partial coverage (e.g., one branch of `if` statement) ⚠️

**Focus Areas**:

1. **Domain entities/value objects**: Should be near 100%
2. **Commands/Steps**: Should be mostly green (70%+)
3. **Utilities**: Should be almost all green (95%+)
4. **Adapters**: May have more red (E2E tested)

### Analyzing Low Coverage

If coverage is low:

1. **Identify** which modules have low coverage
2. **Determine** if those modules are excluded (see "What We DON'T Require Coverage For")
3. **For non-excluded modules**, assess:
   - Are there missing unit tests?
   - Are there untested error paths?
   - Are there unused functions that can be removed?
4. **Prioritize** coverage improvements for:
   - Business-critical logic
   - Complex algorithms
   - Error handling paths

### Common Coverage Gaps

**Error Handling**:

- Error paths are often undertested
- Consider using `Result` tests with both `Ok` and `Err` cases
- Test error propagation and recovery

**Edge Cases**:

- Boundary conditions
- Empty collections
- Null/None values
- Maximum/minimum values

**Conditional Logic**:

- Both branches of `if/else`
- All cases in `match` statements
- Loop conditions (empty, single item, multiple items)

## ✅ PR Review Guidelines

### Coverage Checklist for PR Reviewers

When reviewing PRs:

- [ ] **Check coverage change**: Did overall coverage increase, decrease, or stay the same?
- [ ] **Assess new code coverage**: Are new features adequately tested?
- [ ] **Verify test quality**: Do tests actually validate behavior, or just exercise code?
- [ ] **Review excluded modules**: Is any code moved to excluded areas justified?
- [ ] **Evaluate coverage drops**: If coverage decreased, is the reason acceptable?

### When to Request More Tests

Request additional tests when:

- ✅ New domain logic has <90% coverage
- ✅ New commands/steps have <70% coverage
- ✅ Critical business logic is untested
- ✅ Error paths are completely untested
- ✅ Tests exist but don't validate actual behavior (dummy tests)

### When Coverage Gaps Are Acceptable

Accept lower coverage when:

- ✅ Code is in an excluded module (binaries, E2E infrastructure, adapters)
- ✅ Error conditions require real infrastructure failures
- ✅ Code is being removed/deprecated
- ✅ Refactoring temporarily reduces coverage with a plan to restore it
- ✅ Security patch needs immediate merge

### Reviewing Coverage Reports

1. **Download** the HTML coverage artifact from GitHub Actions
2. **Open** `index.html` in a browser
3. **Navigate** to changed files
4. **Verify** that:
   - New code is covered
   - Critical paths are tested
   - Error handling is reasonable

## 💡 Best Practices

### Do's

- ✅ **Run coverage locally** before submitting PRs
- ✅ **Focus on meaningful tests** that validate behavior
- ✅ **Test error paths** not just happy paths
- ✅ **Use coverage to find gaps** in test suites
- ✅ **Document intentional exclusions** in code comments when appropriate
- ✅ **Prioritize domain logic coverage** over infrastructure code
- ✅ **Write tests that will catch bugs**, not just increase percentages

### Don'ts

- ❌ **Don't write tests just for coverage** without validating behavior
- ❌ **Don't obsess over 100% coverage** - it's not realistic or valuable
- ❌ **Don't delay security patches** for coverage
- ❌ **Don't block refactoring** due to temporary coverage drops
- ❌ **Don't test implementation details** - test behavior
- ❌ **Don't ignore coverage warnings** - investigate before dismissing
- ❌ **Don't remove tests** to avoid fixing them - fix or document why

## 🔍 Troubleshooting

### Coverage Check Fails Locally

**Problem**: `cargo cov-check` reports coverage below 70%

**Solutions**:

1. Run `cargo cov-html` to see detailed report
2. Identify which modules have low coverage
3. Check if they're in excluded categories
4. Add tests for critical uncovered code
5. If justified, proceed with PR and explain in description

### Coverage Report Shows Unexpected Results

**Problem**: Coverage seems incorrect for tested code

**Possible Causes**:

1. **Test is not running**: Verify test is not `#[ignore]`d
2. **Feature flags**: Check if code requires `--all-features`
3. **Conditional compilation**: Code may be platform-specific
4. **Dead code**: Code may be unreachable

**Solutions**:

- Run `cargo test` and verify all tests pass
- Check `cargo cov-check` uses `--all-features`
- Review conditional compilation attributes

### CI Coverage Workflow Fails

**Problem**: Coverage workflow fails in GitHub Actions

**Common Causes**:

1. **Tests failing**: Coverage requires tests to pass
2. **Missing dependencies**: `cargo-llvm-cov` installation failed
3. **Timeout**: Tests taking too long

**Solutions**:

- Check test output in workflow logs
- Verify tests pass locally: `cargo test`
- Review workflow step outputs

## 🔗 Related Documentation

- **[Testing Conventions](./README.md)** - Main testing documentation and principles
- **[Unit Testing](./unit-testing.md)** - Unit test naming conventions and patterns
- **[Testing Commands](./testing-commands.md)** - Command testing strategies
- **[Pre-commit Integration](./pre-commit-integration.md)** - Pre-commit checks and enforcement
- **[Development Principles](../../development-principles.md)** - Quality standards and principles
- **[Error Handling](../error-handling.md)** - Error handling patterns and testing

## 📚 Additional Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Conventional Commits](https://www.conventionalcommits.org/) - Commit message format
- [LLVM Coverage Mapping Format](https://llvm.org/docs/CoverageMappingFormat.html) - Technical details
