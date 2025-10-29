# Refactoring

This directory contains detailed refactoring plans for improving the codebase. Each document outlines specific improvements, implementation strategies, and progress tracking.

## 📋 Purpose

Refactoring plans serve to:

- **Document planned improvements** before implementation
- **Track progress** on ongoing refactoring work
- **Provide context** for reviewers and future maintainers
- **Align team** on technical improvements
- **Preserve decisions** and rationale

## 📁 Active Refactoring Plans

See [active-refactorings.md](./active-refactorings.md) for the current list of ongoing refactoring work.

## ✅ Completed Refactorings

See [completed-refactorings.md](./completed-refactorings.md) for the complete history of finished refactoring work.

## 📂 Directory Structure

```text
docs/refactors/
├── README.md                    # This file - refactoring process documentation
├── TEMPLATE.md                  # Template for creating new refactoring plans
├── active-refactorings.md       # Index of ongoing refactoring work
├── completed-refactorings.md    # Historical record of completed refactorings
└── plans/                       # Directory containing detailed refactoring plan documents
    └── command-code-quality-improvements.md
```

**Meta-Documentation (at root):**

- `README.md` - Explains the refactoring process and conventions
- `TEMPLATE.md` - Template for creating new refactoring plan documents
- `active-refactorings.md` - Index of current ongoing refactoring work
- `completed-refactorings.md` - Historical record of finished refactorings

**Refactoring Plan Documents (`plans/` subdirectory):**

- Each active refactoring has a detailed plan document in `plans/` (e.g., `plans/command-code-quality-improvements.md`)
- These contain the full implementation details, progress tracking, and technical specifications
- When refactoring is complete, the plan document is typically deleted or archived

**File Management Process:**

1. **New refactoring**: Create plan document in `plans/`, add entry to `active-refactorings.md` with status 📋 Planning
2. **Start work**: Update status to 🚧 In Progress in `active-refactorings.md`
3. **Complete work**: Move entry from `active-refactorings.md` to `completed-refactorings.md`
4. **Cleanup**: Delete the detailed plan document from `plans/` (work is now in git history)

## 🎯 Plan Structure

Each refactoring plan follows this structure:

1. **Overview**: Summary of goals and scope
2. **Progress Tracking**: Current status and completion metrics
3. **Phased Proposals**: Organized by impact and effort
4. **Implementation Details**: Code examples and checklists
5. **Timeline**: Estimated duration and sprint planning
6. **Review Process**: Approval and completion criteria

## 📊 Status Legend

- 📋 **Planning** - Document created, awaiting review and approval
- 🚧 **In Progress** - Implementation has started
- ✅ **Completed** - All proposals implemented and merged
- ⏸️ **Paused** - Work temporarily suspended
- ❌ **Cancelled** - Plan was abandoned or superseded

## 🔄 Workflow

### 1. Creation

1. Identify area needing refactoring
2. Create detailed plan document in this directory
3. Organize proposals by impact/effort ratio
4. Add implementation checklists and timeline
5. **Add entry to [active-refactorings.md](./active-refactorings.md) with status 📋 Planning**

### 2. Review and Approval

1. Team reviews plan for technical feasibility
2. Validate alignment with project principles
3. Approve or request modifications
4. Set implementation timeline

### 3. Implementation

1. **Update status to 🚧 In Progress in [active-refactorings.md](./active-refactorings.md)**
2. Create tracking issue (optional)
3. Create feature branch
4. Implement proposals in priority order
5. Update progress in plan document
6. Run tests and linters after each change

### 4. Completion

1. Final verification of all changes
2. Update plan document status to ✅ Completed
3. **Move refactoring entry from [active-refactorings.md](./active-refactorings.md) to [completed-refactorings.md](./completed-refactorings.md)**
4. Delete the refactoring plan document (or archive if needed for reference)
5. Create pull request
6. Merge after review approval

## 🎓 Best Practices

### When to Create a Refactoring Plan

Create a plan when:

- ✅ Changes affect multiple functions or modules
- ✅ Multiple improvements should be coordinated
- ✅ Work will span multiple sessions or PRs
- ✅ Team alignment is needed before starting
- ✅ Changes require careful sequencing

Skip a formal plan for:

- ❌ Single-function improvements
- ❌ Obvious bug fixes
- ❌ Trivial style changes
- ❌ Urgent hotfixes

### Plan Quality

Good refactoring plans:

- ✅ **Prioritize by impact/effort** - Quick wins first
- ✅ **Include code examples** - Show before/after
- ✅ **Provide checklists** - Track implementation steps
- ✅ **Document rationale** - Explain why, not just what
- ✅ **Set realistic timelines** - Based on team capacity
- ✅ **Align with principles** - Support project goals

### Progress Tracking

Update progress regularly:

- **After each proposal**: Mark as completed
- **Weekly**: Update summary metrics
- **Phase completion**: Update phase status
- **Issues/blockers**: Document in plan

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Core principles guiding refactoring
- [Contributing Guidelines](../contributing/README.md) - General contribution process
- [Error Handling Guide](../contributing/error-handling.md) - Error handling standards
- [Testing Conventions](../contributing/testing/) - Testing best practices

## 💡 Tips

### For Plan Authors

- **Start with high-level goals** before diving into details
- **Group related changes** into logical phases
- **Estimate conservatively** - refactoring often takes longer than expected
- **Get early feedback** on the plan structure
- **Keep it updated** as implementation progresses

### For Implementers

- **Read the entire plan** before starting work
- **Follow the phase order** unless there's a strong reason not to
- **Update progress** after completing each proposal
- **Run tests frequently** to catch issues early
- **Ask questions** if anything is unclear

### For Reviewers

- **Check alignment** with project principles
- **Validate priorities** - are high-impact items first?
- **Verify feasibility** - can this actually be done?
- **Suggest improvements** - what's missing?
- **Approve clearly** - signal when implementation can begin

## 📞 Questions?

- Open a GitHub issue with label `refactoring`
- Reference specific plan documents
- Tag relevant maintainers

---

**Last Updated**: October 10, 2025
