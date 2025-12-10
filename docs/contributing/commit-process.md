# Commit Process and Pre-commit Checks

This document outlines the commit process, including required pre-commit checks and conventions for the Torrust Tracker Deployer project.

## 📝 Conventional Commits

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages.

### Commit Message Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Issue Number Convention

When working on a branch with an issue number, include the issue number in your commit messages:

- **Structure**: `{type}: [#{issue}] {description}`
- **Examples**:

  ```text
  feat: [#42] add MySQL database support
  fix: [#15] resolve SSL certificate renewal issue
  docs: [#8] update deployment guide
  ci: [#23] add infrastructure validation tests
  ```

### Commit Types

| Type       | Description                           | Example                                                |
| ---------- | ------------------------------------- | ------------------------------------------------------ |
| `feat`     | New feature or enhancement            | `feat: [#42] add LXD container provisioning`           |
| `fix`      | Bug fix                               | `fix: [#15] resolve ansible inventory parsing error`   |
| `docs`     | Documentation changes                 | `docs: [#8] update installation guide`                 |
| `style`    | Code style changes (formatting, etc.) | `style: [#31] apply rustfmt to all source files`       |
| `refactor` | Code refactoring                      | `refactor: [#45] simplify linting script structure`    |
| `test`     | Adding or updating tests              | `test: [#67] add e2e tests for multipass provisioning` |
| `chore`    | Maintenance tasks                     | `chore: [#89] update dependencies to latest versions`  |
| `ci`       | CI/CD related changes                 | `ci: [#23] add workflow for testing provisioning`      |
| `perf`     | Performance improvements              | `perf: [#52] optimize container startup time`          |

### Commit Examples

```bash
# Feature addition with issue number
git commit -m "feat: [#42] add support for Ubuntu 22.04 in cloud-init"

# Bug fix with issue number
git commit -m "fix: [#15] resolve ansible inventory parsing error"

# Documentation update with issue number
git commit -m "docs: [#8] add troubleshooting section to README"

# CI/CD changes with issue number
git commit -m "ci: [#23] add workflow for testing provisioning"

# Bug fix with scope (optional format)
git commit -m "fix(ansible): [#15] correct inventory file path resolution"

# Breaking change with issue number (note the !)
git commit -m "feat!: [#42] change default container provider from multipass to lxd"

# Commit with body and footer including issue reference
git commit -m "feat: [#23] add automated testing workflow

Add GitHub Actions workflow that tests:
- LXD container provisioning
- Multipass VM provisioning
- Ansible playbook execution

Closes #23"
```

### ⚠️ Important: Hashtag Usage in Commit Messages

**Only use the `#` character when intentionally referencing a GitHub issue.**

GitHub automatically links any `#NUMBER` pattern in commit messages to the corresponding issue. This means:

- ✅ **Correct**: `feat: [#42] add new feature` - Links to issue #42 (intentional)
- ✅ **Correct**: `Closes #23` in commit body/footer - Links to issue #23 (intentional)
- ❌ **Incorrect**: `fix: update config #1 priority` - Accidentally links to issue #1
- ❌ **Incorrect**: `docs: add section #3 to guide` - Accidentally links to issue #3
- ❌ **Incorrect**: `test: verify test case #5 works` - Accidentally links to issue #5

**Common Mistakes to Avoid:**

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

**When you DO want to reference an issue:**

```bash
# Correct: Issue reference in standardized format
git commit -m "feat: [#42] add new feature"

# Correct: Issue reference in footer
git commit -m "feat: add new feature

Closes #42"
```

If you accidentally link a commit to an issue, you'll need to amend the commit message to remove the unintended `#NUMBER` pattern.

## ✅ Pre-commit Checklist

Before committing any changes, you **MUST** run the pre-commit verification script:

```bash
./scripts/pre-commit.sh
```

This script runs all mandatory checks:

1. **Check for unused dependencies**: `cargo machete`
2. **Run all linters**: `cargo run --bin linter all` (stable & nightly toolchains)
3. **Run tests**: `cargo test`
4. **Test documentation builds**: `cargo doc --no-deps --bins --examples --workspace --all-features`
5. **Run E2E infrastructure lifecycle tests**: `cargo run --bin e2e-infrastructure-lifecycle-tests`
6. **Run E2E deployment workflow tests**: `cargo run --bin e2e-deployment-workflow-tests`

**Note**: Code coverage is checked automatically in CI via GitHub Actions, not in the pre-commit script, to keep local commits fast and efficient.

**All checks must pass** before committing. Fix any reported issues.

### E2E Test Execution Strategy

The pre-commit script runs E2E tests as **two separate commands** instead of a single comprehensive test:

- **Provision and Destroy Tests**: Test infrastructure lifecycle (LXD VMs)
- **Configuration Tests**: Test software installation and configuration (Docker containers)

This split approach provides the same comprehensive coverage while being compatible with GitHub Actions runners. The split is necessary because GitHub Actions has networking limitations with nested LXD VMs that prevent the full E2E test suite from running.

**For local development**, you can still run the full E2E test suite manually for convenience:

```bash
cargo run --bin e2e-tests-full
```

This provides the same coverage in a single run but is only supported in local environments with proper LXD networking.

### Running Individual Linters

If you need to run specific linters for debugging:

```bash
cargo run --bin linter markdown   # Markdown
cargo run --bin linter yaml       # YAML
cargo run --bin linter clippy     # Rust code analysis
cargo run --bin linter rustfmt    # Rust formatting
cargo run --bin linter shellcheck # Shell scripts
```

## 📋 Commit Quality Guidelines

### Good Commits

✅ **Atomic**: Each commit represents one logical change  
✅ **Descriptive**: Clear, concise description of what changed  
✅ **Tested**: All tests pass  
✅ **Linted**: All linters pass  
✅ **Conventional**: Follows conventional commit format

### Example of Good Commit History

```text
feat: add LXD container provisioning support
fix: resolve ansible inventory template rendering
docs: update contributing guidelines
test: add unit tests for configuration parsing
refactor: extract common logging utilities
```

### Commits to Avoid

❌ **Too large**: Multiple unrelated changes in one commit  
❌ **Vague**: Messages like "fix stuff" or "updates"  
❌ **Broken**: Commits that don't build or pass tests  
❌ **Non-conventional**: Not following the conventional commit format

### Example of Poor Commit History

```text
fix stuff
WIP
updates
more changes
final fix
actually final fix
```
