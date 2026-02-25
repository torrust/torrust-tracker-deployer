---
name: complete-feature-spec
description: Guide for completing feature specifications in the torrust-tracker-deployer project. Covers moving entries from active to completed features, updating status, and preserving documentation. Use when finishing feature implementation, marking features as done, or updating the features index. Triggers on "complete feature", "finish feature", "feature done", "mark feature complete", or "feature completed".
metadata:
  author: torrust
  version: "1.0"
---

# Completing Feature Specifications

This skill guides you through marking a feature as complete in the Torrust Tracker Deployer project.

## Quick Reference

```bash
# 1. Verify all implementation is merged
# 2. Update feature README status to ✅ Completed
# 3. Move entry from active-features.md to completed-features.md
# 4. Keep the feature folder — do NOT delete it
# 5. Commit changes
```

## Key Difference From Refactoring Plans

> **Features are kept permanently. Refactoring plans are deleted.**

When a refactoring plan is completed, the plan document is deleted because the work lives in git history. Features are **different**:

- Feature specifications serve as permanent reference documentation
- They explain _why_ a feature exists and what requirements drove it
- They are useful for future contributors, audits, and follow-up work
- The feature folder and all its documents remain in the repository

Only the **index entry** moves — from `active-features.md` to `completed-features.md`.

## When to Complete a Feature

Complete a feature when:

- ✅ All implementation issues are closed and merged to main
- ✅ All tests pass
- ✅ All linters pass
- ✅ Feature behavior matches the specification's acceptance criteria
- ✅ No open implementation work remains

**Do not complete prematurely:**

- ❌ If implementation PRs are still in review
- ❌ If acceptance criteria are partially met
- ❌ If follow-up implementation work is planned soon
- ❌ If bugs introduced by the feature are still open

## Completion Workflow

### Step 1: Verify Completion Criteria

Check the feature's `specification.md` for its Definition of Done / acceptance criteria and confirm each item is satisfied.

Typical criteria to check:

- [ ] All user-facing functionality implemented
- [ ] Unit tests written and passing
- [ ] Integration/E2E tests passing (if applicable)
- [ ] Documentation updated (user guide, commands, etc.)
- [ ] All linters pass
- [ ] All implementation issues/PRs are merged

### Step 2: Update the Feature README Status

Edit `docs/features/{feature-name}/README.md` and update the status to completed:

```markdown
## 📋 Status

**Current Phase**: Complete

**Completed**:

1. ✅ Create feature specification
2. ✅ Answer clarifying questions
3. ✅ Update specification
4. ✅ Implement feature
5. ✅ All tests and linters pass
```

### Step 3: Add to completed-features.md

Edit `docs/features/completed-features.md` and add a new row at the **top** of the table:

```markdown
| Document                                 | Completed    | Description                                |
| ---------------------------------------- | ------------ | ------------------------------------------ |
| [Feature Name](./feature-name/README.md) | MMM DD, YYYY | Brief description of what the feature does |
| [Previous entries...]                    | ...          | ...                                        |
```

**Column format:**

- **Document**: Markdown link to the feature's README
- **Completed**: Month DD, YYYY format (e.g., `Feb 18, 2026`)
- **Description**: One sentence describing what the feature does or adds (10–20 words)

**Example:**

```markdown
| [Config Validation Command](./config-validation-command/README.md) | Feb 18, 2026 | Validates environment configuration files before provisioning to catch errors early |
```

### Step 4: Remove from active-features.md

Edit `docs/features/active-features.md` and **delete the entire row** for the completed feature:

**Before:**

```markdown
| Document                                                             | Status         | Priority | Created      |
| -------------------------------------------------------------------- | -------------- | -------- | ------------ |
| [Config Validation Command](./config-validation-command/README.md)   | 🚧 In Progress | Medium   | Jan 21, 2026 |
| [Environment Status Command](./environment-status-command/README.md) | 📋 Specified   | Medium   | Dec 16, 2025 |
```

**After:**

```markdown
| Document                                                             | Status       | Priority | Created      |
| -------------------------------------------------------------------- | ------------ | -------- | ------------ |
| [Environment Status Command](./environment-status-command/README.md) | 📋 Specified | Medium   | Dec 16, 2025 |
```

### Step 5: Keep the Feature Folder

Unlike refactoring plans, **do NOT delete the feature folder or its documents**. The specification, questions, and README remain in `docs/features/{feature-name}/` as permanent reference documentation.

The completed-features.md entry links directly to the feature README so future contributors can access the full specification.

### Step 6: Commit the Changes

Stage and commit all modified files:

```bash
git add docs/features/{feature-name}/README.md
git add docs/features/active-features.md
git add docs/features/completed-features.md
git commit -m "docs: complete {feature-name} feature spec"
```

**Commit message examples:**

```bash
git commit -m "docs: complete config-validation-command feature spec"
git commit -m "docs: complete json-schema-generation feature spec"
git commit -m "docs: complete environment-status-command feature spec"
```

## Completion Entry Format

### Standard Entry Template

```markdown
| [Feature Name](./feature-name/README.md) | MMM DD, YYYY | One-sentence description of what the feature delivers |
```

### Components Explained

#### 1. Feature Name (with link)

- Markdown link pointing to the feature's README
- Display text is the human-readable feature name
- Example: `[Config Validation Command](./config-validation-command/README.md)`

#### 2. Completed Date

- Format: "Month DD, YYYY"
- Use three-letter month abbreviation
- Example: `Feb 18, 2026`

#### 3. Description

- One sentence, 10–20 words
- Describe what the feature **does** for users
- Start with a verb or noun, no period needed
- Examples:
  - `Multi-provider architecture with Hetzner Cloud as production provider`
  - `Register command to import already-provisioned instances`
  - `Validates configuration files and reports errors with fix instructions`

### Real-World Examples

```markdown
| [Hetzner Provider Support](./hetzner-provider-support/README.md) | Dec 1, 2025 | Multi-provider architecture with Hetzner Cloud as production provider |
| [Register Existing Instances](./import-existing-instances/README.md) | Nov 19, 2025 | Register command to import already-provisioned instances |
```

## Special Cases

### Partially Implemented Feature

If a feature is partially implemented and remaining work is postponed:

1. Update status in `active-features.md` to ⏸️ Deferred
2. Add a note in the feature README about what was implemented and what was deferred
3. Do **not** move to `completed-features.md` until fully done

### Cancelled Feature

If a feature is cancelled:

1. Update status in `active-features.md` to ❌ Cancelled
2. Add a note in the feature README explaining why
3. Remove from `active-features.md`
4. Optionally add to `completed-features.md` with a cancellation note in the description
5. Keep the feature folder for historical reference

**Example cancelled entry:**

```markdown
| [Linter Parallel Execution](./linter-parallel-execution/README.md) | Feb 18, 2026 | Cancelled — deferred indefinitely; parallel execution complexity outweighs benefit |
```

### Superseded Feature

If a feature is superseded by another:

1. Mark the old feature as ❌ Cancelled in `active-features.md`
2. Note the superseding feature in the README
3. Move to `completed-features.md` with a superseded note

## Verification Checklist

Before completing a feature, verify:

- [ ] All implementation issues/PRs are merged to main
- [ ] All tests pass (`cargo test`)
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] Feature README updated with ✅ Completed status
- [ ] Entry added to `completed-features.md` at the top
- [ ] Entry removed from `active-features.md`
- [ ] Feature folder and documents kept intact (not deleted)
- [ ] Changes committed with conventional commit message

## Related Documentation

- **Feature Overview**: `docs/features/README.md`
- **Active Features**: `docs/features/active-features.md`
- **Completed Features**: `docs/features/completed-features.md`
- **Creating Feature Specs**: `.github/skills/dev/planning/create-feature-spec/skill.md`
- **Committing Changes**: `.github/skills/dev/git-workflow/commit-changes/skill.md`

## Key Reminders

1. **Keep the feature folder** — do not delete it (unlike refactor plans)
2. **Link to README** in completed-features.md — the documentation is still accessible
3. **Add entry at the top** of `completed-features.md`
4. **Remove from active** — delete the row from `active-features.md`
5. **Update feature README** — mark status as Complete
6. **Use conventional commit** — `docs: complete {feature-name} feature spec`
