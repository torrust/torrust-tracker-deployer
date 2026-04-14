---
name: open-pull-request
description: Open a pull request from a feature branch using GitHub CLI (preferred) or GitHub MCP tools. Covers pre-flight checks, correct base/head configuration for fork workflows, title/body conventions, and post-creation validation. Use when asked to "open PR", "create pull request", or "submit branch for review".
metadata:
  author: torrust
  version: "1.0"
---

# Open a Pull Request

This skill explains how to create a pull request for this repository in a repeatable way.

## CLI vs MCP decision rule

Use the tool that matches the loop:

- **Inner loop (fast local branch work):** prefer GitHub CLI (`gh`) because it is fast and low overhead.
- **Outer loop (cross-system coordination):** use MCP when you need structured/authenticated access across shared systems.

For opening a PR from the current local branch, prefer `gh pr create`.

## Pre-flight checks

Before opening a PR:

- [ ] Working tree is clean (`git status`)
- [ ] Branch is pushed to remote
- [ ] Commits are signed (`git log --show-signature -n 1`)
- [ ] Required checks have been run (`./scripts/pre-commit.sh`)

## Title and description convention

Use conventional commit style in the PR title when possible, including issue reference.

Examples:

- `ci: [#448] add crate publish workflow`
- `docs: [#448] define release process`

Include in PR body:

- Summary of changes
- Files/workflows touched
- Validation performed
- Issue link (`Closes #<issue-number>`)

## Option A (Preferred): GitHub CLI

### Same-repo branch

```bash
gh pr create \
  --repo torrust/torrust-tracker-deployer \
  --base main \
  --head <branch-name> \
  --title "<title>" \
  --body "<body>"
```

### Fork branch (common maintainer flow)

```bash
gh pr create \
  --repo torrust/torrust-tracker-deployer \
  --base main \
  --head <fork-owner>:<branch-name> \
  --title "<title>" \
  --body "<body>"
```

If successful, `gh` prints the PR URL.

## Option B: GitHub MCP tools

When MCP pull request management tools are available:

1. Create branch remotely if needed
2. Open PR with base `main` and correct head branch
3. Capture and share resulting PR URL

## Post-creation validation

After PR creation:

- [ ] Verify PR points to `torrust/torrust-tracker-deployer:main`
- [ ] Verify head branch is correct
- [ ] Confirm CI workflows started
- [ ] Confirm issue is linked in description

## Troubleshooting

- `fatal: ... does not appear to be a git repository`: push to correct remote (`git remote -v`)
- `A pull request already exists`: open existing PR URL instead of creating a new one
- Permission errors on upstream repo: create PR from your fork branch (`owner:branch`)

## References

- [`docs/contributing/commit-process.md`](../../../../../docs/contributing/commit-process.md)
- [`docs/contributing/pr-review-guide.md`](../../../../../docs/contributing/pr-review-guide.md)
- Existing branch skill: `.github/skills/dev/git-workflow/create-feature-branch/skill.md`
