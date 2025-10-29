# Issue: Add PR Review Guide for Contributors

**Issue**: [#80](https://github.com/torrust/torrust-tracker-deployer/issues/80)
**Related**: [#75](https://github.com/torrust/torrust-tracker-deployer/issues/75) - Move config module to correct DDD layer
**Depends On**: [#79](https://github.com/torrust/torrust-tracker-deployer/issues/79) - Add DDD Layer Placement Guide

## 📋 Issue Information

- **Type**: Documentation Enhancement
- **Priority**: High
- **Related Issue**: #75 - Move config module to correct DDD layer
- **Depends On**: Issue for DDD Layer Placement Guide (should reference that guide)
- **Parent Epic**: None (standalone improvement)

## 🎯 Problem Statement

Code reviews currently lack a systematic guide for reviewers. This has allowed issues like architectural violations (#75 - config module in wrong layer) to be merged without detection.

Reviewers need:

- A quick reference guide for important review aspects
- Clear red flags to watch for
- Consistent review criteria across all PRs

Without a standardized guide, we risk:

- Merging code that violates architectural principles
- Inconsistent review quality
- Missing important quality aspects
- Technical debt accumulation

## 💡 Proposed Solution

Create a concise PR review guide at `docs/contributing/pr-review-guide.md` that consolidates project quality standards for reviewers.

**Key Insight**: The quality standards in `.github/copilot-instructions.md` (rules 1-8) provide a good foundation for what reviewers should check. Extract and adapt these into a reviewer-focused checklist.

### Guide Structure

The guide should:

1. **Quality Standards Checklist** - Direct checklist of what reviewers should verify:
   - Branching conventions followed
   - Commit message format correct
   - Error handling patterns used
   - Module organization followed
   - DDD layer placement correct
   - Testing conventions applied
2. **Quick Red Flags Section** - Common violations that warrant immediate attention
3. **Providing Feedback Guidance** - How to review effectively:
   - How to provide constructive feedback
   - When to request changes vs. approve
   - How to prioritize issues (blocking vs. nice-to-have)
4. **Living Document Note** - Can be expanded as we learn from reviews

### Example Guide Structure

```markdown
# PR Review Guide

This guide helps reviewers verify that pull requests meet our quality standards.

> **Note**: All automated checks (linters, tests, builds) must pass before review begins.

## 📋 Quality Standards Checklist

### Core Quality Standards

Verify contributors followed these project conventions:

**Branching & Commits:**

- [ ] Branch name follows `{issue-number}-{description}` format (see [branching.md](./branching.md))
- [ ] Commit messages use conventional format: `{type}: [#{issue}] {description}` (see [commit-process.md](./commit-process.md))
- [ ] Pre-commit checks passed (contributor should have run `./scripts/pre-commit.sh`)

**Code Quality:**

- [ ] Error handling uses explicit enum errors with context (see [error-handling.md](./error-handling.md))
- [ ] Errors are clear, informative, and actionable
- [ ] Module organization follows conventions: public before private, top-down (see [module-organization.md](./module-organization.md))
- [ ] DDD layer placement is correct (see [DDD Layer Placement Guide](./ddd-layer-placement.md))

**Testing:**

- [ ] New logic has appropriate test coverage (see [testing.md](./testing.md))
- [ ] Tests follow naming conventions (`it_should_...` pattern)
- [ ] Tests are isolated and use temporary resources

**Documentation:**

- [ ] Significant architectural decisions documented as ADRs (see [decisions/](../decisions/))
- [ ] Public APIs have rustdoc comments
- [ ] Complex logic is explained

**Templates (if applicable):**

- [ ] Tera templates use correct variable syntax: `{{ variable }}` not `{ { variable } }` (see [templates.md](./templates.md))

### Quick Red Flags 🚩

Watch for these common issues that indicate quality problems:

**Architecture:**

- ❌ File I/O in domain or application layers
- ❌ Business logic in infrastructure or presentation layers
- ❌ Presentation layer directly calling infrastructure (should go through application)

**Error Handling:**

- ❌ Generic string errors instead of typed enums
- ❌ Error messages without context or actionable guidance
- ❌ Lost error chains (missing source preservation)

**Code Organization:**

- ❌ Private items before public items in modules
- ❌ Low-level details before high-level abstractions
- ❌ Error types mixed with main implementation logic

**Testing:**

- ❌ Tests using `unwrap()` without explanation
- ❌ Tests creating real directories instead of temp directories
- ❌ Missing tests for new error paths

## 🗣️ Providing Feedback

### Be Constructive

1. **Point to documentation** - Reference specific contributing guides
2. **Be specific** - Link to exact lines and explain the concern
3. **Suggest alternatives** - Provide examples or point to similar code in the codebase
4. **Distinguish severity**:
   - **Request changes** - Blocking issues (violations of documented standards)
   - **Comment** - Suggestions or questions (non-blocking improvements)
   - **Approve** - All standards met, ready to merge

### Example Feedback

Good feedback references our documentation:

> "This error handling doesn't follow our guidelines from `docs/contributing/error-handling.md`. Errors should use explicit enums with context rather than generic strings. Please see the `ConfigValidationError` enum in `src/domain/config.rs` as an example."

## 🔄 Evolving This Guide

This is a living document. As we identify new patterns during reviews, we can:

1. Update this guide with new checklist items
2. Update specific contributing guides for detailed guidance
3. Add to "Quick Red Flags" section for common review issues
4. Document new architectural decisions as ADRs
```

### Integration with Review Process

The guide should be:

1. **Comprehensive checklist** - Complete list of quality standards reviewers should verify
2. **Review-specific** - Focused on HOW to review effectively
3. **Quick red flags** - Visual checklist of common issues for fast scanning
4. **Living document** - Updated as we learn from reviews
5. **Linked from contributing guide** - Part of standard documentation

## 📝 Implementation Plan

### Deliverable

Create `docs/contributing/pr-review-guide.md` with:

1. **Introduction**

   - Purpose: Guide for reviewers on HOW to review effectively
   - Note: All automated checks must pass first
   - Scope: Focuses on quality standards not automatically checked

2. **Quality Standards Checklist Section**

   - Branching and commits (format, conventions)
   - Code quality (error handling, module organization, DDD layers)
   - Testing (coverage, conventions, isolation)
   - Documentation (ADRs, rustdoc, explanations)
   - Templates (if applicable)
   - Links to detailed contributing guides for each area

3. **Quick Red Flags Section**

   - Visual checklist of common violations
   - Architecture, error handling, code organization, testing categories
   - Easy to scan during review

4. **Providing Feedback Section**

   - How to be constructive and specific
   - How to reference documentation in feedback
   - When to use "request changes" vs "comment" vs "approve"
   - Example of good feedback

5. **Evolving This Guide Section**
   - How to update as we learn
   - Where to add new guidance (copilot-instructions vs contributing guides vs this doc)

### Steps

1. Create the guide document with comprehensive quality checklist
2. Extract relevant review points from `.github/copilot-instructions.md` (for inspiration, not direct reference)
3. Add quick red flags section for fast scanning
4. Include example feedback showing how to reference our docs
5. Update issue templates to remind contributors about acceptance criteria:
   - Add note to `docs/issues/SPECIFICATION-TEMPLATE.md` Acceptance Criteria section
   - Add note to `docs/issues/GITHUB-ISSUE-TEMPLATE.md` Acceptance Criteria section
   - Note content: "These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations."
6. Update roadmap-issues.md documentation:
   - Add section "Acceptance Criteria as PR Review Checklist" after "Common Acceptance Criteria" section
   - Explain that criteria serve as pre-review checklist for contributors
   - Emphasize benefits: clear expectations, self-review, faster merges
7. Link from `docs/contributing/README.md`
8. Run linters and commit

### Integration Points

- **Contributing Guide**: Add link from `docs/contributing/README.md`
- **Related Guides**: Cross-reference DDD placement, error handling, testing, module organization, branching, commit process guides

## ✅ Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

- [ ] Document created at `docs/contributing/pr-review-guide.md`
- [ ] Comprehensive quality standards checklist included
- [ ] Checklist covers: branching, commits, code quality, testing, documentation
- [ ] Quick red flags section for common violations
- [ ] Example feedback showing how to reference docs
- [ ] Guidance on when to request changes vs. comment vs. approve
- [ ] Issue templates updated with contributor note about acceptance criteria
- [ ] `docs/issues/SPECIFICATION-TEMPLATE.md` updated with pre-review checklist note
- [ ] `docs/issues/GITHUB-ISSUE-TEMPLATE.md` updated with pre-review checklist note
- [ ] `docs/contributing/roadmap-issues.md` updated with "Acceptance Criteria as PR Review Checklist" section
- [ ] Linked from `docs/contributing/README.md`
- [ ] All linters pass (markdownlint, cspell)
- [ ] Document follows project markdown conventions
- [ ] Clearly states it's a living document that will evolve

## 🔗 Related Issues

- [#75](https://github.com/torrust/torrust-tracker-deployer/issues/75) - Move config module to correct DDD layer (the issue that revealed this need)
- [#79](https://github.com/torrust/torrust-tracker-deployer/issues/79) - Add DDD Layer Placement Guide (dependency)

## 📚 Reference Materials

### Internal Documentation

- **DDD layer placement guide** (to be created in separate issue) - Includes authoritative DDD resources and examples
- Error handling guide (`docs/contributing/error-handling.md`)
- Testing conventions (`docs/contributing/testing.md`)
- Module organization guide (`docs/contributing/module-organization.md`)
- Development principles (`docs/development-principles.md`)
- Branching conventions (`docs/contributing/branching.md`)
- Commit process (`docs/contributing/commit-process.md`)
- Templates guide (`docs/contributing/templates.md`)

## 🏷️ Labels

- `documentation`
- `enhancement`
- `code-review`
- `contributing-guide`
