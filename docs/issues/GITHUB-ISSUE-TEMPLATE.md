Title: [Task Name from Roadmap]

## Overview

[Brief description of what this task accomplishes]

## Specification

See detailed specification: [docs/issues/{number}-{name}.md](../docs/issues/{number}-{name}.md)

(Link will be updated after file rename)

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

## Implementation Plan

### Phase 1: [Phase Name]

- [ ] Task 1.1
- [ ] Task 1.2

### Phase 2: [Phase Name]

- [ ] Task 2.1

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Criterion 1
- [ ] Criterion 2

## Related

- Parent: #X (Epic: [Epic Name])
- Roadmap: #1 (Project Roadmap)
- Specification: docs/issues/{number}-{name}.md
