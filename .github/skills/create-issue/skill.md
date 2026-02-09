---
name: create-issue
description: Guide for creating GitHub issues in the torrust-tracker-deployer project. Covers the full workflow from specification drafting, user review, to GitHub issue creation with proper documentation, linking, and file naming. Supports task, bug, feature, and epic issue types. Use when creating issues, opening tickets, filing bugs, proposing tasks, or adding features. Triggers on "create issue", "open issue", "new issue", "file bug", "add task", "create epic", or "open ticket".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Issues

This skill guides you through the complete workflow for creating GitHub issues in the Torrust Tracker Deployer project.

## Issue Types

| Type        | Label     | When to Use                                  |
| ----------- | --------- | -------------------------------------------- |
| **Task**    | `task`    | Single implementable unit of work            |
| **Bug**     | `bug`     | Something broken that needs fixing           |
| **Feature** | `feature` | New capability or enhancement                |
| **Epic**    | `epic`    | Major feature area containing multiple tasks |

## Issue Hierarchy

```text
Roadmap (Issue #1)
└── Epic (e.g., Issue #2)
    ├── Task (e.g., Issue #3)
    ├── Bug (e.g., Issue #4)
    └── Feature (e.g., Issue #5)
```

Not all issues need a parent epic. Standalone tasks, bugs, and features are valid.

## Workflow Overview

The process is **spec-first**: the specification document is written and reviewed before any GitHub issue is created.

1. **Draft specification** document in `docs/issues/` (using template)
2. **User reviews** the draft specification
3. **Create GitHub issue** (epic first if needed, then task/bug/feature)
4. **Update specification** with issue number and rename file
5. **Update links** bidirectionally (epic task list, roadmap)
6. **Pre-commit checks** and commit specification + links

**CRITICAL**: Never create the GitHub issue before the specification is reviewed and approved by the user. The spec draft is the foundation — the GitHub issue references it.

## Step-by-Step Process

### Step 1: Draft Issue Specification

Create a specification document from the template with a **temporary name** (no issue number yet):

```bash
cp docs/issues/SPECIFICATION-TEMPLATE.md docs/issues/{short-description}.md
```

Fill in the specification. Key sections:

- **Overview**: Clear description of what needs to be done
- **Goals**: High-level objectives with checkboxes
- **Architecture Requirements**: DDD layer, module path, patterns
- **Specifications**: Detailed technical specs with code examples
- **Implementation Plan**: Phased breakdown with actionable subtasks
- **Acceptance Criteria**: Definition of done (always include `./scripts/pre-commit.sh`)
- **Related Documentation**: Links to relevant docs and ADRs

Leave the **Issue** and **Parent Epic** fields as placeholders — they will be filled after the GitHub issue is created.

For simple bugs or small tasks, the specification can be minimal but should still be created for user review.

### Step 2: User Reviews the Draft

**STOP HERE** and present the draft specification to the user for review.

The user may:

- Approve it as-is
- Request changes to scope, approach, or acceptance criteria
- Add implementation details or constraints
- Restructure the phases

Iterate on the draft until the user approves it.

### Step 3: Create GitHub Epic Issue (If Needed)

If this is the first task in a new area, create an epic issue first:

1. Use the template at `docs/issues/EPIC-TEMPLATE.md`
2. Add labels: `epic` + relevant labels (add `roadmap` if it's a planned item)
3. Set parent issue if applicable
4. Note the epic issue number

### Step 4: Create GitHub Task/Bug/Feature Issue

Use the GitHub issue template at `docs/issues/GITHUB-ISSUE-TEMPLATE.md` as the body structure.

**Required fields**:

- **Title**: Clear, descriptive title
- **Labels**: Issue type (`task`, `bug`, `feature`) + relevant technical labels (`rust`, `cli`, `e2e`, `testing`, etc.)
- **Body**: Overview, specification link (temporary path), implementation plan, acceptance criteria

**Optional fields**:

- **Parent issue**: Link to parent epic if applicable
- **Roadmap label**: Add `roadmap` label if this is a planned roadmap item

**Acceptance criteria must always include**:

```markdown
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
```

Note the issue number (e.g., #42).

### Step 5: Update Specification with Issue Number

Update the specification document header:

```markdown
# [Task Title]

**Issue**: #42
**Parent Epic**: #X - [Epic Name]
**Related**: [Links to related issues]
```

### Step 6: Rename Specification File

Add the issue number prefix:

```bash
# Task/Bug/Feature:
mv docs/issues/{description}.md docs/issues/{number}-{description}.md

# Epic:
mv docs/issues/{description}.md docs/issues/{number}-epic-{description}.md
```

### Step 7: Update GitHub Issue with Correct Link

Update the specification link in the GitHub issue body to point to the renamed file:

```markdown
## Specification

See detailed specification: [docs/issues/42-short-description.md](../docs/issues/42-short-description.md)
```

### Step 8: Update Epic and Roadmap (If Applicable)

- **Epic task list**: Add `- [ ] #42 - Task description` to the parent epic
- **Roadmap**: Update `docs/roadmap.md` with issue link if this is a roadmap item

### Step 9: Pre-Commit and Commit

```bash
./scripts/pre-commit.sh
git add docs/issues/ docs/roadmap.md  # only files that changed
git commit -m "docs: add issue specification for #{number}"
git push
```

## Simplified Flow for Small Issues

For trivial bugs or tiny tasks where a full specification is unnecessary:

1. Draft a brief description and present it to the user for review
2. After user approval, create the GitHub issue directly
3. No specification file or repository commit needed

## GitHub Issue Body Template (Quick Reference)

```markdown
## Overview

[Brief description]

## Specification

See detailed specification: [docs/issues/{number}-{name}.md](../docs/issues/{number}-{name}.md)

## Implementation Plan

### Phase 1: [Phase Name]

- [ ] Task 1.1
- [ ] Task 1.2

## Acceptance Criteria

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] [Task-specific criterion]

## Related

- Parent: #X (Epic: [Epic Name])
- Specification: docs/issues/{number}-{name}.md
```

## Common Labels

| Label     | Usage                        |
| --------- | ---------------------------- |
| `task`    | Implementation work          |
| `bug`     | Something is broken          |
| `feature` | New capability               |
| `epic`    | Contains multiple sub-issues |
| `roadmap` | Part of project roadmap      |
| `rust`    | Rust code changes            |
| `cli`     | CLI interface changes        |
| `testing` | Test infrastructure changes  |
| `e2e`     | End-to-end test related      |
| `docs`    | Documentation changes        |
| `ansible` | Ansible playbook changes     |

## References

- Detailed roadmap issue guide: `docs/contributing/roadmap-issues.md`
- Specification template: `docs/issues/SPECIFICATION-TEMPLATE.md`
- GitHub issue template: `docs/issues/GITHUB-ISSUE-TEMPLATE.md`
- Epic template: `docs/issues/EPIC-TEMPLATE.md`
- Commit conventions: `docs/contributing/commit-process.md`
