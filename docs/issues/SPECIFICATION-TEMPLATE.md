# [Task Title]

**Issue**: #X
**Parent Epic**: #X - [Epic Name]
**Related**: [Links to related issues, ADRs, or documentation]

## Overview

[Clear description of what this task accomplishes]

## Goals

- [ ] Goal 1
- [ ] Goal 2
- [ ] Goal 3

## 🏗️ Architecture Requirements

**DDD Layer**: [Domain | Application | Infrastructure | Presentation]
**Module Path**: `src/{layer}/{module}/`
**Pattern**: [Command | Step | Action | CLI Subcommand | Entity | Value Object | Repository]

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Domain layer depending on infrastructure
- ❌ Monolithic modules with multiple responsibilities

## Specifications

### [Specification Section 1]

[Detailed specifications with code examples, configurations, etc.]

### [Specification Section 2]

[More detailed specifications]

## Implementation Plan

### Phase 1: [Phase Name] (estimated time)

- [ ] Task 1.1: [Specific, actionable task]
- [ ] Task 1.2: [Specific, actionable task]
- [ ] Task 1.3: [Specific, actionable task]

### Phase 2: [Phase Name] (estimated time)

- [ ] Task 2.1: [Specific, actionable task]
- [ ] Task 2.2: [Specific, actionable task]

### Phase 3: [Phase Name] (estimated time)

- [ ] Task 3.1: [Specific, actionable task]

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Related Documentation

- [Link to relevant docs]
- [Link to ADRs]
- [Link to examples]

## Notes

[Any additional context, decisions, or considerations]
