# Feature Development

## 📋 Purpose

Feature documentation serves to:

- **Define requirements** before implementation begins
- **Clarify scope and goals** through structured questions
- **Track progress** on feature development
- **Provide context** for reviewers and future maintainers
- **Align team** on feature design and implementation
- **Preserve decisions** and rationale for future reference

## 📁 Active Features

See [active-features.md](./active-features.md) for the current list of ongoing feature work.

## ✅ Completed Features

See [completed-features.md](./completed-features.md) for the complete history of finished features.

## 🔄 Feature Development Workflow

### 1. Draft Feature Specification

Create a new folder in `docs/features/{feature-name}/` with three core documents:

- **README.md** - Feature overview, status, and quick summary
- **questions.md** - Clarifying questions to refine requirements
- **specification.md** - Detailed technical specification

**Recommended structure** (flexible based on feature complexity):

```text
docs/features/{feature-name}/
├── README.md          # Overview, status tracking, quick summary
├── questions.md       # Questions for product owner/stakeholders
└── specification.md   # Technical specification and implementation details
```

### 2. Ask Questions (Optional but Recommended)

The `questions.md` file helps clarify:

- **Scope**: What's included vs. what's deferred?
- **Requirements**: What are the must-haves vs. nice-to-haves?
- **Constraints**: What limitations or dependencies exist?
- **Success criteria**: How do we know it's complete?
- **Priority**: What's the timeline and urgency?

**Product owner or stakeholders** answer these questions directly in the document to provide clear direction.

### 3. Update Specification

Based on question answers:

1. Refine the specification with clearer requirements
2. Update scope and goals
3. Add implementation details
4. Document decisions and rationale
5. Define acceptance criteria

### 4. Implementation

1. Create implementation plan (can be in specification or separate document)
2. Break down into phases or milestones
3. Track progress with checkboxes
4. Commit frequently with conventional commit messages
5. Update documentation as implementation progresses

### 5. Completion

When feature is complete:

1. Mark status as ✅ Complete in feature README
2. Move entry from [active-features.md](./active-features.md) to [completed-features.md](./completed-features.md)
3. Feature documentation remains in repository for reference
4. Implementation history preserved in git

### 6. Cleanup (Optional)

For features that are fully mature and well-documented elsewhere:

- Consider moving to git history (delete from main branch)
- Update [completed-features.md](./completed-features.md) to reference git commit/tag where documentation existed
- Keep entry in [completed-features.md](./completed-features.md) with link to git history

## 🎯 Feature Document Structure

Each feature should have three core documents. **Templates are provided** to make creating new features easier.

### Templates

Use these templates when creating a new feature:

- **[TEMPLATE-README.md](./TEMPLATE-README.md)** - Feature overview and status tracking
- **[TEMPLATE-QUESTIONS.md](./TEMPLATE-QUESTIONS.md)** - Clarifying questions for stakeholders
- **[TEMPLATE-SPECIFICATION.md](./TEMPLATE-SPECIFICATION.md)** - Detailed technical specification

### Quick Start

To create a new feature:

1. Copy the templates to a new folder: `docs/features/{feature-name}/`
2. Rename files (remove `TEMPLATE-` prefix): `README.md`, `questions.md`, `specification.md`
3. Fill in the templates with feature-specific information
4. Add the new feature to [active-features.md](./active-features.md)

### Document Purposes

- **README.md**: High-level overview, status tracking, quick reference
- **questions.md**: Clarify requirements with product owners before implementation
- **specification.md**: Detailed technical design, implementation plan, acceptance criteria

## 📊 Status Legend

- 📋 **Specified** - Requirements documented, awaiting implementation
- 🚧 **In Progress** - Implementation has started
- ✅ **Completed** - Feature fully implemented and merged
- ⏸️ **Deferred** - Work postponed for future consideration
- 🔄 **Refactoring** - Being redesigned or improved
- ❌ **Cancelled** - Feature abandoned or superseded

## 🎓 Best Practices

### When to Create a Feature Document

Create feature documentation when:

- ✅ Adding new user-facing capabilities
- ✅ Implementing significant new functionality
- ✅ Building features that span multiple components
- ✅ Work requires stakeholder alignment
- ✅ Design decisions need documentation
- ✅ Implementation will take multiple sessions

Skip formal feature documentation for:

- ❌ Simple bug fixes
- ❌ Internal code improvements (use refactoring docs)
- ❌ Trivial enhancements
- ❌ Emergency hotfixes

### Feature Quality

Good feature specifications:

- ✅ **Clear problem statement** - Explain why this matters
- ✅ **Defined scope** - What's in and what's out
- ✅ **User-focused goals** - How does this help users?
- ✅ **Technical feasibility** - Can we actually build this?
- ✅ **Measurable outcomes** - How do we know we're done?
- ✅ **Risk assessment** - What could go wrong?

### Progress Tracking

Keep documentation updated:

- **After each milestone**: Update status and checklist
- **When blocked**: Document blockers and decisions
- **On completion**: Mark feature as complete
- **Post-implementation**: Add lessons learned (optional)

## 🔗 Relationship to Refactoring

**Features** add new capabilities for users.  
**Refactorings** improve existing code quality.

| Aspect         | Features                                | Refactorings                               |
| -------------- | --------------------------------------- | ------------------------------------------ |
| **Purpose**    | Add new user-facing functionality       | Improve code quality and maintainability   |
| **Outcome**    | New capabilities, behaviors, or options | Better structure, performance, readability |
| **Visibility** | Users see and use the changes           | Users don't notice (internal improvement)  |
| **Location**   | `docs/features/`                        | `docs/refactors/`                          |

Some work may involve both - implement a feature, then refactor to improve it.

## 📚 Related Documentation

- [Refactoring Plans](../refactors/README.md) - Code quality improvements
- [Development Principles](../development-principles.md) - Core principles guiding development
- [Contributing Guidelines](../contributing/README.md) - General contribution process
- [Architectural Decision Records](../decisions/README.md) - Significant design decisions

## 💡 Tips

### For Feature Authors

- **Start with the problem** - Why does this feature matter?
- **Use questions.md** - Clarify ambiguities early
- **Keep it simple** - Avoid over-engineering
- **Include examples** - Show how it works
- **Define success** - Clear acceptance criteria
- **Get feedback early** - Share draft specifications

### For Product Owners

- **Answer questions promptly** - Unblock development
- **Prioritize clearly** - What's must-have vs. nice-to-have?
- **Be specific** - Vague requirements lead to wrong solutions
- **Consider constraints** - Technical limitations matter
- **Validate assumptions** - Check if the proposed solution fits needs

### For Implementers

- **Read everything** - Questions, specification, related docs
- **Ask for clarification** - Don't guess on unclear requirements
- **Update as you go** - Keep documentation current
- **Track progress** - Update checklists and status
- **Document decisions** - Capture important choices made during implementation

### For Reviewers

- **Verify completeness** - All acceptance criteria met?
- **Check alignment** - Matches specification and principles?
- **Test thoroughly** - Does it actually work?
- **Provide feedback** - Constructive suggestions for improvement
- **Approve clearly** - Signal when it's ready to merge

## 📞 Questions?

- Open a GitHub issue with label `feature`
- Reference specific feature documents
- Tag relevant maintainers for feedback

---

**Last Updated**: October 9, 2025
