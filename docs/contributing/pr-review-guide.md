# PR Review Guide

This guide helps reviewers verify that pull requests meet our quality standards. Use this as a systematic checklist when reviewing code to ensure consistency and catch issues early.

> **Note**: This is a living document that will evolve as we learn from reviews. All automated checks (linters, tests, builds) must pass before review begins.

## 📋 Quality Standards Checklist

### Branching & Commits

Verify contributors followed these project conventions:

- [ ] **Branch name** follows `{issue-number}-{description}` format (see [branching.md](./branching.md))
- [ ] **Commit messages** use conventional format:
  - With issue branch: `{type}: [#{issue}] {description}`
  - Without issue branch: `{type}: {description}`
  - See [commit-process.md](./commit-process.md) for details
- [ ] **Pre-commit checks** passed (contributor should have run `./scripts/pre-commit.sh`)

### Code Quality

- [ ] **DDD layer placement** is correct - business logic in domain, orchestration in application, external integrations in infrastructure (see [ddd-layer-placement.md](./ddd-layer-placement.md))
- [ ] **Error handling** uses explicit enum errors with context, not generic strings (see [error-handling.md](./error-handling.md))
- [ ] **Errors are actionable** - include clear guidance on how to fix the issue
- [ ] **Module organization** follows conventions: public before private, top-down structure (see [module-organization.md](./module-organization.md))
- [ ] **Code is clean and maintainable** - clear naming, minimal complexity, well-structured
- [ ] **No obvious security vulnerabilities** introduced

### Testing

- [ ] **New logic has appropriate test coverage** (see [testing/](./testing/))
- [ ] **Tests follow naming conventions** (`it_should_...` pattern)
- [ ] **Tests are isolated** - use temporary resources, don't depend on external state
- [ ] **Tests are readable** - clear intent and easy to understand what's being tested
- [ ] **Test setup code is DRY** - no duplicate Arrange sections across tests
- [ ] **Test fixtures are decoupled** - helper functions use parameters, not hardcoded internal calls
- [ ] **Both production and test code** meet quality standards (clean, maintainable, sustainable)

### Documentation

- [ ] **Significant architectural decisions** documented as ADRs in `docs/decisions/` (see [decisions/README.md](../decisions/README.md))
- [ ] **Public APIs have rustdoc comments** with clear descriptions
- [ ] **Complex logic is explained** with comments where necessary
- [ ] **User-facing documentation updated** if behavior changes affect users

### Templates (if applicable)

- [ ] **Tera templates use correct syntax**: `{{ variable }}` not `{ { variable } }` (see [templates/tera.md](./templates/tera.md))
- [ ] **Static Ansible playbooks** (without `.tera` extension) are registered in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs`

## 🚩 Quick Red Flags

Watch for these common issues that indicate quality problems:

### Architecture Violations

- ❌ **File I/O in domain or application layers** - Should be in infrastructure
- ❌ **Business logic in infrastructure or presentation layers** - Should be in domain
- ❌ **Presentation layer directly calling infrastructure** - Should go through application layer
- ❌ **Domain layer depending on infrastructure** - Violates dependency flow rules
- ❌ **Mixing concerns across layers** - Each layer should have clear responsibilities

### Error Handling Issues

- ❌ **Generic string errors** instead of typed enums - Use explicit error types
- ❌ **Error messages without context** - Include what, where, when, why
- ❌ **Error messages without actionable guidance** - Tell user how to fix it
- ❌ **Lost error chains** - Missing source preservation, can't trace root cause
- ❌ **Using `anyhow` where explicit enums would be better** - Prefer pattern matching

### Code Organization Problems

- ❌ **Private items before public items** in modules - Public should come first
- ❌ **Low-level details before high-level abstractions** - Use top-down organization
- ❌ **Error types mixed with main implementation logic** - Separate concerns
- ❌ **Inconsistent naming** - Follow Rust conventions and project patterns

### Testing Issues

- ❌ **Tests using `unwrap()` without explanation** - Use proper error handling
- ❌ **Tests creating real directories** instead of temp directories - Use `TempDir`
- ❌ **Missing tests for new error paths** - Error handling should be tested
- ❌ **Tests that depend on external state** - Tests should be isolated
- ❌ **Test code that doesn't meet production quality standards** - Both should be clean
- ❌ **Duplicate test setup code** - Extract to helper functions (see PR [#373](https://github.com/torrust/torrust-tracker-deployer/pull/373))
- ❌ **Coupled helper functions** - Parameterize helpers instead of hardcoding dependencies
- ❌ **Field-by-field assertions on DTOs** - Add `PartialEq` and assert equality directly

### Known Issues vs. Real Problems

Be aware of expected behaviors documented in [known-issues.md](./known-issues.md):

- ✅ **SSH host key warnings** in E2E test output are normal and expected
- ✅ **Red error messages** during setup don't always indicate failure
- ❌ **New unexpected failures** should be investigated and resolved

## 🗣️ Providing Feedback

### Be Constructive and Specific

When providing feedback:

1. **Point to documentation** - Reference specific contributing guides
2. **Be specific** - Link to exact lines and explain the concern clearly
3. **Suggest alternatives** - Provide examples or point to similar code in the codebase
4. **Explain why** - Help contributors understand the reasoning behind standards
5. **Distinguish severity** - Make clear whether something is blocking or a suggestion

### Example Feedback

**Good feedback** references our documentation and is constructive:

> This error handling doesn't follow our guidelines from [error-handling.md](./error-handling.md). Errors should use explicit enums with context rather than generic strings.
>
> Please see the example in `src/domain/config/error.rs` which shows how to create a typed error enum with proper context fields. This will make the errors more actionable for users and easier to handle in calling code.

**Better feedback** is specific and actionable:

> In `src/application/commands/provision.rs` line 42, using `.unwrap()` will panic if the file doesn't exist. Instead:
>
> 1. Define an error variant in the command's error enum for file not found
> 2. Use `.map_err()` to convert the I/O error to your domain error
> 3. Include the file path in the error context
>
> See [error-handling.md](./error-handling.md) section 2 for the pattern.

### When to Request Changes vs. Comment vs. Approve

**Request Changes** - Blocking issues that violate documented standards:

- Architectural violations (wrong DDD layer placement)
- Security vulnerabilities introduced
- Breaking existing functionality
- Missing required tests for new code
- Error handling that doesn't meet standards
- Pre-commit checks not passing

**Comment** - Suggestions or questions (non-blocking improvements):

- Minor style inconsistencies not caught by linters
- Optional refactoring opportunities
- Questions about approach or implementation
- Nice-to-have documentation improvements
- Suggestions for future enhancements

**Approve** - All standards met:

- All quality checklist items verified
- No blocking issues found
- Changes are minimal and focused
- Tests pass and provide good coverage
- Documentation is adequate
- Code follows project conventions

## 🔄 Evolving This Guide

This is a living document. As we identify new patterns during reviews, we can:

1. **Update this guide** with new checklist items or red flags
2. **Update specific contributing guides** for detailed guidance on particular topics
3. **Add to "Quick Red Flags"** section for common review issues we encounter
4. **Document new architectural decisions** as ADRs in `docs/decisions/`
5. **Update issue templates** if acceptance criteria patterns emerge

If you notice patterns that should be added to this guide, please:

- Create an issue to discuss the addition
- Reference examples from recent PRs that show the pattern
- Propose specific checklist items or red flag entries
- Update relevant contributing guides with detailed guidance

## 📚 Related Resources

### Contributing Guides

- [DDD Layer Placement Guide](./ddd-layer-placement.md) - Architectural guidance
- [Error Handling Guide](./error-handling.md) - Error handling patterns
- [Module Organization](./module-organization.md) - Code organization within files
- [Branching Conventions](./branching.md) - Branch naming rules
- [Commit Process](./commit-process.md) - Commit message format
- [Testing Guide](./testing/) - Testing conventions and patterns
- [Templates Guide](./templates/) - Working with Tera templates
- [Known Issues](./known-issues.md) - Expected behaviors and workarounds

### Development Principles

- [Development Principles](../development-principles.md) - Core principles guiding all development
- [Codebase Architecture](../codebase-architecture.md) - Overall architecture overview
- [Architectural Decisions](../decisions/) - Decision records and rationale

### Tools and Automation

- [Linting Guide](./linting.md) - Running and configuring linters
- [Pre-commit Script](../../scripts/pre-commit.sh) - Automated quality checks

---

**Remember**: Reviews are about maintaining quality and helping contributors improve. Focus on being helpful, constructive, and educational. Point to documentation, provide examples, and explain the "why" behind standards. Together we build better software! 🎉
