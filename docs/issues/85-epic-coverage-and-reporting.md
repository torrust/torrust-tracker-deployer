Title: Coverage & Reporting EPIC - Test coverage checks, workflows, and docs

**Issue**: #85

## Overview

## Overview

This epic tracks improving test coverage visibility and guardrails across the project. The goal is to ensure coverage is visible, actionable, and enforced where appropriate so we keep regressions small and reviewers aware of coverage impacts.

This epic groups related tasks for coverage reporting infrastructure:

- Add coverage check to pre-commit script (non-blocking, runs last after all other checks)
- Create a CI workflow to generate coverage reports and upload artifacts
- Refactor testing documentation into organized folder structure
- Write new coverage documentation in the refactored structure

**Implementation Status:**

- ✅ The `cov-check` alias has been added to `.cargo/config.toml` using the native `cargo llvm-cov --fail-under-lines 85` option
- ⏳ The four tasks above are specified in this EPIC and will be implemented through separate issues

## Roadmap Reference

From [docs/roadmap.md](../roadmap.md):

> See "Development Process" guidance for creating roadmap tasks and epics. This EPIC adds quality tooling around tests and CI reporting to the roadmap's implementation practices.

## Tasks

- [ ] Add coverage check to pre-commit script (non-blocking, informational)
- [ ] Create a coverage workflow to generate the coverage report
- [ ] Refactor testing documentation structure into organized folder
- [ ] Write documentation about coverage (expectations, how-to, PR acceptance criteria)

(Detailed task specifications exist as child issue specifications in `docs/issues/`.)

### Child Issue Specifications

1. [#86 - `86-add-coverage-check-to-pre-commit.md`](./86-add-coverage-check-to-pre-commit.md) - Add `cargo cov-check` to pre-commit STEPS array
2. [#87 - `87-create-coverage-ci-workflow.md`](./87-create-coverage-ci-workflow.md) - Create GitHub Actions workflow for coverage reporting
3. [#88 - `88-refactor-testing-documentation-structure.md`](./88-refactor-testing-documentation-structure.md) - Split testing.md into organized folder structure
4. [#89 - `89-write-coverage-documentation.md`](./89-write-coverage-documentation.md) - Create coverage.md in refactored testing docs

**Implementation Order:**

Tasks 1 and 2 can be done independently. Task 3 must be completed before Task 4 (documentation refactoring before adding new coverage docs).

## Goals

- Improve visibility of test coverage across the repository
- Prevent accidental, unnoticed coverage regressions
- Make it easy for contributors to generate and inspect coverage locally

## Acceptance Criteria

- [ ] EPIC specification exists in `docs/issues/` and is reviewed
- [ ] The four child task specifications are clear and scoped to be implemented independently
- [ ] `cov-check` alias is available in `.cargo/config.toml` using native `--fail-under-lines 85`
- [ ] Pre-commit script includes coverage check (non-blocking, informational only)
- [ ] Testing documentation is organized in `docs/contributing/testing/` folder structure
- [ ] Coverage documentation exists at `docs/contributing/testing/coverage.md`
- [ ] Documentation explains how coverage is measured and how to run the local coverage command (references `.cargo/config.toml` aliases)

## Related

- Parent: #1 (Project Roadmap)
- See project coverage aliases in `.cargo/config.toml` (`cov`, `cov-lcov`, `cov-html`, `cov-codecov`)

---

Notes:

- This EPIC includes four child issues with detailed specifications in `docs/issues/`
- The `cov-check` alias was implemented immediately using cargo llvm-cov's native `--fail-under-lines` option rather than a custom script
- Coverage check in pre-commit is non-blocking to allow urgent patches/fixes even if coverage temporarily drops below threshold
- Testing documentation refactoring (task 3) must be completed before coverage documentation (task 4) to avoid mixing refactoring with new content
