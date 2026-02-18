---
name: create-feature-spec
description: Guide for creating feature specifications in the torrust-tracker-deployer project. Covers folder structure, document templates, questions for stakeholders, and workflow for registering new features. Use when specifying new features, documenting feature requirements, or adding entries to the active features list. Triggers on "create feature spec", "new feature", "feature specification", "specify feature", or "feature document".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Feature Specifications

This skill guides you through creating feature specifications for the Torrust Tracker Deployer project.

## Quick Reference

```bash
# 1. Create feature folder and copy templates
mkdir docs/features/{feature-name}
cp docs/features/TEMPLATE-README.md docs/features/{feature-name}/README.md
cp docs/features/TEMPLATE-QUESTIONS.md docs/features/{feature-name}/questions.md
cp docs/features/TEMPLATE-SPECIFICATION.md docs/features/{feature-name}/specification.md

# 2. Fill in the documents
# 3. Add entry to active-features.md with status 📋 Specified
# 4. Run linters and commit
```

## What Is a Feature Specification?

A feature specification is a set of documents that define a new user-facing capability **before** implementation begins. It captures:

- **What** needs to be built and why
- **How** it should behave from the user's perspective
- **Open questions** that need answers before work starts
- **Scope boundaries** — what is in and out
- **Acceptance criteria** — how to know it's done

Feature specifications are **not implementation plans**. They focus on requirements and design intent. Implementation details (issues, branches, PRs) are separate.

> **Note**: Unlike refactoring plans, feature specifications are **kept in the repository permanently** after completion. They serve as a historical record and reference for future work.

## When to Create a Feature Specification

**Create a spec when:**

- ✅ Adding new user-facing capabilities
- ✅ Implementing significant new functionality
- ✅ Building features that span multiple components
- ✅ Work requires stakeholder alignment before starting
- ✅ Design decisions need documentation
- ✅ Implementation will take multiple sessions or issues

**Skip a formal spec for:**

- ❌ Simple bug fixes
- ❌ Internal code improvements (use refactoring docs instead)
- ❌ Trivial enhancements
- ❌ Emergency hotfixes

## Document Structure

Each feature lives in its own folder under `docs/features/{feature-name}/` and contains three core documents:

```text
docs/features/{feature-name}/
├── README.md          # Overview, status tracking, quick reference
├── questions.md       # Clarifying questions for stakeholders
└── specification.md   # Detailed technical specification
```

### README.md

High-level overview with:

- Brief description and problem statement
- Current status and phase tracking
- Quick summary of goals and approach
- Links to related documentation

### questions.md

Pre-implementation questions covering:

- **Scope** — what's in and out
- **Requirements** — must-haves vs. nice-to-haves
- **Technical approach** — patterns and constraints
- **Priority and timeline** — urgency and dependencies
- **Success criteria** — how to know it's done

### specification.md

Technical specification with:

- Detailed problem statement
- Proposed solution and design
- Implementation plan and phases
- Acceptance criteria and test strategy
- Risk assessment

## Creating a New Feature Specification

### Step 1: Choose a Feature Name

Use lowercase with hyphens, descriptive of what the feature does:

- ✅ Good: `config-validation-command`, `environment-status-command`, `json-schema-generation`
- ❌ Bad: `new-feature`, `improvements`, `feature2`

### Step 2: Create the Folder and Copy Templates

```bash
mkdir docs/features/{feature-name}
cp docs/features/TEMPLATE-README.md docs/features/{feature-name}/README.md
cp docs/features/TEMPLATE-QUESTIONS.md docs/features/{feature-name}/questions.md
cp docs/features/TEMPLATE-SPECIFICATION.md docs/features/{feature-name}/specification.md
```

### Step 3: Fill in README.md

Replace all `{Feature Name}` placeholders and fill in:

```markdown
# {Feature Name}

Brief description of the feature — one or two sentences.

## 📋 Status

**Current Phase**: Planning

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ⏳ Answer clarifying questions
   ...

## 🎯 Quick Summary

- **Problem**: What issue are we solving?
- **Solution**: How are we solving it?
- **Status**: Where are we in the implementation?
```

### Step 4: Fill in questions.md

Write the clarifying questions relevant to this feature. Use the template sections as a starting point and add feature-specific questions. Leave answers as `[To be filled by product owner]` — stakeholders will answer them.

### Step 5: Draft specification.md

Write the initial specification based on what is known. Mark uncertain areas clearly. The specification will be refined once questions are answered.

**Key sections to complete:**

- **Overview** — feature goals and scope
- **Problem Statement** — what problem this solves
- **Proposed Solution** — how it works at a high level
- **Out of Scope** — explicit exclusions
- **Definition of Done** — acceptance criteria

### Step 6: Add to active-features.md

Edit `docs/features/active-features.md` and add a new row to the table:

```markdown
| [Feature Name](./feature-name/README.md) | 📋 Specified | High/Medium/Low | MMM DD, YYYY |
```

**Column format:**

- **Document**: Markdown link to the feature's README, display text is the feature name
- **Status**: Use the appropriate status emoji (see Status Legend below)
- **Priority**: `High`, `Medium`, or `Low`
- **Created**: Month DD, YYYY (e.g., `Feb 18, 2026`)

**Example:**

```markdown
| [Config Validation Command](./config-validation-command/README.md) | 📋 Specified | Medium | Jan 21, 2026 |
```

### Step 7: Run Linters

```bash
cargo run --bin linter all
```

Fix any issues found:

- **Markdown formatting** — follow markdownlint rules
- **Spelling** — add project-specific terms to `project-words.txt`

### Step 8: Commit the Specification

```bash
git add docs/features/{feature-name}/ docs/features/active-features.md
git commit -m "docs: add feature spec for {short-description}"
```

**Commit message examples:**

- `docs: add feature spec for config validation command`
- `docs: add feature spec for json schema generation`

## Status Legend

- 📋 **Specified** — Requirements documented, awaiting implementation
- 🚧 **In Progress** — Implementation has started
- ✅ **Completed** — Feature fully implemented and merged
- ⏸️ **Deferred** — Work postponed for future consideration
- 🔄 **Refactoring** — Being redesigned or improved
- ❌ **Cancelled** — Feature abandoned or superseded

## Status Lifecycle

```text
📋 Specified → 🚧 In Progress → ✅ Completed
                    ↓
               ⏸️ Deferred
```

Update the status in `active-features.md` as work progresses:

1. **📋 Specified** — Specification written, questions pending or answered
2. **🚧 In Progress** — Implementation issues have been created and work has started
3. **✅ Completed** — All implementation is merged; move entry to `completed-features.md`

## Best Practices

### Specification Quality

Good feature specifications:

- ✅ **Clear problem statement** — Explain why this matters
- ✅ **Defined scope** — What's in and what's out
- ✅ **User-focused goals** — How does this help users?
- ✅ **Measurable outcomes** — How do we know we're done?
- ✅ **Risk assessment** — What could go wrong?
- ✅ **Realistic scope** — Don't over-specify or under-specify

### Questions First

Use `questions.md` to surface ambiguities early:

- Write questions **before** completing the specification
- Have stakeholders answer in the document
- Refine the specification based on answers
- Don't start implementation until key questions are answered

### Keep It Separate From Implementation

The specification is about **what** and **why**, not **how it will be coded**:

- ❌ Don't include specific Rust types or file paths in the spec
- ❌ Don't assign GitHub issues inside the spec
- ✅ Do describe user-facing behavior
- ✅ Do describe integration points at a conceptual level

## Common Pitfalls

❌ **Don't:**

- Skip the questions document — ambiguities cause rework
- Start implementing before the spec is reviewed
- Mix features and refactorings in the same document
- Use vague problem statements ("improve the UX")
- Forget to update `active-features.md`

✅ **Do:**

- Write the problem statement before the solution
- Get stakeholder input on scope before detailing implementation
- Keep the spec updated as the feature evolves
- Reference related ADRs and documentation
- Include clear acceptance criteria

## Workflow Summary

1. **Create** — Copy templates to `docs/features/{feature-name}/`
2. **Draft** — Fill in README, questions, and initial specification
3. **Register** — Add entry to `active-features.md` with status 📋 Specified
4. **Lint** — Run `cargo run --bin linter all` and fix any issues
5. **Commit** — Commit documents and index update
6. **Review** — Stakeholders answer questions in `questions.md`
7. **Refine** — Update specification based on answers
8. **Implement** — Create issues; update status to 🚧 In Progress
9. **Complete** — See `complete-feature-spec` skill when implementation is done

## Related Documentation

- **Feature Overview**: `docs/features/README.md`
- **Active Features**: `docs/features/active-features.md`
- **Completed Features**: `docs/features/completed-features.md`
- **Templates**: `docs/features/TEMPLATE-README.md`, `TEMPLATE-QUESTIONS.md`, `TEMPLATE-SPECIFICATION.md`
- **Development Principles**: `docs/development-principles.md`
- **Contributing Guidelines**: `docs/contributing/README.md`
- **Completing Features**: `.github/skills/complete-feature-spec/skill.md`
