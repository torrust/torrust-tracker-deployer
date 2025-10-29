# Refactor Testing Documentation Structure

**Issue**: #88
**Parent Epic**: #85 - Coverage & Reporting EPIC
**Related**: [85-epic-coverage-and-reporting.md](./85-epic-coverage-and-reporting.md), [89-write-coverage-documentation.md](./89-write-coverage-documentation.md)

## Overview

Refactor the testing documentation from a single large `testing.md` file (652 lines) into a well-organized `testing/` folder structure with separate files for different concerns. This refactoring improves maintainability, discoverability, and sets the stage for adding new coverage documentation without mixing concerns.

**Why This Matters**: The current `testing.md` file has grown to over 650 lines and covers multiple distinct topics (unit testing, commands testing, E2E testing, pre-commit integration, etc.). Splitting this into separate focused files makes the documentation easier to navigate, maintain, and extend.

## Goals

- [ ] Create `docs/contributing/testing/` directory structure
- [ ] Split `testing.md` into focused, single-concern documents
- [ ] Create a main `README.md` as the testing documentation entry point
- [ ] Update all links referencing `testing.md` throughout the project
- [ ] Preserve all existing content without modification (pure refactoring)
- [ ] Prepare structure for adding new coverage documentation

## 🏗️ Architecture Requirements

**Documentation Layer**: Contributing Guidelines
**Module Path**: `docs/contributing/testing/`
**Pattern**: Documentation refactoring (no code changes)

### Proposed Directory Structure

```text
docs/contributing/testing/
  ├── README.md                    # Main entry point, overview, and quick start
  ├── unit-testing.md              # Unit test naming, AAA pattern, parameterized tests
  ├── resource-management.md       # TempDir usage, test isolation, cleanup
  ├── testing-commands.md          # Command test patterns, builders, mocks, E2E
  ├── clock-service.md             # MockClock usage for deterministic time tests
  └── pre-commit-integration.md    # AI enforcement tests, SKIP_AI_ENFORCEMENT flag
```

**Future addition** (done in `write-coverage-documentation.md` issue):

```text
docs/contributing/testing/
  └── coverage.md                  # Coverage generation, thresholds, reporting (NEW)
```

### Content Mapping

Map sections from current `testing.md` to new files:

#### `README.md` (Main Entry Point)

- Overview of testing philosophy
- Quick links to specialized docs
- "Getting Started" section (from current `testing.md`)
- Links to related documentation

#### `unit-testing.md`

- Unit Test Naming Style (`it_should_` prefix)
- Naming Convention and Examples
- Benefits section

#### `resource-management.md`

- Resource Management principles
- Key Rules (TempDir usage, avoiding production data)
- Good and bad examples

#### `testing-commands.md`

- Testing Commands section
- Command Test Patterns
- Unit Tests with Test Builders
- Mock Strategies for Commands
- Integration Tests with E2E Infrastructure
- Testing Destroy Command (all subsections)
- E2E Test Integration

#### `clock-service.md`

- Using the Clock Service for Deterministic Time Tests
- Why Use Clock Service?
- Production Code examples
- Test Code examples with MockClock
- Key Benefits and When to Use

#### `pre-commit-integration.md`

- Pre-commit Integration Testing
- How It Works
- Skipping Expensive Tests During Development
- Why This Approach
- Running Tests
- AI Assistant Integration

### Architectural Constraints

- ❌ **No content changes** - This is a pure refactoring (move only)
- ❌ **No functional changes** - All information must be preserved
- ✅ **Update all links** - Any `testing.md` references must be updated
- ✅ **Maintain context** - Each file should have clear intro and context
- ✅ **Cross-linking** - Related sections should link to each other

### Anti-Patterns to Avoid

- ❌ Modifying content during the refactoring (do that in separate issues)
- ❌ Breaking existing links without updating them
- ❌ Creating files without clear introductions
- ❌ Losing context by splitting too granularly
- ❌ Mixing refactoring with new content addition

## Specifications

### File-by-File Requirements

#### README.md (Main Entry Point)

**Purpose**: Serve as the landing page for all testing documentation

**Structure**:

1. **Title**: "Testing Conventions"
2. **Overview**: Brief intro to testing philosophy (from current intro)
3. **Quick Navigation**: Links to all specialized testing docs
4. **Principles Section**: Core testing principles (currently at line ~106)
5. **Getting Started**: Quick start guide (currently at line ~573)
6. **Good Practices Overview**: High-level summary with links to details
7. **Related Documentation**: Links to other contributing docs

**Example Structure**:

```markdown
# Testing Conventions

This document outlines the testing conventions for the Torrust Tracker Deployer project.

## 📚 Documentation Index

- [Unit Testing](./unit-testing.md) - Naming conventions, AAA pattern, parameterized tests
- [Resource Management](./resource-management.md) - Test isolation and cleanup
- [Testing Commands](./testing-commands.md) - Command test patterns and E2E integration
- [Clock Service](./clock-service.md) - Deterministic time testing with MockClock
- [Pre-commit Integration](./pre-commit-integration.md) - AI enforcement and CI validation

## 🎯 Principles

Test code should be held to the same quality standards as production code...
[Rest of principles section]

## 🚀 Getting Started

When writing new tests:

- Always use the `it_should_` prefix...
  [Rest of getting started section]
```

#### unit-testing.md

**Content from lines**: ~5-60 (Unit Test Naming Style section)

**Required sections**:

- Title: "Unit Testing Conventions"
- Introduction (new): Brief context about unit testing in the project
- Unit Test Naming Style (existing content)
- Naming Convention (existing content)
- Examples (existing content)
- Benefits (existing content)

#### resource-management.md

**Content from lines**: ~61-105 (Resource Management section)

**Required sections**:

- Title: "Resource Management in Tests"
- Introduction (new): Why test isolation matters
- Resource Management (existing content)
- Key Rules (existing content)
- Examples (existing content)

#### testing-commands.md

**Content from lines**: ~368-572 (Testing Commands section)

**Required sections**:

- Title: "Testing Commands"
- Introduction (new): Overview of command testing strategy
- Command Test Patterns (existing content)
- All subsections (existing content)

#### clock-service.md

**Content from lines**: ~258-367 (Clock Service section)

**Required sections**:

- Title: "Clock Service for Deterministic Time Tests"
- Introduction (new): Time as infrastructure concern
- Full existing content about MockClock

#### pre-commit-integration.md

**Content from lines**: ~584-652 (Pre-commit Integration Testing section)

**Required sections**:

- Title: "Pre-commit Integration Testing"
- Introduction (new): Why we test pre-commit in the test suite
- Full existing content about AI enforcement

### Link Updates Required

Search for all references to `testing.md` in the project and update them:

**Files to check**:

- `docs/contributing/README.md` - Main contributing guide
- `.github/copilot-instructions.md` - AI assistant instructions
- `docs/contributing/error-handling.md` - References testing patterns
- Any other `docs/` files that link to testing documentation

**Update pattern**:

- `docs/contributing/testing.md` → `docs/contributing/testing/README.md`
- Or more specific: `docs/contributing/testing/unit-testing.md` (if linking to specific section)

## Implementation Plan

### Phase 1: Create Directory Structure (5 minutes)

- [ ] Create `docs/contributing/testing/` directory
- [ ] Verify directory is created successfully

### Phase 2: Create README.md Entry Point (20 minutes)

- [ ] Create `docs/contributing/testing/README.md`
- [ ] Copy overview and introduction from current `testing.md`
- [ ] Add "Documentation Index" section with links to specialized docs
- [ ] Copy "Principles" section from `testing.md`
- [ ] Copy "Getting Started" section from `testing.md`
- [ ] Add "Related Documentation" section
- [ ] Verify all content is preserved

### Phase 3: Create Specialized Documentation Files (60 minutes)

Each file creation follows this pattern:

1. Create the file
2. Add title and brief introduction
3. Copy relevant content from `testing.md`
4. Add "Related Documentation" section at the end
5. Verify content completeness

**Files to create** (in order):

- [ ] `unit-testing.md` (lines ~5-60 from `testing.md`)
- [ ] `resource-management.md` (lines ~61-105 from `testing.md`)
- [ ] `testing-commands.md` (lines ~368-572 from `testing.md`)
- [ ] `clock-service.md` (lines ~258-367 from `testing.md`)
- [ ] `pre-commit-integration.md` (lines ~584-652 from `testing.md`)

### Phase 4: Update Cross-References (30 minutes)

- [ ] Search project for `testing.md` references: `grep -r "testing.md" docs/`
- [ ] Update references in `docs/contributing/README.md`
- [ ] Update references in `.github/copilot-instructions.md`
- [ ] Update references in `docs/contributing/error-handling.md`
- [ ] Update any other files that link to testing documentation
- [ ] Verify all links are updated correctly

### Phase 5: Remove Old File (5 minutes)

- [ ] Delete `docs/contributing/testing.md`
- [ ] Verify file is removed from git tracking

### Phase 6: Validation (20 minutes)

- [ ] Run markdown linter: `cargo run --bin linter markdown`
- [ ] Check all links work: manually verify navigation between docs
- [ ] Verify no content was lost: compare line counts
- [ ] Run cspell: ensure no new spelling issues
- [ ] Review all files for completeness

## Acceptance Criteria

### Functional Requirements

- [ ] `docs/contributing/testing/` directory exists with 6 files total
- [ ] All content from original `testing.md` is preserved (no deletions)
- [ ] `README.md` serves as clear entry point with navigation
- [ ] Each specialized file has clear introduction and context
- [ ] All cross-references between files work correctly
- [ ] Original `docs/contributing/testing.md` is removed

### Link Integrity

- [ ] All internal links within testing docs work
- [ ] All external links TO testing docs are updated
- [ ] No broken links in the documentation

### Quality Requirements

- [ ] All markdown files pass linting: `cargo run --bin linter markdown`
- [ ] All files pass spell check: `cargo run --bin linter cspell`
- [ ] Consistent formatting across all files
- [ ] Clear navigation and cross-linking

### Verification

Run these commands to verify:

```bash
# Verify directory structure
ls -la docs/contributing/testing/

# Should show 6 files:
# README.md
# unit-testing.md
# resource-management.md
# testing-commands.md
# clock-service.md
# pre-commit-integration.md

# Verify old file is removed
[ ! -f docs/contributing/testing.md ] && echo "✅ Old file removed" || echo "❌ Old file still exists"

# Check for broken links to testing.md
grep -r "testing\.md" docs/ --exclude-dir=issues

# Run linters
cargo run --bin linter markdown
cargo run --bin linter cspell
```

## Dependencies

**Before This Issue**:

- None - This is the first implementation step

**After This Issue**:

- `write-coverage-documentation.md` - Adds `coverage.md` to the new structure

## Estimated Time

**Total**: ~2.5 hours

- Phase 1: 5 minutes
- Phase 2: 20 minutes
- Phase 3: 60 minutes
- Phase 4: 30 minutes
- Phase 5: 5 minutes
- Phase 6: 20 minutes

## Benefits

### Maintainability

- Smaller, focused files are easier to update
- Changes to one concern don't affect others
- Clear separation of responsibilities

### Discoverability

- Users can find specific information faster
- Table of contents in README provides overview
- Each file focuses on one topic

### Extensibility

- Easy to add new testing documentation (like coverage)
- Clear where to add new content
- Doesn't bloat a single large file

### Developer Experience

- Faster to navigate to relevant sections
- Easier to link to specific testing topics
- Better organization reduces cognitive load

## Notes

- **Pure Refactoring**: This issue is ONLY about restructuring - no content changes
- **Preserve Everything**: All information must be retained exactly as-is
- **Link Integrity**: Critical to update all references to avoid broken links
- **Prepare for Coverage**: Sets up structure for coverage documentation in next issue

## Related Documentation

- [Write Coverage Documentation](./write-coverage-documentation.md) - Adds coverage.md after this refactoring
- [EPIC: Coverage & Reporting](./epic-coverage-and-reporting.md) - Parent EPIC
- [Module Organization](../contributing/module-organization.md) - Code organization patterns
