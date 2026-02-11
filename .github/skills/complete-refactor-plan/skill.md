---
name: complete-refactor-plan
description: Guide for completing and archiving refactoring plans in the torrust-tracker-deployer project. Covers moving entries from active to completed, updating documentation, and cleanup procedures. Use when finishing refactorings, archiving plans, or marking refactor work as done. Triggers on "complete refactor", "finish refactor plan", "archive refactor", "mark refactor done", or "refactor cleanup".
metadata:
  author: torrust
  version: "1.0"
---

# Completing Refactoring Plans

This skill guides you through the completion and cleanup process for refactoring plans in the Torrust Tracker Deployer project.

## Quick Reference

```bash
# 1. Verify all proposals are completed
# 2. Move entry from active-refactorings.md to completed-refactorings.md
# 3. Delete the plan document from docs/refactors/plans/
# 4. Commit changes
```

## When to Complete a Refactor

Complete a refactor when:

- ✅ All active proposals are implemented and merged
- ✅ All tests pass
- ✅ All linters pass
- ✅ Documentation is updated
- ✅ Changes are merged to main branch

## Completion Workflow

### Step 1: Verify Completion Criteria

Check the plan document's completion criteria:

```markdown
### Completion Criteria

- [x] All active proposals implemented
- [x] All tests passing
- [x] All linters passing
- [x] Documentation updated
- [x] Code reviewed and approved
- [x] Changes merged to main branch
```

**CRITICAL**: All checkboxes must be checked before proceeding.

### Step 2: Gather Completion Information

From the refactoring plan document, collect:

1. **Plan title** - The descriptive name of the refactoring
2. **Target area** - Brief description of what was refactored
3. **Creation date** - When the plan was created
4. **Completion date** - Today's date
5. **Key details** - Phases completed, final commit hash, related issues/PRs
6. **Notable outcomes** - Lines changed, files affected, improvements gained

### Step 3: Update completed-refactorings.md

Edit `docs/refactors/completed-refactorings.md` and add a new row at the **top** of the table:

```markdown
| Document               | Completed    | Target                   | Notes                                                                                          |
| ---------------------- | ------------ | ------------------------ | ---------------------------------------------------------------------------------------------- |
| Your Refactoring Title | MMM DD, YYYY | Brief target description | See git history at `docs/refactors/plans/your-file.md` - Key details (phases, commits, issues) |
| [Previous entries...]  | ...          | ...                      | ...                                                                                            |
```

**Format Guidelines:**

- **Document**: Plain text title (no link needed - file will be deleted)
- **Completed**: Month DD, YYYY format (e.g., "Feb 11, 2026")
- **Target**: Brief description of what was refactored (1-10 words)
- **Notes**: Comprehensive summary including:
  - Git history reference: `See git history at \`docs/refactors/plans/{filename}.md\``
  - Number of phases completed
  - Final commit hash (8 characters)
  - Related issue or PR numbers
  - Key metrics (files changed, lines changed, improvements)

**Example Entry:**

```markdown
| Extract Template Rendering Services | Feb 10, 2026 | Eliminate rendering duplication | See git history at `docs/refactors/plans/extract-template-rendering-services.md` - Extracted template rendering logic into shared application-layer services for all 8 template types, eliminating duplication between render command and Steps (5 phases completed, final commit 463e7933) |
```

### Step 4: Remove from active-refactorings.md

Edit `docs/refactors/active-refactorings.md` and **delete the entire row** for the completed refactoring:

**Before:**

```markdown
| Document                               | Status         | Issue | Target             | Created    |
| -------------------------------------- | -------------- | ----- | ------------------ | ---------- |
| [Your Refactoring](plans/your-file.md) | 🚧 In Progress | TBD   | Target description | 2026-02-01 |
| [Other active refactoring](plans/...)  | 📋 Planning    | TBD   | ...                | ...        |
```

**After:**

```markdown
| Document                              | Status      | Issue | Target | Created |
| ------------------------------------- | ----------- | ----- | ------ | ------- |
| [Other active refactoring](plans/...) | 📋 Planning | TBD   | ...    | ...     |
```

### Step 5: Delete the Plan Document

Remove the detailed plan document from the repository:

```bash
git rm docs/refactors/plans/your-refactoring-file.md
```

**Why delete the plan?**

- The work is now in git history (commits, PRs)
- Completed plans are indexed in `completed-refactorings.md`
- Historical plan can be recovered from git history if needed
- Keeps the `plans/` directory focused on active work

**Exception**: If the plan has ongoing reference value or contains unique insights not captured elsewhere, you may choose to keep it, but this is rare.

### Step 6: Commit the Changes

Stage and commit all changes:

```bash
# Stage the updated index files
git add docs/refactors/active-refactorings.md docs/refactors/completed-refactorings.md

# Commit with conventional format
git commit -m "docs: complete [refactoring-name] refactor plan"
```

**Example commit messages:**

```bash
git commit -m "docs: complete extract-template-rendering-services refactor plan"
git commit -m "docs: complete simplify-error-handling refactor plan"
git commit -m "docs: complete ssh-client-improvements refactor plan"
```

## Completion Entry Format

### Standard Entry Template

```markdown
| [Refactoring Title] | [Month DD, YYYY] | [Brief Target] | See git history at `docs/refactors/plans/[filename].md` - [Key accomplishments, phases, commits, issues] |
```

### Components Explained

#### 1. Refactoring Title

- Plain text (no markdown link)
- Same as the H1 heading in the plan document
- Example: "Extract Template Rendering Services"

#### 2. Completed Date

- Format: "Month DD, YYYY"
- Use three-letter month abbreviation
- Example: "Feb 11, 2026"

#### 3. Brief Target

- 1-10 words describing what was refactored
- Action-oriented when possible
- Examples:
  - "Eliminate rendering duplication"
  - "Split 562-line file into focused submodule"
  - "Move topology rules to domain"

#### 4. Notes

- Always start with: `See git history at \`docs/refactors/plans/{filename}.md\``
- Include:
  - Summary of what was accomplished
  - Number of phases completed (e.g., "5 phases completed")
  - Final commit hash (8 characters, e.g., "final commit 463e7933")
  - Related issue or PR numbers (e.g., "Issue #243" or "PR #319")
  - Key metrics if significant (files affected, lines changed)
  - Special notes (e.g., "superseded by...", "4 postponed", "Epic #287")

### Real-World Examples

**Simple Refactoring:**

```markdown
| Rename Test Functions to Follow Conventions | Dec 12, 2025 | All test functions with `test_` prefix (21 files) | See git history at `docs/refactors/plans/rename-test-functions-to-follow-conventions.md` - Renamed 93 test functions and 14 helper functions across 20 files to follow behavior-driven naming conventions (21 proposals, all completed), PR #228 |
```

**Complex Refactoring with Epic:**

```markdown
| Docker Compose Topology Domain Model | Jan 26, 2026 | Move topology rules to domain, derive volumes/networks | See git history at `docs/refactors/plans/docker-compose-topology-domain-model.md` - Moved all Docker Compose topology decisions to domain layer with Network/Service enums and DockerComposeTopology aggregate (Epic #287, 8 proposals) |
```

**Refactoring with Postponed Work:**

```markdown
| Command Code Quality Improvements | Dec 3, 2025 | `ProvisionCommand`, `ConfigureCommand` | See git history at `docs/refactors/plans/command-code-quality-improvements.md` - API simplification, state persistence, clock injection, trace writing, and test builders (5 of 9 proposals completed, 4 postponed for future work) |
```

**Superseded Refactoring:**

```markdown
| User Output Architecture Improvements | Nov 13, 2025 | `src/presentation/views/` (formerly `user_output`) | See git history at `docs/refactors/plans/user-output-architecture-improvements.md` - Superseded by Presentation Layer Reorganization which renamed user_output to views and integrated it into four-layer MVC architecture |
```

## Best Practices

### Documentation Quality

✅ **Do:**

- Provide complete git history reference
- Include all relevant issue/PR numbers
- Document key metrics and outcomes
- Note if work was postponed or superseded
- Use consistent date format (MMM DD, YYYY)
- Add entry at the **top** of the completed table

❌ **Don't:**

- Link to deleted files
- Skip the git history reference
- Use vague descriptions
- Forget to remove from active-refactorings.md
- Delete plan without updating indexes

### Timing

**Complete the refactor when:**

- All code is merged to main
- Documentation is updated
- Tests and linters pass
- No pending PRs related to the refactor

**Don't complete prematurely:**

- If proposals are still in review
- If documentation updates are pending
- If related issues are still open
- If follow-up work is planned soon

## Special Cases

### Paused Refactoring

If a refactoring is paused, update the status in `active-refactorings.md`:

```markdown
| Document                          | Status    | Issue | Target | Created    |
| --------------------------------- | --------- | ----- | ------ | ---------- |
| [Your Refactoring](plans/file.md) | ⏸️ Paused | TBD   | ...    | 2026-02-01 |
```

Don't move to completed until actually finished.

### Cancelled Refactoring

If a refactoring is cancelled:

1. Update status in `active-refactorings.md` to ❌ Cancelled
2. Add note explaining why (superseded, no longer needed, etc.)
3. Delete the plan document
4. Optionally move to completed-refactorings.md with cancellation note

### Partial Completion

If only some proposals were completed:

1. Document which proposals were completed
2. Note which were postponed or discarded
3. Consider if remaining work should be a new separate plan
4. Example: "5 of 9 proposals completed, 4 postponed for future work"

## Verification Checklist

Before completing a refactor, verify:

- [ ] All active proposals are implemented and merged
- [ ] All tests pass (`cargo test`)
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] E2E tests pass (if applicable)
- [ ] Documentation is updated
- [ ] Related issues/PRs are closed or merged
- [ ] Entry added to `completed-refactorings.md` at the top
- [ ] Entry removed from `active-refactorings.md`
- [ ] Plan document deleted (or archived if keeping)
- [ ] Changes committed with conventional commit message

## Related Documentation

- **Refactoring Overview**: `docs/refactors/README.md`
- **Active Refactorings**: `docs/refactors/active-refactorings.md`
- **Completed Refactorings**: `docs/refactors/completed-refactorings.md`
- **Creating Refactor Plans**: `.github/skills/create-refactor-plan/skill.md`
- **Committing Changes**: `.github/skills/commit-changes/skill.md`

## Key Reminders

1. **Verify all completion criteria** before proceeding
2. **Add entry at the top** of completed-refactorings.md
3. **Include git history reference** in notes
4. **Remove from active-refactorings.md** completely
5. **Delete the plan document** from plans/
6. **Use conventional commit** message for changes
7. **Document key outcomes** (phases, commits, issues, metrics)
