# Write Coverage Documentation

**Issue**: #89
**Parent Epic**: #85 - Coverage & Reporting EPIC
**Related**: [85-epic-coverage-and-reporting.md](./85-epic-coverage-and-reporting.md), [88-refactor-testing-documentation-structure.md](./88-refactor-testing-documentation-structure.md)

## Overview

Create comprehensive code coverage documentation (`docs/contributing/testing/coverage.md`) to provide guidance on coverage expectations, how to run coverage locally, how coverage is checked in CI/CD, and how coverage should be evaluated in PR reviews.

This is a **new documentation file** that will be added to the refactored testing documentation structure created in the `refactor-testing-documentation-structure` issue.

**Prerequisites**: The `refactor-testing-documentation-structure` issue must be completed first, as it creates the `docs/contributing/testing/` directory structure that this file will be added to.

## Goals

- [ ] Create new documentation file `docs/contributing/testing/coverage.md`
- [ ] Document all coverage commands and aliases
- [ ] Explain coverage expectations and thresholds
- [ ] Provide guidance for PR reviews involving coverage changes
- [ ] Document the pre-commit coverage check behavior
- [ ] Explain CI/CD coverage workflow
- [ ] Add link to `coverage.md` in testing README navigation

## 🏗️ Architecture Requirements

**DDD Layer**: Documentation
**Module Path**: `docs/contributing/testing/coverage.md`
**Pattern**: Contributor documentation

### Module Structure Requirements

- [ ] Follow existing documentation structure in `docs/contributing/testing/`
- [ ] Use consistent formatting with other testing guides
- [ ] Include practical examples and commands
- [ ] Add navigation link in `docs/contributing/testing/README.md`

### Architectural Constraints

- [ ] Must align with actual implementation (don't document features that don't exist)
- [ ] Must reference `.cargo/config.toml` aliases correctly
- [ ] Must explain non-blocking nature of coverage checks

### Anti-Patterns to Avoid

- ❌ Documenting features that aren't implemented yet
- ❌ Making coverage sound mandatory when it's informational
- ❌ Missing practical examples

## Specifications

### Document Structure

Create a new file `docs/contributing/testing/coverage.md` with these main sections:

1. **Coverage Targets** - What coverage percentages we aim for
2. **What We DON'T Require Coverage For** - Exceptions and special cases
3. **Running Coverage Locally** - Commands and aliases
4. **Pre-commit Coverage Check** - How coverage is checked before commits
5. **CI/CD Coverage Workflow** - How coverage is generated in GitHub Actions
6. **Coverage Expectations for PRs** - Guidelines for PR authors and reviewers
7. **Analyzing Low Coverage** - How to investigate and improve
8. **Coverage Checklist for PR Reviews** - What reviewers should check
9. **Best Practices** - Do's and don'ts
10. **Related Documentation** - Links to other guides

### Section 1: Coverage Targets

Explain the project's coverage goals:

```markdown
# Code Coverage Guide

This guide explains our code coverage practices, expectations, and how to work with coverage reports in the Torrust Tracker Deployer project.

## 📊 Coverage Targets

### Project-Wide Goals

- **Overall Coverage Target**: ≥ 85% (lines)
- **Critical Business Logic**: ≥ 90% (domain layer, commands, steps)
- **Shared Utilities**: ≥ 95% (clock, username, command executor)

### What We DON'T Require Coverage For

The following modules are **intentionally excluded** from strict coverage requirements:

1. **Binary Entry Points** (`src/bin/`, `src/main.rs`)

   - These are executables tested through actual execution
   - Coverage: Not measured

2. **E2E Test Infrastructure** (`src/testing/e2e/tasks/`)

   - Testing utilities that support E2E tests
   - Coverage: Not required

3. **Infrastructure Adapters** (when mocking adds no value)

   - `src/adapters/lxd/` - Requires real LXD
   - `src/adapters/tofu/` - Requires real OpenTofu
   - `src/infrastructure/remote_actions/` - Requires real remote infrastructure
   - Coverage: Tested via E2E tests

4. **Linting Package** (`packages/linting/`)

   - Primarily executed as binary, wraps external tools
   - Coverage: 30-40% is acceptable

5. **Error Types** (when only exercised in real scenarios)
   - Some error variants only occur in real infrastructure failures
   - Coverage: Partial coverage is acceptable
```

### Section 2: Running Coverage Locally

Document all available coverage aliases from `.cargo/config.toml`:

```markdown
## 🧪 Running Coverage

### Quick Coverage Check

Use `cargo cov-check` to validate coverage meets the 85% threshold:

(add bash code block with cargo cov-check command)

### Detailed Coverage Reports

(add bash code blocks for cargo cov-lcov, cov-codecov, cov-html commands)
```

### Section 3: Pre-commit Coverage Check

Explain how coverage is checked during pre-commit:

```markdown
## 🚨 Pre-commit Coverage Check

The pre-commit script includes an informational coverage check that runs last.

### How It Works

(document ./scripts/pre-commit.sh behavior)

The coverage check:

- Runs after all other pre-commit checks succeed
- Shows current coverage percentage
- Non-blocking - does NOT fail pre-commit if coverage is low
- Uses the 85% threshold for reporting

### Why Non-blocking?

Coverage checks are informational to allow security patches, refactoring, and WIP commits.

### Example Output

(add text code blocks showing pass and fail scenarios)
```

### Section 4: CI/CD Coverage Workflow

Document the GitHub Actions coverage workflow:

```markdown
## 🔄 CI/CD Coverage Workflow

Code coverage is automatically generated in GitHub Actions for every push to main and every pull request.

### What the Workflow Does

The coverage workflow generates coverage in three formats:

- LCOV: for coverage tools
- Codecov JSON: for Codecov service
- HTML: for human review

(add details about artifact uploads and Codecov integration)

### Accessing Coverage Reports

(document how to access reports from GitHub Actions and Codecov)

### Non-blocking Nature

The coverage workflow:

- Does NOT block merges if coverage is low
- Provides visibility into coverage changes
- Helps reviewers assess test quality
```

### Section 5: Coverage Expectations for PRs

Provide guidelines for PR authors and reviewers:

```markdown
## 📝 Coverage Expectations for PRs

### For New Features

When adding new features, aim for:

- New domain logic: ≥ 90% coverage
- New commands/steps: ≥ 85% coverage
- New utilities: ≥ 95% coverage
- Infrastructure adapters: E2E tests + reasonable unit tests

Note: These are targets, not blockers. PRs may be merged below these thresholds with proper justification.

### For Bug Fixes

When fixing bugs, aim to:

- Add a test that reproduces the bug
- Ensure the test passes after the fix
- Maintain or improve existing coverage

### For Refactoring

When refactoring code, aim to:

- Maintain or improve existing coverage
- Avoid decreasing overall project coverage below 85%
- Document any intentional coverage reductions

### When Coverage Drops

If your PR reduces coverage:

1. Explain why in the PR description
2. Justify the change
3. Plan when/how coverage will be restored (if applicable)
4. Reviewers will evaluate on case-by-case basis

## Implementation Plan

### Phase 1: Create Core Documentation Structure (40 minutes)

- [ ] Create new file `docs/contributing/testing/coverage.md`
- [ ] Add coverage targets section (85% overall, 90% business logic, 95% utilities)
- [ ] Document what doesn't require coverage (binaries, E2E infrastructure, adapters)
- [ ] Add "Running Coverage Locally" section with all cargo aliases
- [ ] Include command examples and expected outputs

### Phase 2: Add Pre-commit Section (20 minutes)

- [ ] Create "Pre-commit Coverage Check" section
- [ ] Document how the check works
- [ ] Explain why it's non-blocking
- [ ] Add example output for both pass and fail scenarios
- [ ] Link to `scripts/pre-commit.sh`

### Phase 3: Add CI/CD Section (20 minutes)

- [ ] Create "CI/CD Coverage Workflow" section
- [ ] Document what the workflow does (LCOV, HTML, Codecov generation)
- [ ] Explain how to access coverage reports from GitHub Actions
- [ ] Clarify non-blocking nature
- [ ] Link to `.github/workflows/coverage.yml`

### Phase 4: Add Remaining Sections (30 minutes)

- [ ] Create "Coverage Expectations for PRs" section
- [ ] Add "Analyzing Low Coverage" section with practical examples
- [ ] Create "Coverage Checklist for PR Reviews" section
- [ ] Add "Best Practices" section (Do's and Don'ts)
- [ ] Add "Related Documentation" section with links

### Phase 5: Review and Polish (15 minutes)

- [ ] Ensure consistent formatting throughout
- [ ] Check all links work
- [ ] Verify all commands are correct
- [ ] Add cross-references to related documentation
- [ ] Run markdown linter

### Phase 6: Validation (10 minutes)

- [ ] Run `./scripts/pre-commit.sh` to catch linting issues
- [ ] Have another contributor review for clarity
- [ ] Test all documented commands work as described

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Markdown linting passes (no MD errors)

**Task-Specific Criteria**:

- [ ] New file `docs/contributing/testing/coverage.md` created with complete documentation
- [ ] Link to `coverage.md` added in `docs/contributing/testing/README.md` navigation section
- [ ] All cargo coverage aliases documented (`cov`, `cov-check`, `cov-lcov`, `cov-codecov`, `cov-html`)
- [ ] Coverage targets clearly defined (85% overall, 90% business logic, 95% utilities)
- [ ] Exclusions documented (binaries, E2E infrastructure, adapters, linting tools)
- [ ] Pre-commit coverage check behavior is clearly explained
- [ ] CI/CD coverage workflow is documented
- [ ] Non-blocking nature of coverage checks is emphasized throughout
- [ ] Coverage expectations for PRs are clear and reasonable
- [ ] Example outputs are provided for key scenarios
- [ ] All command examples are tested and work correctly
- [ ] Links to related documentation are included and working
- [ ] Document aligns with actual implementation (no aspirational features)

## Related Documentation

- [Testing Conventions](../testing/README.md) - Main testing documentation
- [Development Principles](../../development-principles.md) - Quality standards
- [Pre-commit Process](./commit-process.md) - Pre-commit workflow
- [Coverage Documentation](../testing/coverage.md) - The file being created

## Notes

- This is a **new file** - `docs/contributing/testing/coverage.md` does not exist yet
- Must be created AFTER the `refactor-testing-documentation-structure` issue completes
- The documentation should reflect actual behavior, not aspirational features
- Emphasize the non-blocking, informational nature of coverage checks
- Make it clear that coverage is a tool, not a goal
- Provide practical examples that contributors can run immediately
- Balance between being thorough and being concise
- All content should be created from scratch based on the specifications above

## Dependencies

**Before This Issue**:

- `refactor-testing-documentation-structure` - Creates the `docs/contributing/testing/` directory structure

**After This Issue**:

- None - This is the final documentation task in the EPIC

## Related Documentation

- [Refactor Testing Documentation Structure](./refactor-testing-documentation-structure.md) - Prerequisites for this issue
- [Testing Conventions](../contributing/testing/README.md) - Will link to the new coverage.md (after both issues complete)
- [EPIC: Coverage & Reporting](./epic-coverage-and-reporting.md) - Parent EPIC
```
