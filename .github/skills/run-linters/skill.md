---
name: run-linters
description: Run code quality checks and linters for the deployer project. Includes markdown, YAML, TOML, spell checking, Rust clippy, rustfmt, and shellcheck. Use when user needs to lint code, check formatting, fix code quality issues, or prepare for commit. Triggers on "lint", "run linters", "check code quality", "fix formatting", or "run pre-commit checks".
metadata:
  author: torrust
  version: "1.0"
---

# Run Linters

This skill helps you run code quality checks and linters for the Torrust Tracker Deployer project.

## When to Use This Skill

Use this skill when you need to:

- **Before committing code** (mandatory) - All checks must pass before staging files
- **After making code changes** - Verify your changes meet quality standards
- **When fixing CI failures** - Diagnose and fix linting issues that failed in CI
- **When code quality issues are reported** - Address specific linter warnings

## Available Linters

The project provides a unified linting framework through the `linter` binary:

### Run All Linters

```bash
cargo run --bin linter all
```

This runs all linters in sequence (comprehensive check before commits).

### Run Individual Linters

```bash
cargo run --bin linter markdown    # Markdown files
cargo run --bin linter yaml        # YAML configuration files
cargo run --bin linter toml        # TOML files (Cargo.toml, etc.)
cargo run --bin linter cspell      # Spell checking
cargo run --bin linter clippy      # Rust lint checks
cargo run --bin linter rustfmt     # Rust code formatting
cargo run --bin linter shellcheck  # Shell script analysis
```

### Alternative: Shell Script Wrapper

```bash
./scripts/lint.sh
```

This wrapper script calls the Rust linter binary for convenience.

## Common Workflows

### Workflow 1: Pre-Commit Checks (Required)

Before staging any files or creating commits, **always** run:

```bash
./scripts/pre-commit.sh
```

This script runs:

1. Unused dependency check (`cargo machete`)
2. All linters (`cargo run --bin linter all`)
3. Unit tests (`cargo test`)
4. Documentation build (`cargo doc`)
5. E2E tests (infrastructure lifecycle + deployment workflow)

**All checks must pass** before proceeding with `git add` and `git commit`.

### Workflow 2: Fix Specific Linter Errors

When a specific linter fails:

```bash
# Fix markdown issues
cargo run --bin linter markdown

# Fix YAML issues
cargo run --bin linter yaml

# Fix Rust formatting
cargo run --bin linter rustfmt
```

### Workflow 3: Fast Development Checks

During active development, run faster linters frequently:

```bash
cargo run --bin linter clippy    # Rust lint (fast)
cargo run --bin linter rustfmt   # Rust format (fast)
cargo run --bin linter markdown  # Markdown (fast)
```

Save comprehensive checks (`cargo run --bin linter all`) for before commits.

## Fixing Common Issues

### Markdown Formatting Issues

**Error**: Lines too long, trailing spaces, or heading format issues

**Solution**:

- Check `.markdownlint.json` for rules
- Manually fix reported line numbers
- Re-run until clean

### YAML Syntax Errors

**Error**: Indentation, line length, or syntax problems

**Solution**:

- Check `.yamllint-ci.yml` for rules
- Fix indentation (2 spaces for YAML)
- Verify syntax with a YAML validator

### Spelling Mistakes

**Error**: Unknown words flagged by cspell

**Solution**:

- Add project-specific terms to `project-words.txt`
- Use correct spelling for common words
- Check `cspell.json` for configuration

### Rust Formatting and Clippy Warnings

**Error**: Code style violations or potential bugs

**Solution**:

```bash
# Apply automatic formatting
cargo fmt

# Fix clippy warnings
cargo clippy --fix --allow-dirty --allow-staged
```

### Shell Script Issues

**Error**: Shellcheck warnings in bash scripts

**Solution**:

- Review shellcheck documentation for specific warning codes
- Fix script issues (quoting, variable usage, etc.)
- Test scripts after changes

## Integration with Development Workflow

### Pre-Commit Hook

The project provides a pre-commit verification script:

```bash
./scripts/pre-commit.sh
```

This is the **single source of truth** for all quality checks. Run it before every commit.

### CI Pipeline Requirements

GitHub Actions CI runs the same checks as the pre-commit script. Ensure local checks pass to avoid CI failures.

### When Linting Checks are Mandatory

Linting checks are **always mandatory** before:

- Staging files (`git add`)
- Creating commits (`git commit`)
- Pushing to remote (`git push`)
- Opening pull requests

## Detailed Linter Information

For detailed configuration and reference information about each linter, see:

- [references/linters.md](references/linters.md) - Comprehensive linter documentation
- [docs/contributing/linting.md](../../../docs/contributing/linting.md) - Linting guide
- [packages/linting/README.md](../../../packages/linting/README.md) - Linting framework details

## Quick Reference

| Task                   | Command                             |
| ---------------------- | ----------------------------------- |
| Run all linters        | `cargo run --bin linter all`        |
| Run pre-commit checks  | `./scripts/pre-commit.sh`           |
| Fix Rust formatting    | `cargo fmt`                         |
| Fix clippy issues      | `cargo clippy --fix`                |
| Check markdown         | `cargo run --bin linter markdown`   |
| Check spelling         | `cargo run --bin linter cspell`     |
| Add word to dictionary | Edit `project-words.txt`            |
| Check shell scripts    | `cargo run --bin linter shellcheck` |

## Tips

- **Run linters frequently** during development to catch issues early
- **Use individual linters** for fast feedback while coding
- **Always run full pre-commit checks** before staging files
- **Add project-specific terms** to `project-words.txt` for spell checking
- **Reference linter configs** in project root for rule details
