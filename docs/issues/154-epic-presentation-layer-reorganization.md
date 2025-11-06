# Presentation Layer Reorganization (EPIC)

**Issue**: #154  
**Type**: Epic  
**Status**: Planning  
**Related Refactor Plan**: [docs/refactors/plans/presentation-layer-reorganization.md](../refactors/plans/presentation-layer-reorganization.md)

## Overview

This epic tracks the complete reorganization of the `src/presentation/` layer from an implicit, mixed-responsibility structure to an explicit four-layer architecture following industry-standard MVC/MVT patterns.

**Scope**: Transform the entire presentation layer (50+ files) into a clean, maintainable architecture with clear separation of concerns.

## Roadmap Reference

This epic implements a major architectural refactoring documented in the [Presentation Layer Reorganization Refactor Plan](../refactors/plans/presentation-layer-reorganization.md).

### Architecture Goals

Transform from:

```text
src/presentation/
├── cli/             # Clap definitions
├── commands/        # Mixed: routing + context + handlers
├── user_output/     # Output abstractions
├── progress.rs      # Orphaned progress indicator
└── errors.rs        # Error display
```

To:

```text
src/presentation/
├── input/           # Layer 1: CLI parsing
├── dispatch/        # Layer 2: Routing & context
├── controllers/     # Layer 3: Command handlers
├── views/           # Layer 4: Output formatting
│   └── progress/    # Integrated progress
└── errors.rs        # Error display
```

## 🎯 Goals

- [ ] Establish four-layer architecture (Input, Dispatch, Controllers, Views)
- [ ] Separate concerns: parsing → routing → handling → formatting
- [ ] Use standard terminology (controllers, views, not commands/user_output)
- [ ] Integrate Container from `bootstrap/` for dependency injection
- [ ] Integrate orphaned modules (`progress.rs`) into proper locations
- [ ] Remove duplicate patterns (`factory.rs`)
- [ ] Update all documentation and imports
- [ ] Maintain independently mergeable states throughout

## 🔄 Engineering Process

This refactoring followed a systematic **5-step engineering process** due to its complexity:

| Step | Phase             | Document                                                                                | Status      |
| ---- | ----------------- | --------------------------------------------------------------------------------------- | ----------- |
| 1    | **Research**      | [CLI Organization Patterns](../research/presentation-layer-organization-in-cli-apps.md) | ✅ Complete |
| 2    | **Analysis**      | [Current Structure](../analysis/presentation-layer/current-structure.md)                | ✅ Complete |
| 3    | **Design**        | [Design Proposal](../analysis/presentation-layer/design-proposal.md)                    | ✅ Complete |
| 4    | **Refactor Plan** | [Reorganization Plan](../refactors/plans/presentation-layer-reorganization.md)          | ✅ Complete |
| 5    | **Issues**        | This epic + progressive subissues                                                       | 🚀 Started  |

**Why this sophisticated process?** Large scope (50+ files), architectural change, multiple phases, high risk, learning curve. See [Engineering Process section](../refactors/plans/presentation-layer-reorganization.md#-engineering-process) for details.

## 📊 Tasks

### Phase 1: Foundation Layers

- [ ] #155 - Proposal 1: Create Input Layer
- [ ] #156 - Proposal 2: Create Dispatch Layer

### Phase 2: Core Transformation

- [ ] Proposal 3: Create Controllers Layer
- [ ] Proposal 4: Create Views Layer

### Phase 3: Integration & Cleanup

- [ ] #X - Proposal 5: Integrate Progress into Views
- [ ] #X - Proposal 6: Remove Old Commands Structure

(Proposals 3-6 will be created progressively after completing earlier proposals and re-evaluating the plan)

## 🎯 Approach: Progressive & Incremental

This refactoring uses a **dynamic, adaptive approach**:

1. **Start with high-level proposals** - Define overall structure (complete ✅)
2. **Detail proposals incrementally** - Add implementation details for next 1-2 proposals only
3. **Re-evaluate after each completion** - Review current state, adjust plan
4. **Adapt as we learn** - Update proposals based on discoveries

**Benefits**:

- ✅ Reduces upfront planning overhead
- ✅ Allows learning and adaptation
- ✅ Each proposal independently mergeable
- ✅ Responds to discovered issues

## 📈 Progress Tracking

**Total Proposals**: 6  
**Completed**: 0/6 (0%)  
**In Progress**: 0/6 (0%)  
**Not Started**: 6/6 (100%)

**Estimated Total Time**: 12-15 hours

| Proposal | Status         | Impact      | Effort      | Est. Time |
| -------- | -------------- | ----------- | ----------- | --------- |
| 1        | ⏳ Not Started | 🟢🟢 Medium | 🔵🔵 Medium | 2-3h      |
| 2        | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵 Medium | 2-3h      |
| 3        | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵🔵 High | 3-4h      |
| 4        | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵🔵 High | 3-4h      |
| 5        | ⏳ Not Started | 🟢🟢 Medium | 🔵 Low      | 1-2h      |
| 6        | ⏳ Not Started | 🟢 Low      | 🔵 Low      | 1h        |

See [Refactor Plan Progress Tracking](../refactors/plans/presentation-layer-reorganization.md#-progress-tracking) for detailed status.

## 🔴 Key Problems Being Solved

### 1. Mixed Responsibilities

The `commands/` module mixes routing, context management, dependency injection, and command handling.

### 2. No Explicit Router

Command dispatch logic is embedded without clear separation from other concerns.

### 3. Unclear Terminology

`user_output/` doesn't follow industry-standard terminology (should be `views/`).

### 4. Orphaned Modules

`progress.rs` sits at presentation root with no clear ownership.

### 5. Unused Factory Pattern

`commands/factory.rs` duplicates the Container pattern from `bootstrap/container.rs`.

### 6. Scalability Concerns

Adding new commands requires modifying the monolithic dispatch function.

## 📚 Related Documentation

### Process Documents

- [Research: CLI Organization Patterns](../research/presentation-layer-organization-in-cli-apps.md)
- [Analysis: Current Structure](../analysis/presentation-layer/current-structure.md)
- [Design Proposal](../analysis/presentation-layer/design-proposal.md)
- [Refactor Plan (Central Hub)](../refactors/plans/presentation-layer-reorganization.md)

### Project Guidelines

- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Module Organization](../contributing/module-organization.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Development Principles](../development-principles.md)

### Architecture

- [Codebase Architecture](../codebase-architecture.md)

## 🎯 Success Criteria

After all proposals complete:

- [ ] Four-layer architecture fully established
- [ ] All 50+ files reorganized with clear layer boundaries
- [ ] Standard terminology used (controllers, views, not commands/user_output)
- [ ] Container integration complete (no duplicate factory)
- [ ] All orphaned modules integrated
- [ ] All documentation updated
- [ ] All tests passing
- [ ] Code follows project conventions
- [ ] No references to old structure remain

## 🔄 Workflow

### After Each Proposal Completion

1. **Merge to main** - Each proposal is independently mergeable
2. **Review current state** - What did we learn?
3. **Update refactor plan** - Adjust estimates, scope, approach
4. **Review design proposal** - Still accurate? Need changes?
5. **Detail next proposal(s)** - Add implementation specifics (next 1-2 only)
6. **Create next issue(s)** - Ready for work

### Completion Criteria for Each Proposal

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Ready to merge to main
- [ ] Lessons learned documented in refactor plan

## 📝 Notes

### Mergeable State Principle

**Critical**: Each proposal must leave the codebase in a complete, functional, production-ready state:

- ✅ All code compiles and passes tests
- ✅ Documentation explains the current structure
- ✅ References refactor plan for full context
- ✅ Can be safely merged to `main`
- ✅ No intermediate or broken states

### Adaptation Points

After each proposal, consider:

- Did we discover new issues?
- Should proposal scope change?
- Are estimates still accurate?
- Do later proposals need adjustment?
- Is the design still optimal?

---

**Created**: November 6, 2025  
**Next Action**: Begin work on Proposal 1 (Create Input Layer)
