# Commit Process and Pre-commit Checks

This document outlines the commit process, including required pre-commit checks and conventions for the Torrust Tracker Deploy project.

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

## ✅ Pre-commit Checklist

Before committing any changes, you **MUST** complete the following checks:

### 1. Run All Linters

```bash
# Run all linters at once
cargo run --bin linter all

# Or run individually
cargo run --bin linter markdown   # Markdown
cargo run --bin linter yaml       # YAML
cargo run --bin linter clippy     # Rust code analysis
cargo run --bin linter rustfmt    # Rust formatting
cargo run --bin linter shellcheck # Shell scripts
```

**All linters must pass** before committing. Fix any reported issues.

### 2. Run Tests

```bash
# Run Rust unit tests
cargo test

# Run E2E tests (if applicable to your changes)
cargo run --bin e2e-tests-full
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
