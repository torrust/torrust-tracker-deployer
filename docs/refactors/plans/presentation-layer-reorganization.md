# Presentation Layer Reorganization

## 📋 Overview

This refactoring transforms the `src/presentation/` layer from an implicit, mixed-responsibility structure to an explicit four-layer architecture following industry-standard MVC/MVT patterns.

**Target Directory**: `src/presentation/`

**Scope**:

- Reorganize presentation layer into four explicit layers: Input, Dispatch, Controllers, Views
- Rename `user_output/` to `views/` with standard terminology
- Extract routing logic into dedicated `dispatch/router.rs`
- Integrate orphaned `progress.rs` into `views/progress/`
- Remove `commands/factory.rs` (use existing Container from `bootstrap/`)
- Update all import paths and dependencies

**Related Documentation**:

- [Research: CLI Organization Patterns](../../research/presentation-layer-organization-in-cli-apps.md)
- [Analysis: Current Structure](../../analysis/presentation-layer/current-structure.md)
- [Design Proposal](../../analysis/presentation-layer/design-proposal.md)

## 🔄 Engineering Process

**Note**: This refactor is complex and large-scale, requiring a more sophisticated engineering process than typical refactorings.

### 5-Step Process for Complex Refactorings

This refactoring followed a systematic 5-step engineering process:

| Step | Phase             | Document                                                                                   | Status      | Purpose                                                                                  |
| ---- | ----------------- | ------------------------------------------------------------------------------------------ | ----------- | ---------------------------------------------------------------------------------------- |
| 1    | **Research**      | [CLI Organization Patterns](../../research/presentation-layer-organization-in-cli-apps.md) | ✅ Complete | Industry patterns, best practices, examples from successful CLI apps                     |
| 2    | **Analysis**      | [Current Structure](../../analysis/presentation-layer/current-structure.md)                | ✅ Complete | Objective analysis of existing code, identify problems (3 critical, 3 moderate, 2 minor) |
| 3    | **Design**        | [Design Proposal](../../analysis/presentation-layer/design-proposal.md)                    | ✅ Complete | Propose four-layer architecture, evaluate alternatives, define Container integration     |
| 4    | **Refactor Plan** | This document                                                                              | ✅ Complete | High-level proposals, progressive detailing, mergeable states                            |
| 5    | **Issues**        | GitHub Issues                                                                              | ⏳ Next     | EPIC + progressive subissues (start with Proposals #1 and #2)                            |

### Why This Process?

**Complexity indicators** that triggered this approach:

- 🔴 **Large scope**: Entire presentation layer reorganization (50+ files affected)
- 🔴 **Architectural change**: Fundamental restructuring, not just refactoring
- 🔴 **Multiple phases**: 6 proposals with dependencies
- 🔴 **High risk**: Breaking changes require careful planning
- 🔴 **Learning curve**: New patterns (Container, ExecutionContext) to establish

**Benefits of this process**:

- ✅ Thorough research prevents reinventing wheels
- ✅ Clear problem identification focuses efforts
- ✅ Design exploration evaluates alternatives
- ✅ Progressive planning reduces upfront overhead
- ✅ Adaptive execution responds to discoveries

## 🎯 Approach

**Progressive & Incremental**: This refactoring follows a dynamic, adaptive approach:

1. **Start with high-level proposals** - Define the overall structure and goals
2. **Detail proposals incrementally** - Add implementation details only for the next 1-2 proposals
3. **Re-evaluate after each completion** - Review current state and adjust plan
4. **Adapt as we learn** - Update proposals based on what we discover during implementation

**Why this approach?**

- ✅ Reduces upfront planning overhead
- ✅ Allows learning and adaptation
- ✅ Each proposal remains independently mergeable
- ✅ Avoids over-specification of distant future work
- ✅ Responds to discovered issues during implementation

## 📊 Progress Tracking

**Total Proposals**: 6  
**Completed**: 0/6 (0%)  
**In Progress**: 0/6 (0%)  
**Not Started**: 6/6 (100%)

**Estimated Total Time**: 12-15 hours

### Proposal Summary

Each proposal is **independently mergeable** - it leaves the codebase in a complete, functional state with updated documentation and passing tests.

| #   | Proposal                      | Status         | Impact      | Effort      | Est. Time |
| --- | ----------------------------- | -------------- | ----------- | ----------- | --------- |
| 1   | Create Input Layer            | ⏳ Not Started | 🟢🟢 Medium | 🔵🔵 Medium | 2-3h      |
| 2   | Create Dispatch Layer         | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵 Medium | 2-3h      |
| 3   | Create Controllers Layer      | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵🔵 High | 3-4h      |
| 4   | Create Views Layer            | ⏳ Not Started | 🟢🟢🟢 High | 🔵🔵🔵 High | 3-4h      |
| 5   | Integrate Progress into Views | ⏳ Not Started | 🟢🟢 Medium | 🔵 Low      | 1-2h      |
| 6   | Remove Old Commands Structure | ⏳ Not Started | 🟢 Low      | 🔵 Low      | 1h        |

## 🎯 Key Problems Identified

### 1. Mixed Responsibilities

The `commands/` module mixes routing, context management, dependency injection, and command handling in a single directory.

### 2. No Explicit Router

Command dispatch logic is embedded in `commands/mod.rs` without clear separation from other concerns.

### 3. Unclear Terminology

`user_output/` doesn't follow industry-standard terminology (should be `views/` or `presentation/views/`).

### 4. Orphaned Modules

`progress.rs` sits at presentation root with no clear ownership or integration.

### 5. Unused Factory Pattern

`commands/factory.rs` duplicates the Container pattern from `bootstrap/container.rs`.

### 6. Scalability Concerns

Adding new commands requires modifying the monolithic dispatch function in `commands/mod.rs`.

## 🚀 Proposals

### Mergeable State Principle

**Each proposal must leave the codebase in a complete, functional, production-ready state.**

- ✅ All code compiles and passes tests
- ✅ Documentation explains the current structure
- ✅ References this refactor plan for full context
- ✅ Can be safely merged to `main`
- ✅ No intermediate or broken states

### Progressive Detailing

- **Initial proposals (1-2)**: Detailed implementation plans ready for immediate work
- **Future proposals (3-6)**: High-level goals only, to be detailed after earlier proposals complete
- **After each completion**: Re-evaluate plan, current state, and design; detail next 1-2 proposals

---

## Proposal #1: Create Input Layer

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Estimated Time**: 2-3 hours

### Goal

Establish an explicit **Input Layer** in the presentation structure by creating `src/presentation/input/` and moving the CLI module there.

### Why This First?

- Clear separation of user input parsing from command execution
- Establishes the pattern for the four-layer architecture
- Low risk - CLI is well-isolated and easy to move
- Introduces the refactor plan to documentation readers

### What Success Looks Like

After completion:

- ✅ `src/presentation/input/cli/` exists and contains CLI parsing
- ✅ All imports updated to use `presentation::input::cli`
- ✅ Module documentation explains the input layer purpose
- ✅ Documentation references this refactor plan for context
- ✅ Old structure (`commands/`, `user_output/`) remains functional
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed in the GitHub issue** - This proposal is ready for detailed planning in its dedicated issue.

---

## Proposal #2: Create Dispatch Layer

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Estimated Time**: 2-3 hours

### Goal

Extract command routing logic into an explicit **Dispatch Layer** with `src/presentation/dispatch/router.rs` and `ExecutionContext`.

### Why This Second?

- Separates routing from command execution
- Creates the `ExecutionContext` pattern for controller use
- Input layer (Proposal #1) must exist first for clean imports
- Foundation for moving commands to controllers

### What Success Looks Like

After completion:

- ✅ `src/presentation/dispatch/` exists with router and context
- ✅ `route_command()` function handles all command routing
- ✅ `ExecutionContext` wraps `Container` and provides command context
- ✅ `src/main.rs` uses new router instead of direct command execution
- ✅ Documentation explains dispatch layer and routing pattern
- ✅ Old `commands/` directory still functional
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed after Proposal #1 completes** - We'll re-evaluate the current state and add detailed implementation steps in the next iteration.

---

## Proposal #3: Create Controllers Layer

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 High  
**Estimated Time**: 3-4 hours

### Goal

Rename `commands/` to `controllers/` with standard terminology, integrate with `Container` for dependency injection, and update all command handlers to use `ExecutionContext`.

### Why This Third?

- Dispatch layer (Proposal #2) must exist to provide `ExecutionContext`
- Standard "Controllers" terminology aligns with web MVC pattern
- Integration with existing `Container` removes need for `factory.rs`
- Major rename - easier to do before views layer changes

### What Success Looks Like

After completion:

- ✅ `src/presentation/controllers/` contains all command handlers
- ✅ Controllers use `ExecutionContext` for dependencies
- ✅ Container integration complete (lazy-loaded services)
- ✅ Function names simplified (`handle_create_command` → `handle`)
- ✅ Documentation explains controllers layer
- ✅ Old `commands/` directory removed
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed after Proposal #2 completes** - Details will be added based on learnings from earlier proposals.

---

## Proposal #4: Create Views Layer

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 High  
**Estimated Time**: 3-4 hours

### Goal

Rename `user_output/` to `views/` with standard MVC terminology and reorganize into logical submodules (`messages/`, `terminal/`, `formatters/`).

### Why This Fourth?

- Can be done independently after controllers layer established
- Large reorganization - better to do after routing stabilized
- Standard "Views" terminology completes the MVC pattern
- Prepares for progress.rs integration

### What Success Looks Like

After completion:

- ✅ `src/presentation/views/` exists with organized submodules
- ✅ `messages/` contains all message types
- ✅ `terminal/` contains output channel and writer abstractions
- ✅ `formatters/` contains formatting logic
- ✅ All imports updated to use `presentation::views`
- ✅ Documentation explains views layer and organization
- ✅ Old `user_output/` removed
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed after Proposal #3 completes** - Structure will be refined based on earlier implementation experience.

---

## Proposal #5: Integrate Progress into Views

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Estimated Time**: 1-2 hours

### Goal

Move orphaned `src/presentation/progress.rs` into the views layer as `views/progress/` module, establishing clear ownership.

### Why This Fifth?

- Depends on views layer (Proposal #4) existing
- Progress indicators are UI concerns, belong in views
- Small, focused change after major reorganization
- No other modules depend on this location

### What Success Looks Like

After completion:

- ✅ `src/presentation/views/progress/` contains progress bar implementation
- ✅ Imports updated to use `presentation::views::progress`
- ✅ Documentation shows progress as part of views layer
- ✅ Old `progress.rs` at root removed
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed after Proposal #4 completes** - Details will be straightforward given the small scope.

---

## Proposal #6: Remove Old Commands Structure

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Estimated Time**: 1 hour

### Goal

Remove any remaining vestigial structures from the old organization, including `commands/factory.rs` if still present, and update documentation to mark migration complete.

### Why This Last?

- Cleanup step after all migrations complete
- Removes duplicate DI mechanism (factory vs Container)
- Final documentation pass
- Confirms no old references remain

### What Success Looks Like

After completion:

- ✅ No `commands/factory.rs` or other unused factory code
- ✅ No references to old structure in code
- ✅ Documentation updated to remove "ongoing migration" notes
- ✅ Clean, four-layer architecture fully established
- ✅ All tests pass, code compiles, ready to merge

### Implementation Details

**To be detailed after Proposal #5 completes** - May be unnecessary if earlier proposals already removed these.

---

## 📅 Workflow

### Issue Creation Strategy

1. **EPIC Issue**: Created from this refactor plan

   - Links to all subissue proposals
   - Tracks overall progress
   - Contains link to this document

2. **First Two Subissues**: Created immediately

   - Proposal #1 (Input Layer) - Detailed implementation plan
   - Proposal #2 (Dispatch Layer) - Detailed implementation plan

3. **Progressive Issue Creation**: After each completion
   - Re-evaluate current state
   - Review design proposal
   - Update this refactor plan if needed
   - Detail next 1-2 proposals
   - Create next subissue(s)

### After Each Proposal Completion

1. **Merge to main** - Proposal is independently mergeable
2. **Review current state** - What did we learn?
3. **Update refactor plan** - Adjust estimates, scope, approach
4. **Review design proposal** - Still accurate? Need changes?
5. **Detail next proposal(s)** - Add implementation specifics
6. **Create next issue(s)** - Ready for work

---

## 🔍 Review Process

### Completion Criteria for Entire Refactor

- [ ] All 6 proposals completed
- [ ] Four-layer architecture fully established
- [ ] All documentation accurate and complete
- [ ] No references to old structure remain
- [ ] All tests passing
- [ ] Code follows project conventions

### After Each Proposal

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Ready to merge to main
- [ ] Lessons learned documented

---

## 📝 Notes

### Advantages of Progressive Approach

- **Flexibility**: Can adjust course based on discoveries
- **Reduced waste**: Don't over-specify distant work
- **Learning**: Apply lessons from early proposals to later ones
- **Momentum**: Complete and merge frequently
- **Lower risk**: Smaller changes, more frequent integration

### Adaptation Points

After each proposal, consider:

- Did we discover new issues?
- Should proposal scope change?
- Are estimates still accurate?
- Do later proposals need adjustment?
- Is the design still optimal?

---

**Created**: November 6, 2025  
**Status**: 📋 Planning  
**Next Action**: Create EPIC issue and first two subissues (Proposals #1 and #2)
