---
name: create-feature-branch
description: Guide for creating feature branches following the torrust-tracker-deployer branching conventions. Covers branch naming format, lifecycle, and common patterns. Use when creating branches for issues, starting work on tasks, or setting up development branches. Triggers on "create branch", "new branch", "checkout branch", "branch for issue", or "start working on issue".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Feature Branches

This skill guides you through creating feature branches following the Torrust Tracker Deployer branching conventions.

## Branch Naming Convention

**Format**: `{issue-number}-{short-description-following-github-conventions}`

**Rules**:

- Always start with the GitHub issue number
- Use lowercase letters only
- Separate words with hyphens (not underscores)
- Keep description concise but descriptive
- Follow GitHub's recommended conventions

## Creating a Branch

### Standard Workflow

```bash
# Create and checkout branch for issue #42
git checkout -b 42-add-mysql-support

# Create and checkout branch for issue #156
git checkout -b 156-refactor-ansible-inventory-structure
```

### With MCP GitHub Tools

When using GitHub MCP tools, use `mcp_github_github_create_branch` to create the branch remotely:

1. Get the issue number and title
2. Format the branch name: `{number}-{kebab-case-description}`
3. Create the branch from `main` (or specified base branch)
4. Checkout locally: `git fetch && git checkout {branch-name}`

## Branch Naming Examples

✅ **Good branch names**:

- `42-add-mysql-support` - Issue #42 about adding MySQL
- `15-fix-ssl-renewal` - Issue #15 fixing SSL renewal
- `89-update-contributing-guide` - Issue #89 updating docs
- `156-refactor-ansible-inventory-structure` - Issue #156 refactoring
- `203-add-e2e-multipass-tests` - Issue #203 adding tests

❌ **Avoid**:

- `my-feature` - No issue number
- `FEATURE-123` - All caps
- `fix_bug` - Underscores instead of hyphens
- `42_add_support` - Underscores instead of hyphens
- `42-Add-Support` - Mixed case (not lowercase)
- `add-mysql-support` - Missing issue number

## Complete Branch Lifecycle

### 1. Create Branch

```bash
git checkout -b 42-add-mysql-support
```

### 2. Develop

Make commits following [commit conventions](../../../docs/contributing/commit-process.md):

- With issue branch: `{type}: [#42] {description}`
- Without issue branch: `{type}: {description}`

### 3. Test Before Push

```bash
./scripts/pre-commit.sh
```

All checks must pass before pushing.

### 4. Push to Remote

```bash
git push origin 42-add-mysql-support
```

### 5. Create Pull Request

Create PR via GitHub with:

- Clear title and description
- Link to issue in description
- Proper labels

### 6. Review and Merge

Address feedback, then squash and merge when approved.

### 7. Cleanup After Merge

```bash
git checkout main
git pull origin main
git branch -d 42-add-mysql-support
```

## Common Patterns

### Starting Work on a New Issue

```bash
# Ensure you're on latest main
git checkout main
git pull origin main

# Create branch for issue #42
git checkout -b 42-add-mysql-support

# Verify branch name
git branch --show-current
```

### Converting Issue Title to Branch Name

**Process**:

1. Get issue number (e.g., #42)
2. Take issue title (e.g., "Add MySQL Support")
3. Convert to lowercase kebab-case: `add-mysql-support`
4. Prefix with issue number: `42-add-mysql-support`

**Examples**:

| Issue # | Title                                | Branch Name                                |
| ------- | ------------------------------------ | ------------------------------------------ |
| 42      | Add MySQL Support                    | `42-add-mysql-support`                     |
| 15      | Fix SSL Certificate Renewal          | `15-fix-ssl-renewal`                       |
| 156     | Refactor Ansible Inventory Structure | `156-refactor-ansible-inventory-structure` |
| 24      | Improve UX - Add Automatic Waiting   | `24-improve-ux-add-automatic-waiting`      |
| 203     | Add E2E Tests for Multipass          | `203-add-e2e-multipass-tests`              |

### Branch Already Exists Locally

```bash
# If branch exists locally but not checked out
git checkout 42-add-mysql-support

# If branch exists remotely but not locally
git fetch origin
git checkout 42-add-mysql-support
```

### Switching Between Branches

```bash
# Switch to another branch
git checkout 15-fix-ssl-renewal

# Return to main
git checkout main

# List all branches
git branch -a
```

## Validation Checklist

Before pushing your branch:

- ✅ Branch name starts with issue number
- ✅ Branch name uses lowercase only
- ✅ Branch name uses hyphens (not underscores)
- ✅ Branch name is descriptive but concise
- ✅ Pre-commit checks pass (`./scripts/pre-commit.sh`)
- ✅ Commits follow commit conventions

## Related Documentation

- [Branching Conventions](../../../docs/contributing/branching.md) - Full branching documentation
- [Commit Process](../../../docs/contributing/commit-process.md) - Commit message conventions
- [Contributing Guide](../../../docs/contributing/README.md) - Overall contribution workflow
