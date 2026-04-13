---
name: update-dependencies
description: Guide for updating project dependencies using the update-dependencies.sh automation script. Automates the cargo update workflow including branch creation, commit, push, and optional PR creation. Use when updating dependencies, running cargo update, or automating the dependency lifecycle. Triggers on "update dependencies", "cargo update", "update deps", "bump dependencies", or "run dependency update".
metadata:
  author: torrust
  version: "1.0"
---

# Updating Dependencies

This skill guides you through updating project dependencies using the `scripts/update-dependencies.sh` automation script, which handles the complete dependency update workflow from branch creation to PR submission.

## Quick Reference

```bash
# Simple update (no issue)
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote} \
  --create-pr

# Complex update with issue
./scripts/update-dependencies.sh \
  --branch {issue-number}-update-dependencies \
  --push-remote {fork-remote} \
  --create-pr
```

## Workflow Overview

The `update-dependencies.sh` script automates the following steps:

1. Ensures a clean working tree (no uncommitted changes)
2. Fetches and fast-forwards the base branch from upstream
3. Creates a feature branch with the specified name
4. Runs `cargo update` and captures the full output
5. Exits early if no `Cargo.lock` changes are produced
6. Optionally runs `./scripts/pre-commit.sh` (default: enabled)
7. **Commits** the `Cargo.lock` changes with full `cargo update` output in commit body
8. **Pushes** the branch to the fork remote
9. **Creates a PR** on GitHub (optional, default: disabled)

## Usage

### Basic Invocation

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote}
```

### With PR Creation

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote} \
  --create-pr
```

### Signing Commits

Commits are **always signed** with `git commit -S` (GPG signing is mandatory):

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote}
```

Unsigned commits are not permitted in this workflow.

### Skipping Pre-Commit Checks

Pre-commit checks are run by default. Skip them if needed (not recommended):

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote} \
  --skip-pre-commit
```

### Deleting Existing Branch

If a branch with the same name already exists, delete it first:

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote} \
  --delete-existing-branch
```

## All Options

```bash
./scripts/update-dependencies.sh --help
```

| Option                     | Default                      | Description                                            |
| -------------------------- | ---------------------------- | ------------------------------------------------------ |
| `--branch`                 | **required**                 | Feature branch name (e.g., `123-update-deps`)          |
| `--base-branch`            | `main`                       | Target branch for merge base                           |
| `--base-remote`            | Auto-detected                | Remote for base branch (prefers `torrust` → `origin`)  |
| `--push-remote`            | Auto-detected                | Remote to push the branch to                           |
| `--repo`                   | Auto-detected                | GitHub repo slug (owner/repo)                          |
| `--commit-title`           | `chore: update dependencies` | First line of commit message                           |
| `--pr-title`               | `chore: update dependencies` | Pull request title                                     |
| `--skip-pre-commit`        | disabled                     | Skip `./scripts/pre-commit.sh` after update            |
| `--create-pr`              | disabled                     | Create a PR after pushing                              |
| `--delete-existing-branch` | disabled                     | Delete the branch locally and remotely before starting |
| `--help`                   | —                            | Show full usage and all options                        |

## When to Create an Issue

- **Simple updates**: Just running `cargo update` with no special handling → **No issue needed**, use branch name `update-dependencies`
- **Complex updates**: Dependency updates requiring additional changes (migrations, API updates, refactoring) → **Create an issue** and use branch name `{issue-number}-update-dependencies`

This keeps the issue tracker focused on substantial work while allowing for routine maintenance tasks without issue clutter.

## Step-by-Step Example (Simple Update)

### 1. Run the Script (No Issue)

```bash
./scripts/update-dependencies.sh \
  --branch update-dependencies \
  --push-remote {fork-remote} \
  --create-pr
```

## Step-by-Step Example (Complex Update with Issue)

### 1. Create an Issue

```bash
gh issue create \
  --title "chore: update dependencies and migrate to new API" \
  --body "Update dependencies and handle breaking changes in async library."
```

Note the issue number (e.g., `#456`).

### 2. Run the Script

```bash
./scripts/update-dependencies.sh \
  --branch {issue-number}-update-dependencies \
  --push-remote {fork-remote} \
  --create-pr
```

### 3. Observe the Output

The script will:

- Fetch the latest `main` from `torrust` remote
- Create branch `456-update-dependencies`
- Run `cargo update` and show the output
- If dependencies changed: run pre-commit, commit with full output, push, create PR
- If no changes: clean up branch and exit (no-op, which is fine)

### 4. Review and Merge

- Visit the PR created by the script (URL printed to stdout)
- Review the `cargo update` output in the commit body
- Let CI checks pass
- Merge when ready

## Commit Message Format

The script generates commit messages in this format:

```
chore: update dependencies

[Full cargo update output]

- run `cargo update`
- commit the resulting `Cargo.lock` changes
```

The full `cargo update` output is included in the commit body for traceability. Example:

```
    Updating crates.io index
     Locking 14 packages to latest compatible versions
     Updating hyper-rustls from 0.27.7 to 0.27.8
     ...
```

## Pre-Commit Checks

Before committing, the script optionally runs `./scripts/pre-commit.sh` (enabled by default), which verifies:

1. **Unused dependencies**: `cargo machete`
2. **All linters**: markdown, YAML, TOML, spelling, Clippy, rustfmt, shellcheck
3. **Tests**: `cargo test`
4. **Documentation**: `cargo doc --no-deps`
5. **E2E infrastructure tests**: provisioning and destruction
6. **E2E deployment tests**: full workflow

If pre-commit fails, the script exits before committing. Fix issues and run the script again.

### Skip Pre-Commit (Not Recommended)

```bash
./scripts/update-dependencies.sh \
  --branch {issue-number}-update-dependencies \
  --push-remote {fork-remote} \
  --skip-pre-commit
```

## When Dependencies Don't Change

If `cargo update` produces no changes to `Cargo.lock`, the script will:

1. Print: `No dependency changes were produced by cargo update`
2. Clean up the feature branch (delete it locally)
3. Exit cleanly with code 0 (success)

This is **not an error** — it means all dependencies are at their latest compatible versions.

## Troubleshooting

### Error: Working tree has unstaged changes

The script requires a clean working tree. Stage or remove all uncommitted changes:

```bash
git status                      # See what's uncommitted
git add <files>                 # Stage changes
git commit -m "..."             # Or commit them
git stash                       # Or stash them
```

Then retry the script.

### Error: Branch already exists

The branch exists either locally or on the remote:

```bash
# Option 1: Use a different branch name
./scripts/update-dependencies.sh \
  --branch {issue-number}-update-dependencies-retry \
  --push-remote {fork-remote}

# Option 2: Delete the existing branch first
./scripts/update-dependencies.sh \
  --branch {issue-number}-update-dependencies \
  --push-remote {fork-remote} \
  --delete-existing-branch
```

### Commit Signing Failures

GPG signing is mandatory. If commit signing fails:

```bash
# Check GPG setup
gpg --list-keys

# Fix GPG configuration, then retry
gh auth logout && gh auth login  # Re-authenticate if needed
```

Ensure GPG is properly configured before running the script. Unsigned commits are not permitted.

### PR Creation Fails

Ensure `gh` CLI is authenticated:

```bash
gh auth status          # Check authentication
gh auth login           # Log in if needed
```

Then retry with `--create-pr`.

## After Merge

Once the PR is merged to `main`:

1. The updated `Cargo.lock` is now in the base branch
2. All future branches will build with the new dependencies
3. Repeat the workflow for the next update cycle (typically monthly or as needed)

## Integration with CI

When the PR is created, GitHub Actions will automatically run:

- All linters (stable + nightly)
- Full test suite
- E2E infrastructure tests
- E2E deployment tests
- Coverage analysis

All checks must pass before merging.

## Best Practices

- **Run regularly**: Update dependencies monthly or quarterly
- **Always sign commits**: Use GPG signing (default behavior) for audit trails
- **Review the output**: Check the commit message to see which packages were updated
- **Run pre-commit**: Never skip this step (use default behavior)
- **Use issue numbers**: Prefix branch names with issue numbers (`--branch {issue}-update-dependencies`)
- **One branch per run**: Each run creates one branch and optionally one PR
- **Wait for CI**: Never merge until all checks pass

## See Also

- [Committing Changes](../../git-workflow/commit-changes/skill.md) — General commit workflow
- [Creating Feature Branches](../../git-workflow/create-feature-branch/skill.md) — Branch naming conventions
- [Pre-Commit Checks](../../git-workflow/run-pre-commit-checks/skill.md) — Understanding the 6-step verification process
