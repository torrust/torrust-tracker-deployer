---
name: cleanup-completed-issues
description: Guide for cleaning up completed and closed issues in the torrust-tracker-deployer project. Covers removing issue documentation files, updating the roadmap, and syncing the GitHub roadmap issue. Supports single issue cleanup or batch cleanup. Use when cleaning up closed issues, removing issue docs, updating roadmap after PR merge, or maintaining docs/issues/ folder. Triggers on "cleanup issue", "remove issue", "clean completed issues", "delete closed issue", "update roadmap after merge".
metadata:
  author: torrust
  version: "1.0"
---

# Cleaning Up Completed Issues

This skill guides you through cleaning up completed and closed issues in the Torrust Tracker Deployer project.

## When to Clean Up

- **Immediately after PR merge**: Remove the specific issue file when its PR is merged
- **Batch cleanup**: Periodically clean up multiple closed issues during maintenance
- **Before releases**: Clean up documentation before major releases
- **After completing epics**: Clean up when a major milestone is finished

## Cleanup Approaches

### Option 1: Single Issue Cleanup (Recommended)

Use this approach when a PR is merged and you want to remove the issue immediately.

**Process:**

1. Verify the issue is closed on GitHub
2. Remove the issue file from `docs/issues/`
3. Remove any manual test documentation from `docs/issues/manual-tests/`
4. Update `docs/roadmap.md` if the issue is a roadmap item
5. Update GitHub roadmap issue #1 if roadmap was modified
6. Commit and push changes

### Option 2: Batch Cleanup

Use this when you want to clean up multiple closed issues at once during maintenance.

**Process:**

1. List all issue files in `docs/issues/`
2. Check status of each issue on GitHub using GitHub CLI
3. Identify all closed issues
4. Remove all closed issue files and their manual test docs
5. Update `docs/roadmap.md` for all removed roadmap items
6. Update GitHub roadmap issue #1
7. Commit and push changes with detailed message

## Step-by-Step Process

### Step 1: Verify Issue is Closed on GitHub

**For single issue:**

```bash
gh issue view {issue-number} --json state --jq .state
```

Expected output: `CLOSED`

**For batch cleanup:**

```bash
# List all issue files
ls docs/issues/ | grep -E '^[0-9]+-' | sed 's/-.*$//'

# Check status of multiple issues
for issue in 21 22 23 24; do
  state=$(gh issue view $issue --json state --jq .state 2>/dev/null || echo "NOT_FOUND")
  echo "$issue:$state"
done
```

### Step 2: Remove Issue Documentation File

**Single issue:**

```bash
# Example: Removing issue #339
git rm docs/issues/339-provide-config-examples-and-questionnaire-for-ai-agents.md
```

**Batch cleanup:**

```bash
# Remove multiple closed issues
git rm docs/issues/21-fix-e2e-infrastructure-preservation.md \
       docs/issues/22-rename-app-commands-to-command-handlers.md \
       docs/issues/23-add-clap-subcommand-configuration.md \
       docs/issues/24-add-user-documentation.md
```

**Important:** Never delete template files:

- `EPIC-TEMPLATE.md`
- `GITHUB-ISSUE-TEMPLATE.md`
- `SPECIFICATION-TEMPLATE.md`

### Step 3: Check for Manual Test Documentation

Some issues may have manual test results in `docs/issues/manual-tests/`:

```bash
# Check if manual test files exist for the closed issue
ls docs/issues/manual-tests/ | grep "^{issue-number}-"

# Examples:
# 248-network-segmentation-test-results.md
# 339-ai-training-examples-validation.md
```

**Remove manual test files if they exist:**

```bash
# Single issue
git rm docs/issues/manual-tests/339-*.md

# Batch cleanup
git rm docs/issues/manual-tests/21-*.md \
       docs/issues/manual-tests/22-*.md \
       docs/issues/manual-tests/23-*.md
```

### Step 4: Update Roadmap (If Applicable)

Check if the closed issue(s) are mentioned in `docs/roadmap.md`.

**Identify roadmap items:**

```bash
# Search for issue number in roadmap
grep -n "#339" docs/roadmap.md
```

**Update roadmap status:**

For completed roadmap items, ensure they are marked with `[x]` and have completion indicators:

```markdown
# Before:

- [ ] **X.Y** Task description - [Issue #339](...)

# After:

- [x] **X.Y** Task description - [Issue #339](...) ✅ Completed
```

**Note:** Do NOT remove completed items from the roadmap—mark them as completed instead. The roadmap serves as a historical record of progress.

### Step 5: Update GitHub Roadmap Issue #1

**IMPORTANT:** Whenever you modify `docs/roadmap.md`, you must also update the mirrored roadmap issue on GitHub.

The roadmap issue is: [https://github.com/torrust/torrust-tracker-deployer/issues/1](https://github.com/torrust/torrust-tracker-deployer/issues/1)

**Process:**

1. Read the updated `docs/roadmap.md` content
2. Navigate to issue #1 on GitHub
3. Click "Edit" on the issue description
4. Replace the body with the updated roadmap content
5. Click "Update comment" to save

**Why this is necessary:**

- The roadmap issue #1 serves as the central tracking point for all project progress
- It provides a single source of truth visible on GitHub
- Epic and task issues link back to it
- Keeping it synchronized ensures accurate project visibility

### Step 6: Commit Changes

**Single issue commit message format:**

```bash
git commit -m "docs: remove completed issue #{number}

The PR #{pr-number} has been merged to main. The issue tracking file is no longer needed."
```

**Example:**

```bash
git add docs/issues/ docs/roadmap.md
git commit -m "docs: remove completed issue #339

The PR #345 has been merged to main. The issue tracking file is no longer needed."
```

**Batch cleanup commit message format:**

```bash
git commit -m "chore: remove closed issue documentation files

Removed X closed issue documentation files from docs/issues/:
- #21: fix-e2e-infrastructure-preservation
- #22: rename-app-commands-to-command-handlers
- #23: add-clap-subcommand-configuration
- #24: add-user-documentation

Also removed associated manual test documentation from docs/issues/manual-tests/.

Updated roadmap to mark completed items.

All these issues have been closed on GitHub and no longer need
local documentation files.

Remaining open issues: #16, #17, #18, #19, #34"
```

### Step 7: Push Changes

```bash
git push origin main
```

## Complete Example: Single Issue Cleanup

User says: "The PR #339 has been merged. Clean up the issue."

**Actions:**

```bash
# Step 1: Verify issue is closed
gh issue view 339 --json state --jq .state
# Output: CLOSED

# Step 2: Remove issue file
git rm docs/issues/339-provide-config-examples-and-questionnaire-for-ai-agents.md

# Step 3: Check for manual test docs
ls docs/issues/manual-tests/ | grep "^339-"
# (No manual test files found for this issue)

# Step 4: Update roadmap
# Check if issue #339 is in roadmap
grep -n "#339" docs/roadmap.md
# (Found on line 245)

# Edit docs/roadmap.md to mark task as completed
# Change: - [ ] **11.3** ... to: - [x] **11.3** ... ✅ Completed

# Step 5: Update GitHub roadmap issue #1
# Navigate to https://github.com/torrust/torrust-tracker-deployer/issues/1
# Edit the issue body to include the updated roadmap content

# Step 6: Commit
git add docs/issues/ docs/roadmap.md
git commit -m "docs: remove completed issue #339

The PR #345 has been merged to main. The issue tracking file is no longer needed."

# Step 7: Push
git push origin main
```

## Complete Example: Batch Cleanup

User says: "Clean up all closed issues."

**Actions:**

```bash
# Step 1: List issue files and check status
for file in docs/issues/[0-9]*.md; do
  issue=$(echo $file | grep -oP '\d+' | head -1)
  state=$(gh issue view $issue --json state --jq .state 2>/dev/null || echo "NOT_FOUND")
  echo "$issue:$state:$file"
done

# Output:
# 21:CLOSED:docs/issues/21-fix-e2e-infrastructure.md
# 22:CLOSED:docs/issues/22-rename-commands.md
# 23:CLOSED:docs/issues/23-add-clap-config.md
# 34:OPEN:docs/issues/34-implement-create-command.md

# Step 2 & 3: Remove closed issue files and manual tests
git rm docs/issues/21-fix-e2e-infrastructure.md \
       docs/issues/22-rename-commands.md \
       docs/issues/23-add-clap-config.md

# Check for manual test docs for these issues
git rm docs/issues/manual-tests/21-*.md docs/issues/manual-tests/22-*.md 2>/dev/null || true

# Step 4: Update roadmap for each closed issue
# Edit docs/roadmap.md to mark tasks 21, 22, 23 as completed

# Step 5: Update GitHub roadmap issue #1
# (Update issue #1 body with the modified roadmap)

# Step 6: Commit
git add docs/issues/ docs/roadmap.md
git commit -m "chore: remove closed issue documentation files

Removed 3 closed issue documentation files from docs/issues/:
- #21: fix-e2e-infrastructure
- #22: rename-commands
- #23: add-clap-config

Updated roadmap to mark completed items.

All these issues have been closed on GitHub and no longer need
local documentation files.

Remaining open issues: #34, #35, #36"

# Step 7: Push
git push origin main
```

## Verification Checklist

After cleanup, verify:

- [ ] Issue file removed from `docs/issues/`
- [ ] Manual test files removed from `docs/issues/manual-tests/` (if they existed)
- [ ] Template files NOT deleted (`*-TEMPLATE.md`)
- [ ] Roadmap updated if issue was a roadmap item
- [ ] GitHub roadmap issue #1 updated if roadmap was modified
- [ ] Commit message lists which issues were removed
- [ ] Changes pushed to remote

## Common Mistakes to Avoid

❌ **Don't delete template files** (`EPIC-TEMPLATE.md`, `GITHUB-ISSUE-TEMPLATE.md`, `SPECIFICATION-TEMPLATE.md`)
❌ **Don't forget manual test documentation** in `docs/issues/manual-tests/`
❌ **Don't remove roadmap items** from `docs/roadmap.md`—mark them as completed instead
❌ **Don't forget to update GitHub roadmap issue #1** when modifying `docs/roadmap.md`
❌ **Don't delete issues that are still open** on GitHub

✅ **Do verify issue status** before deletion
✅ **Do update roadmap** for roadmap items
✅ **Do sync GitHub issue #1** with roadmap changes
✅ **Do check for associated manual tests**
✅ **Do use descriptive commit messages**

## References

- Detailed cleanup guide: `docs/contributing/roadmap-issues.md` (Section: "Cleaning Up Closed Issues")
- Roadmap document: `docs/roadmap.md`
- GitHub roadmap issue: [Issue #1](https://github.com/torrust/torrust-tracker-deployer/issues/1)
- Commit conventions: `docs/contributing/commit-process.md`

## GitHub CLI Quick Reference

```bash
# Check single issue status
gh issue view {number} --json state --jq .state

# Check issue and get title
gh issue view {number} --json state,title --jq '{state: .state, title: .title}'

# List all closed issues (filter in repo)
gh issue list --state closed --limit 100
```

## Pro Tips

- **Clean up regularly**: Don't let `docs/issues/` accumulate too many closed issues
- **Clean immediately after merge**: Easier than batch cleanup later
- **Keep roadmap synchronized**: Always update both `docs/roadmap.md` and GitHub issue #1
- **Document removals**: Use clear commit messages listing removed issues
- **Git history**: Deleted files remain in git history if needed for reference
