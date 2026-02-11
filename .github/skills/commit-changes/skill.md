---
name: commit-changes
description: Guide for committing changes in the torrust-tracker-deployer project. Covers conventional commit format, pre-commit verification checks, issue number conventions, and commit quality guidelines. Use when committing code, running pre-commit checks, or following project commit standards. Triggers on "commit", "commit changes", "how to commit", "pre-commit", "commit message", "commit format", or "conventional commits".
metadata:
  author: torrust
  version: "1.0"
---

# Committing Changes

This skill guides you through the complete commit process for the Torrust Tracker Deployer project, including conventional commit format, pre-commit checks, and quality standards.

## Quick Reference

```bash
# 1. Run pre-commit checks (MANDATORY)
./scripts/pre-commit.sh

# 2. Stage changes
git add <files>

# 3. Commit with conventional format
git commit -m "{type}: [#{issue}] {description}"
```

## Conventional Commit Format

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Message Structure

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Issue Number Convention

When working on a branch with an issue number, include it in your commit messages:

- **With issue branch**: `{type}: [#{issue}] {description}`
  - Example: `feat: [#42] add MySQL database support`
- **Without issue branch**: `{type}: {description}`
  - Example: `docs: update deployment guide`

### Commit Types

| Type       | Description                           | Example                                              |
| ---------- | ------------------------------------- | ---------------------------------------------------- |
| `feat`     | New feature or enhancement            | `feat: [#42] add LXD container provisioning`         |
| `fix`      | Bug fix                               | `fix: [#15] resolve ansible inventory parsing error` |
| `docs`     | Documentation changes                 | `docs: [#8] update installation guide`               |
| `style`    | Code style changes (formatting, etc.) | `style: [#31] apply rustfmt to all source files`     |
| `refactor` | Code refactoring                      | `refactor: [#45] simplify linting script structure`  |
| `test`     | Adding or updating tests              | `test: [#67] add e2e tests for provisioning`         |
| `chore`    | Maintenance tasks                     | `chore: [#89] update dependencies`                   |
| `ci`       | CI/CD related changes                 | `ci: [#23] add workflow for testing provisioning`    |
| `perf`     | Performance improvements              | `perf: [#52] optimize container startup time`        |

## Pre-commit Verification (MANDATORY)

**Before committing any changes**, you **MUST** run:

```bash
./scripts/pre-commit.sh
```

This script runs all mandatory checks:

1. **Unused dependencies**: `cargo machete`
2. **All linters**: `cargo run --bin linter all` (stable & nightly)
3. **Tests**: `cargo test`
4. **Documentation builds**: `cargo doc --no-deps --bins --examples --workspace --all-features`
5. **E2E infrastructure tests**: `cargo run --bin e2e-infrastructure-lifecycle-tests`
6. **E2E deployment tests**: `cargo run --bin e2e-deployment-workflow-tests`

**All checks must pass** before committing. Fix any reported issues.

### Running Individual Linters (for debugging)

```bash
cargo run --bin linter markdown   # Markdown
cargo run --bin linter yaml       # YAML
cargo run --bin linter toml       # TOML
cargo run --bin linter clippy     # Rust code analysis
cargo run --bin linter rustfmt    # Rust formatting
cargo run --bin linter shellcheck # Shell scripts
cargo run --bin linter cspell     # Spell checking
```

## Commit Examples

### Feature with Issue Number

```bash
git commit -m "feat: [#42] add support for Ubuntu 22.04 in cloud-init"
```

### Bug Fix with Issue Number

```bash
git commit -m "fix: [#15] resolve ansible inventory parsing error"
```

### Documentation Update

```bash
git commit -m "docs: [#8] add troubleshooting section to README"
```

### With Scope (optional)

```bash
git commit -m "fix(ansible): [#15] correct inventory file path resolution"
```

### Breaking Change (note the !)

```bash
git commit -m "feat!: [#42] change default container provider from multipass to lxd"
```

### Multi-line with Body and Footer

```bash
git commit -m "feat: [#23] add automated testing workflow

Add GitHub Actions workflow that tests:
- LXD container provisioning
- Multipass VM provisioning
- Ansible playbook execution

Closes #23"
```

## Hashtag Usage Warning

**Only use `#` when intentionally referencing a GitHub issue.**

GitHub automatically links `#NUMBER` patterns to issues, so avoid accidental references:

- ✅ **Correct**: `feat: [#42] add new feature` (intentional reference)
- ✅ **Correct**: `Closes #23` in footer (intentional)
- ❌ **Incorrect**: `fix: update config #1 priority` (accidentally links to issue #1)
- ❌ **Incorrect**: `docs: add section #3 to guide` (accidentally links to issue #3)

### Common Mistakes to Avoid

```bash
# Bad: Accidentally links to issue #1
git commit -m "fix: make feature #1 priority"

# Good: Use alternative wording
git commit -m "fix: make feature top priority"
git commit -m "fix: make feature number one priority"

# Bad: Accidentally links to issue #3
git commit -m "docs: add section #3 about deployment"

# Good: Use alternative formatting
git commit -m "docs: add section 3 about deployment"
git commit -m "docs: add third section about deployment"
```

## Commit Quality Guidelines

### Good Commits (✅)

- **Atomic**: Each commit represents one logical change
- **Descriptive**: Clear, concise description of what changed
- **Tested**: All tests pass
- **Linted**: All linters pass
- **Conventional**: Follows conventional commit format

**Example of Good Commit History:**

```text
feat: [#42] add LXD container provisioning support
fix: [#15] resolve ansible inventory template rendering
docs: [#8] update contributing guidelines
test: [#67] add unit tests for configuration parsing
refactor: [#45] extract common logging utilities
```

### Commits to Avoid (❌)

- **Too large**: Multiple unrelated changes in one commit
- **Vague**: Messages like "fix stuff" or "updates"
- **Broken**: Commits that don't build or pass tests
- **Non-conventional**: Not following the conventional commit format

## Complete Workflow Example

```bash
# 1. Make your changes
vim src/main.rs

# 2. Run pre-commit checks (MANDATORY)
./scripts/pre-commit.sh

# 3. If checks fail, fix issues and re-run
#    Repeat until all checks pass

# 4. Stage changes
git add src/main.rs

# 5. Commit with conventional format
git commit -m "feat: [#42] add new CLI command"

# 6. Push to remote
git push origin 42-add-new-cli-command
```

## Troubleshooting

### Pre-commit Script Fails

**Problem**: One or more checks fail in `./scripts/pre-commit.sh`

**Solution**:

1. Read the error output carefully
2. Fix the specific issue (e.g., run `cargo fmt` for formatting)
3. Re-run `./scripts/pre-commit.sh`
4. Repeat until all checks pass

### E2E Tests Take Too Long

**Problem**: E2E tests add significant time to the commit process

**Context**: This is intentional to ensure quality. The split test approach (`e2e-infrastructure-lifecycle-tests` + `e2e-deployment-workflow-tests`) is optimized for GitHub Actions compatibility while maintaining comprehensive coverage.

**Alternative for Local Development**: You can run the full E2E suite manually:

```bash
cargo run --bin e2e-complete-workflow-tests
```

Note: This is only supported in local environments with proper LXD networking and cannot run on GitHub Actions.

### Linter Fails on Nightly Toolchain

**Problem**: `cargo run --bin linter all` fails on nightly Rust

**Solution**:

1. Ensure you have the nightly toolchain: `rustup toolchain install nightly`
2. Update toolchains: `rustup update`
3. Re-run the linter

## Related Documentation

- **Full Commit Process Guide**: `docs/contributing/commit-process.md`
- **Contributing Guide**: `docs/contributing/README.md`
- **Branching Conventions**: `docs/contributing/branching.md`
- **Pre-commit Script**: `scripts/pre-commit.sh`
- **Linting Guide**: `docs/contributing/linting.md`

## Key Reminders

1. **Always run `./scripts/pre-commit.sh` before committing** - This is non-negotiable
2. **Use issue numbers consistently** - Follow the `[#{issue}]` format
3. **Be careful with hashtags** - Only use `#NUMBER` when referencing issues
4. **Keep commits atomic** - One logical change per commit
5. **Write descriptive messages** - Future you will thank present you
