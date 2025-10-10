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

| Document                                                                    | Status         | Target                                 | Created     |
| --------------------------------------------------------------------------- | -------------- | -------------------------------------- | ----------- |
| [Command Code Quality Improvements](./command-code-quality-improvements.md) | 🚧 In Progress | `ProvisionCommand`, `ConfigureCommand` | Oct 7, 2025 |

## ✅ Completed Refactorings

| Document                                 | Completed    | Target                                  | Notes                                                                                                                                                                                       |
| ---------------------------------------- | ------------ | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository Rename to Deployer            | Oct 10, 2025 | Repository and package names            | Renamed from "Torrust Tracker Deploy" to "Torrust Tracker Deployer" - Updated all references, package names, and added deprecation notices to PoC repositories (5 proposals, all completed) |
| Environment Context Three-Way Split      | Oct 8, 2025  | `EnvironmentContext`                    | See git history at `docs/refactors/environment-context-three-way-split.md` - Split context into UserInputs, InternalConfig, and RuntimeOutputs (4 proposals, all completed)                 |
| Environment Context Extraction           | Oct 8, 2025  | `Environment<S>`, `AnyEnvironmentState` | See git history at `docs/refactors/environment-context-extraction.md` - Extracted EnvironmentContext from Environment to reduce pattern matching (2 phases, all completed)                  |
| JSON File Repository Improvements        | Oct 3, 2025  | `json_file_repository.rs`               | See git history at `docs/refactors/json-file-repository-improvements.md` for the complete refactoring plan (9 proposals, all completed)                                                     |
| File Lock Improvements                   | Oct 3, 2025  | `file_lock.rs`                          | See git history at `docs/refactors/file-lock-improvements.md` for the complete refactoring plan (10 proposals, all completed)                                                               |
| Command Preparation for State Management | Oct 7, 2025  | `ProvisionCommand`, `ConfigureCommand`  | See git history at `docs/refactors/command-preparation-for-state-management.md` - Refactored commands to prepare for type-state pattern integration                                         |
| Error Context with Trace Files           | Oct 7, 2025  | Error handling infrastructure           | See git history at `docs/refactors/error-context-with-trace-files.md` - Replaced string-based error context with structured, type-safe context and trace files                              |
| Error Kind Classification Strategy       | Oct 7, 2025  | `Traceable` trait, error types          | See git history at `docs/refactors/error-kind-classification-strategy.md` - Moved error kind determination into error types via `Traceable` trait                                           |
| Step Tracking for Failure Context        | Oct 7, 2025  | Command execution flow                  | See git history at `docs/refactors/step-tracking-for-failure-context.md` - Added explicit step tracking to eliminate reverse engineering from error types                                   |

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

### 2. Review and Approval

1. Team reviews plan for technical feasibility
2. Validate alignment with project principles
3. Approve or request modifications
4. Set implementation timeline

### 3. Implementation

1. Create tracking issue (optional)
2. Create feature branch
3. Implement proposals in priority order
4. Update progress in plan document
5. Run tests and linters after each change

### 4. Completion

1. Final verification of all changes
2. Update plan document status to ✅ Completed
3. Create pull request
4. Merge after review approval

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
- [Testing Conventions](../contributing/testing.md) - Testing best practices

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

**Last Updated**: October 2, 2025
