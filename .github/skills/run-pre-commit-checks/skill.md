---
name: run-pre-commit-checks
description: Run the mandatory pre-commit verification script before committing changes. Executes all quality gates including cargo machete, linters, tests, documentation builds, and E2E tests. Use when preparing to commit code, ensuring all quality checks pass, or verifying changes meet project standards. Triggers on "pre-commit", "run pre-commit", "pre-commit checks", "before commit", "verify before commit", "quality checks", or "commit verification".
metadata:
  author: torrust
  version: "1.0"
---

# Run Pre-commit Checks

This skill guides you through running the mandatory pre-commit verification script for the Torrust Tracker Deployer project.

## ⚠️ Critical Rule

**Before committing ANY changes**, you **MUST** run the pre-commit verification script:

```bash
./scripts/pre-commit.sh
```

**All checks must pass** before staging files with `git add` or creating commits with `git commit`. This applies to **any** method of committing:

- Terminal: `git add`, `git commit`, `git commit -am`, `cd ../ && git add ...`
- VS Code: Git panel, Source Control view, commit shortcuts
- IDEs: IntelliJ, CLion, RustRover git integration
- Git clients: GitHub Desktop, GitKraken, etc.
- CI/CD: Any automated commits or merges

## When to Use Pre-commit Checks

Run the pre-commit script:

- ✅ **Before every commit** (mandatory) - Regardless of how you commit
- ✅ **After making any code changes** - Verify quality standards
- ✅ **Before pushing to remote** - Ensure CI will pass
- ✅ **After resolving merge conflicts** - Verify integrity
- ✅ **When preparing a pull request** - Final verification

## What the Script Checks

The pre-commit script runs comprehensive quality gates:

### Standard Mode (Full Checks)

When running without environment variables, executes all checks:

1. **Unused Dependencies**: `cargo machete`
   - Detects and reports unused dependencies in Cargo.toml
   - Zero unused dependencies required

2. **All Linters**: `cargo run --bin linter all`
   - Markdown linting (markdownlint)
   - YAML linting (yamllint)
   - TOML formatting and linting (taplo)
   - Spell checking (cspell)
   - Rust code analysis (clippy on stable & nightly)
   - Rust code formatting (rustfmt)
   - Shell script analysis (shellcheck)

3. **Unit Tests**: `cargo test`
   - All unit and integration tests must pass

4. **Documentation Build**: `cargo doc --no-deps --bins --examples --workspace --all-features`
   - Verifies documentation compiles without errors

5. **E2E Infrastructure Lifecycle Tests**: `cargo run --bin e2e-infrastructure-lifecycle-tests`
   - Tests infrastructure provisioning and destruction (~44 seconds)

6. **E2E Deployment Workflow Tests**: `cargo run --bin e2e-deployment-workflow-tests`
   - Tests software installation, configuration, release, and run (~48 seconds)

**Total execution time**: 5-10 minutes (depends on machine performance)

### ⚠️ Important for AI Agents

If you are an AI agent executing this script programmatically:

- **Set command timeout to at least 10 minutes** (600 seconds)
- The script takes 5+ minutes even on powerful development machines
- Many AI agent environments have default timeouts of 180 seconds (3 minutes), which is **too short**
- Prematurely terminating the script will result in incomplete validation
- Consider using **Fast Mode** for quicker feedback within timeout constraints

### Fast Mode (Skip Slow Tests)

For faster feedback during development or when working within time constraints (e.g., GitHub Copilot's 5-minute timeout):

```bash
TORRUST_TD_SKIP_SLOW_TESTS=true ./scripts/pre-commit.sh
```

**This mode skips**:

- E2E infrastructure lifecycle tests (~44 seconds)
- E2E deployment workflow tests (~48 seconds)

**Fast mode still runs**:

- Unused dependency check
- All linters (comprehensive check)
- Unit tests
- Documentation build

**Total execution time**: Approximately 2 minutes 30 seconds

**Important**: Skipped tests will run automatically in CI after PR creation. Use fast mode only when:

- Working interactively with AI agents (timeout constraints)
- Rapid iteration during development
- You've verified E2E tests pass locally in a previous run

## Usage Workflows

### Workflow 1: Standard Pre-commit (Recommended)

```bash
# 1. Make your changes
# ... edit files ...

# 2. Run pre-commit checks (MANDATORY)
./scripts/pre-commit.sh

# 3. If all checks pass, stage and commit
git add <files>
git commit -m "feat: [#42] add new feature"

# 4. Push to remote
git push
```

**For AI agents**: When running programmatically, ensure command timeout is set to **at least 600 seconds (10 minutes)**.

### Workflow 2: Fast Pre-commit (Time-Constrained)

```bash
# 1. Make your changes
# ... edit files ...

# 2. Run pre-commit checks in fast mode
TORRUST_TD_SKIP_SLOW_TESTS=true ./scripts/pre-commit.sh

# 3. If all checks pass, stage and commit
git add <files>
git commit -m "feat: [#42] add new feature"

# 4. Push to remote (E2E tests will run in CI)
git push
```

### Workflow 3: Fixing Failures

```bash
# 1. Run pre-commit checks
./scripts/pre-commit.sh

# 2. If a check fails, read the error output carefully

# 3. Fix the specific issue (examples follow)

# 4. Re-run the pre-commit script
./scripts/pre-commit.sh

# 5. Repeat until all checks pass
```

## Common Failure Scenarios

### Unused Dependencies Detected

```text
FAILED: Unused dependencies detected by cargo machete
```

**Fix**:

1. Read the cargo machete output to see which dependencies are unused
2. Remove unused dependencies from `Cargo.toml`
3. Run `cargo build` to verify the project still compiles
4. Re-run `./scripts/pre-commit.sh`

### Linter Failures

```text
FAILED: Linters found issues
```

**Fix**:

1. Identify which linter failed from the output
2. Run the specific linter to see detailed errors:

   ```bash
   cargo run --bin linter markdown   # Or yaml, toml, cspell, clippy, rustfmt, shellcheck
   ```

3. Fix the reported issues
4. Re-run `./scripts/pre-commit.sh`

**Common linter fixes**:

- **rustfmt**: Run `cargo run --bin linter rustfmt` to auto-format
- **clippy**: Address code quality warnings
- **markdown**: Fix markdown formatting (line length, lists, etc.)
- **cspell**: Add unknown words to `project-words.txt` or fix typos

### Test Failures

```text
FAILED: Some tests failed
```

**Fix**:

1. Run tests to see which ones failed:

   ```bash
   cargo test
   ```

2. Read the test failure output carefully
3. Fix the failing tests or the code causing the failure
4. Run `cargo test` again to verify
5. Re-run `./scripts/pre-commit.sh`

### Documentation Build Failures

```text
FAILED: Documentation build failed
```

**Fix**:

1. Check the error output for broken documentation links or invalid syntax
2. Fix documentation comments in your Rust code
3. Run `cargo doc` to verify
4. Re-run `./scripts/pre-commit.sh`

### E2E Test Failures

```text
FAILED: E2E tests failed
```

**Fix**:

1. Read the E2E test output carefully
2. Check if it's a real failure or an expected behavior (see `docs/contributing/known-issues.md`)
3. If it's a real failure:
   - Fix the code causing the failure
   - Run the specific E2E test: `cargo run --bin e2e-infrastructure-lifecycle-tests` or `cargo run --bin e2e-deployment-workflow-tests`
4. Re-run `./scripts/pre-commit.sh`

**Note**: Some E2E test output appears red but is expected (e.g., SSH host key warnings). See [`docs/contributing/known-issues.md`](../../../docs/contributing/known-issues.md).

## Troubleshooting

### Script Takes Too Long

If the full pre-commit script is taking too long:

1. **Use fast mode**: `TORRUST_TD_SKIP_SLOW_TESTS=true ./scripts/pre-commit.sh`
2. **Run checks individually** during development:

   ```bash
   cargo run --bin linter clippy    # Quick Rust checks
   cargo run --bin linter rustfmt   # Quick formatting
   cargo test                       # Quick unit tests
   ```

3. **Run full pre-commit** only before final commit

### Environment Issues

If you encounter environment-related issues:

1. **Check dependencies**: `cargo run --bin dependency-installer check`
2. **Install missing tools**: `cargo run --bin dependency-installer install`
3. **Verify tool versions**: `cargo run --bin dependency-installer list`

### Permission Issues

If you get permission errors:

```bash
chmod +x ./scripts/pre-commit.sh
```

## Best Practices

1. **Run early, run often**: Don't wait until you have many changes to run pre-commit
2. **Fix issues incrementally**: Address linter warnings as you code
3. **Use fast mode during development**: Run full checks before final commit
4. **Read error messages carefully**: They usually contain the solution
5. **Don't skip checks**: They catch issues early and save CI time
6. **Keep commits small**: Easier to debug if pre-commit fails

## Related Documentation

- [Commit Process Guide](../../../docs/contributing/commit-process.md) - Full commit workflow
- [Contributing Guide](../../../docs/contributing/README.md) - General contribution guidelines
- [Linting Guide](../../../docs/contributing/linting.md) - Detailed linting information
- [Known Issues](../../../docs/contributing/known-issues.md) - Expected behaviors and workarounds
- [Testing Documentation](../../../docs/contributing/testing/) - Unit and E2E testing guides

## Script Location

The pre-commit script is located at:

```text
scripts/pre-commit.sh
```

You can also view the script source to understand what checks are being run:

```bash
cat scripts/pre-commit.sh
```

## Integration with Git Hooks

While the project doesn't use automatic git hooks by default, you can optionally install a git pre-commit hook to run the script automatically:

```bash
./scripts/install-git-hooks.sh
```

This will create a `.git/hooks/pre-commit` file that runs the verification script before every commit.

## Summary

**Remember**: The pre-commit script is your quality gate. It ensures:

- Code quality standards are met
- Tests pass consistently
- Documentation is valid
- No unused dependencies exist
- Changes are ready for review

Running it before every commit saves time, prevents CI failures, and maintains project quality.
