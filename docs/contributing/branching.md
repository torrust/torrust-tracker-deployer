# Branching Conventions

This document outlines the branching conventions for the Torrust Testing Infrastructure project.

## 🌿 Branch Naming

- **Format**: `{issue-number}-{short-description-following-github-conventions}`
- **GitHub conventions**: Use lowercase, separate words with hyphens, descriptive but concise
- **Examples**: `42-add-mysql-support`, `15-fix-ssl-renewal`, `24-improve-ux-add-automatic-waiting-to-infra-apply-and-app-deploy-commands`
- Always start with the GitHub issue number
- Follow GitHub's recommended branch naming: lowercase, hyphens for word separation, descriptive of the change

### Examples

```bash
# Good branch names
git checkout -b 42-add-mysql-support
git checkout -b 15-fix-ssl-renewal
git checkout -b 89-update-contributing-guide
git checkout -b 156-refactor-ansible-inventory-structure
git checkout -b 203-add-e2e-multipass-tests

# Avoid
git checkout -b my-feature        # No issue number
git checkout -b FEATURE-123       # All caps
git checkout -b fix_bug           # Underscores instead of hyphens
git checkout -b 42_add_support    # Underscores instead of hyphens
git checkout -b 42-Add-Support    # Mixed case
```

## 📋 Branch Lifecycle

1. **Create**: `git checkout -b {issue-number}-{description}`
2. **Develop**: Make commits following [commit conventions](./commit-process.md)
3. **Test**: Run linters and tests before pushing
4. **Push**: `git push origin {issue-number}-{description}`
5. **PR**: Create pull request via GitHub
6. **Review**: Address feedback from maintainers
7. **Merge**: Squash and merge when approved
8. **Cleanup**: Delete branch after merge

```bash
# After merge, clean up local branch
git checkout main
git pull origin main
git branch -d 42-add-mysql-support
```

## 🚫 What to Avoid

### Branch Names to Avoid

```bash
# No issue number
git checkout -b add-mysql-support
git checkout -b fix-ssl-issue

# Personal references
git checkout -b jose-feature
git checkout -b my-branch

# Wrong format
git checkout -b issue-42
git checkout -b 42_add_support
git checkout -b 42-Add-MySQL-Support

# Special characters
git checkout -b 42-add@new#feature
```
