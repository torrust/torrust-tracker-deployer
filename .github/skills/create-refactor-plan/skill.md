---
name: create-refactor-plan
description: Guide for creating refactoring plans in the torrust-tracker-deployer project. Covers plan structure, proposal organization by impact/effort, progress tracking, and workflow. Use when planning refactorings, documenting improvements, or organizing technical debt work. Triggers on "create refactor", "new refactor plan", "refactoring plan", "plan refactor", or "refactor document".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Refactoring Plans

This skill guides you through creating comprehensive refactoring plans for the Torrust Tracker Deployer project.

## Quick Reference

```bash
# 1. Copy template
cp docs/refactors/TEMPLATE.md docs/refactors/plans/{short-descriptive-name}.md

# 2. Fill in plan details
# 3. Add entry to active-refactorings.md with status 📋 Planning
# 4. Get team approval before implementation
```

## When to Create a Refactoring Plan

**Create a plan when:**

- ✅ Changes affect multiple functions or modules
- ✅ Multiple improvements should be coordinated
- ✅ Work will span multiple sessions or PRs
- ✅ Team alignment is needed before starting
- ✅ Changes require careful sequencing

**Skip a formal plan for:**

- ❌ Single-function improvements
- ❌ Obvious bug fixes
- ❌ Trivial style changes
- ❌ Urgent hotfixes

## Plan Structure

Each refactoring plan follows this structure:

1. **Overview** - Summary of goals, scope, and target files
2. **Progress Tracking** - Status metrics and phase summary
3. **Key Problems Identified** - Categorized list of issues
4. **Refactoring Phases** - Organized proposals by priority
5. **Timeline** - Start date and completion tracking
6. **Review Process** - Approval and completion criteria

## Creating a New Plan

### Step 1: Copy the Template

```bash
cp docs/refactors/TEMPLATE.md docs/refactors/plans/{short-descriptive-name}.md
```

**Naming Convention**: Use lowercase with hyphens, descriptive of the refactoring area

- ✅ Good: `simplify-error-handling.md`, `extract-ssh-client-config.md`
- ❌ Bad: `refactor.md`, `improvements.md`, `fix-stuff.md`

### Step 2: Fill in Overview Section

```markdown
# [Descriptive Refactoring Title]

## 📋 Overview

Brief summary of what this refactoring addresses - goals, scope, and expected impact.

**Target Files:**

- `path/to/file1.rs`
- `path/to/file2.rs`

**Scope:**

- [Specific area or component]
- [What will change]
- [What will NOT change]
```

### Step 3: Identify Key Problems

Document the problems you're solving:

```markdown
## 🎯 Key Problems Identified

### 1. [Problem Category]

Clear description of the issue with code examples if helpful.

### 2. [Problem Category]

Another issue to address.
```

### Step 4: Organize Proposals by Impact/Effort

**Priority Matrix:**

```text
         High Impact
             ↑
   P0 (Quick Wins) | P1 (Major Projects)
   Low Effort      | High Effort
   ─────────────────┼─────────────────
   P2 (Fill-ins)   | P3 (Hard Slogs)
   Low Impact      | High Impact
             ↓
         Low Effort → High Effort
```

**Impact Levels:**

- 🟢🟢🟢 High - Significantly improves code quality, maintainability, or performance
- 🟢🟢 Medium - Noticeable improvement in specific area
- 🟢 Low - Minor improvement or cleanup

**Effort Levels:**

- 🔵 Low - < 1 hour
- 🔵🔵 Medium - 1-4 hours
- 🔵🔵🔵 High - > 4 hours

### Step 5: Structure Proposals

```markdown
## Phase 0: [Phase Name] (Highest Priority)

Description of why this phase should be done first.

### Proposal #0: [Clear, Action-Oriented Title]

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None
**Completed**: -
**Commit**: -

#### Problem

Clear description with code example showing the issue.

#### Proposed Solution

Detailed solution with improved code example.

#### Rationale

Why this solution was chosen, alternatives considered.

#### Benefits

- ✅ [Specific benefit 1]
- ✅ [Specific benefit 2]

#### Implementation Checklist

- [ ] [Concrete step 1]
- [ ] [Concrete step 2]
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Update documentation

#### Testing Strategy

How to verify these changes work correctly.
```

### Step 6: Add Progress Tracking

Update the progress section with accurate counts:

```markdown
## 📊 Progress Tracking

**Total Active Proposals**: 5
**Total Postponed**: 1
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 5

### Phase Summary

- **Phase 0 - Quick Wins (High Impact, Low Effort)**: ⏳ 0/3 completed (0%)
- **Phase 1 - Major Improvements (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
```

### Step 7: Set Timeline and Review Criteria

```markdown
## 📈 Timeline

- **Start Date**: [Today's date]
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [ ] Technical feasibility validated
- [ ] Aligns with Development Principles
- [ ] Implementation plan is clear and actionable
- [ ] Priorities are correct (high-impact/low-effort first)

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Changes merged to main branch
```

### Step 8: Add to Active Refactorings Index

Edit `docs/refactors/active-refactorings.md`:

```markdown
| Document                                          | Status      | Issue | Target            | Created    |
| ------------------------------------------------- | ----------- | ----- | ----------------- | ---------- |
| [Your Refactoring Title](plans/your-file-name.md) | 📋 Planning | TBD   | Brief target area | YYYY-MM-DD |
```

## Status Legend

- 📋 **Planning** - Document created, awaiting review and approval
- 🚧 **In Progress** - Implementation has started
- ✅ **Completed** - All proposals implemented and merged
- ⏸️ **Paused** - Work temporarily suspended
- ❌ **Cancelled** - Plan was abandoned or superseded

## Best Practices

### Plan Quality

Good refactoring plans:

- ✅ **Prioritize by impact/effort** - Quick wins (P0) first
- ✅ **Include code examples** - Show before/after
- ✅ **Provide checklists** - Track implementation steps
- ✅ **Document rationale** - Explain why, not just what
- ✅ **Set realistic timelines** - Based on team capacity
- ✅ **Align with principles** - Support project goals

### Proposal Organization

**Phase 0 - Quick Wins (P0)**: High impact, low effort

- These should be done first for maximum value

**Phase 1 - Major Projects (P1)**: High impact, high effort

- Complex improvements requiring significant work

**Phase 2 - Fill-ins (P2)**: Low impact, low effort

- Nice-to-have improvements if time permits

**Phase 3 - Hard Slogs (P3)**: Low impact, high effort

- Usually postponed or discarded

### Common Pitfalls

❌ **Don't:**

- Start implementation before plan approval
- Mix unrelated improvements in one plan
- Skip code examples in problem descriptions
- Forget to update progress tracking
- Ignore dependencies between proposals

✅ **Do:**

- Get team review before starting work
- Keep proposals focused and atomic
- Show clear before/after examples
- Update progress after each completion
- Document proposal dependencies clearly

## Workflow Summary

1. **Create** - Copy template, fill in details, organize by impact/effort
2. **Register** - Add entry to `active-refactorings.md` with status 📋 Planning
3. **Review** - Get team approval on the plan
4. **Update Status** - Change to 🚧 In Progress when starting
5. **Implement** - Follow proposals in priority order
6. **Track Progress** - Update plan after each proposal
7. **Complete** - See "complete-refactor-plan" skill for cleanup steps

## Example Proposals

### Good Proposal (Clear and Actionable)

````markdown
### Proposal #1: Extract Magic Numbers into Config Constants

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P0

#### Problem

Hard-coded timeout values scattered across SSH client:

```rust
sleep(Duration::from_secs(5)).await;
// ... later ...
timeout(Duration::from_secs(30), operation);
```
````

#### Proposed Solution

Extract into `SshConnectionConfig`:

```rust
pub struct SshConnectionConfig {
    pub connection_timeout: Duration,
    pub retry_delay: Duration,
}

// Usage:
sleep(config.retry_delay).await;
timeout(config.connection_timeout, operation);
```

#### Benefits

- ✅ Centralized configuration
- ✅ Easier to adjust for different environments
- ✅ Testable with custom timeouts

````markdown
### Poor Proposal (Vague and Unclear)

```markdown
### Proposal #X: Improve Error Handling

**Problem**: Errors are bad
**Solution**: Make them better
```
````

## Related Documentation

- **Refactoring Overview**: `docs/refactors/README.md`
- **Template**: `docs/refactors/TEMPLATE.md`
- **Active Refactorings**: `docs/refactors/active-refactorings.md`
- **Completed Refactorings**: `docs/refactors/completed-refactorings.md`
- **Development Principles**: `docs/development-principles.md`
- **Contributing Guide**: `docs/contributing/README.md`

## Troubleshooting

### Plan Too Large

**Problem**: Refactoring plan has 15+ proposals

**Solution**: Split into multiple focused plans, each targeting a specific area

### Unclear Priorities

**Problem**: Not sure which proposals are P0 vs P1

**Solution**: Use the impact/effort matrix - P0 should be high-impact + low-effort

### No Code Examples

**Problem**: Proposals lack concrete before/after code

**Solution**: Add at least one code snippet showing current problem and proposed solution

### Missing Dependencies

**Problem**: Some proposals depend on others but aren't marked

**Solution**: Add "Depends On: Proposal #X" field to proposals with dependencies

## Key Reminders

1. **Always use the template** - Ensures consistency and completeness
2. **Prioritize by impact/effort** - P0 (quick wins) first
3. **Include code examples** - Makes problems and solutions concrete
4. **Get approval before implementing** - Plan must be reviewed
5. **Update progress regularly** - Keep metrics current
6. **Add to active-refactorings.md** - Register the plan in the index
