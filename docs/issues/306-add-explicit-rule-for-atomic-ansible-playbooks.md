# Add explicit rule for atomic Ansible playbooks

**Issue**: #306
**Parent Epic**: None
**Related**: #292 (where the violation was discovered)

## Overview

Make the "one playbook = one responsibility" principle explicit and enforceable. The issue formalizes an ADR that defines atomic Ansible playbooks, updates contributor instructions (AGENTS rule #8) to require atomicity, and adds red-flag guidance and a pre-implementation checklist to the Ansible template guide. Conditional enablement logic must live in Rust (command/step), not in Ansible `when:` clauses. The ADR is the core deliverable (about 95% of the scope).

## Goals

- [ ] Publish an ADR that defines atomic Ansible playbooks, rationale, and consequences
- [ ] Update AGENTS rule #8 to mandate one-responsibility playbooks and Rust-driven conditional execution
- [ ] Add red flags and a pre-implementation checklist to the Ansible template guide to prevent multi-responsibility playbooks

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (Ansible tooling and documentation)
**Module Path**: docs/contributing/templates/ansible.md, AGENTS.md, docs/decisions/atomic-ansible-playbooks.md
**Pattern**: Documentation + ADR; Ansible playbook conventions (Command → Step → Playbook)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Conditional service enablement belongs in Rust command/step orchestration, not in Ansible `when:` clauses
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities

### Anti-Patterns to Avoid

- ❌ Adding new responsibilities to an existing playbook instead of creating a new one
- ❌ Using Ansible `when:` to decide if a service is enabled (should be gated in Rust)
- ❌ Mixing unrelated tasks (e.g., storage creation + config deployment) in a single playbook

## Specifications

### ADR: Atomic Ansible Playbooks

Document in [docs/decisions/atomic-ansible-playbooks.md](../decisions/atomic-ansible-playbooks.md):

- Context: prior violation in #292, implicit convention caused drift
- Decision: one playbook = one responsibility; conditional execution handled by Rust steps; static playbooks must be registered in `copy_static_templates()`; Tera only for templates, not logic
- Rationale: testability, composability, clearer failure domains, reproducibility, easier reviews
- Consequences: new features require a new playbook + Rust step gating; `when:` limited to host facts, not service enablement; playbook names must describe a single action
- Status: Proposed → Accepted in this issue
- Examples: wrong vs right patterns (storage creation embedded vs separate playbook + Rust gate)

### AGENTS Rule #8 Update

- Amend AGENTS rule #8 to state explicitly: one playbook = one responsibility
- Require conditional logic for enablement to reside in Rust command/step orchestration
- Remind to register static playbooks in `copy_static_templates()` so they are copied during generation

### Ansible Guide Red Flags & Checklist

Update [docs/contributing/templates/ansible.md](../contributing/templates/ansible.md) with:

- Red flags: adding tasks to existing playbooks, playbook names with "and", multiple unrelated file tasks, use of `when:` for feature enablement
- Correct pattern: new feature → new atomic playbook + new Rust step that decides whether to run it
- Pre-implementation checklist mirroring red flags to enforce atomicity

## Implementation Plan

### Phase 1: Author ADR (primary work)

- [ ] Draft `docs/decisions/atomic-ansible-playbooks.md` with context, decision, rationale, consequences, and examples
- [ ] Capture the Command → Step → Playbook flow showing where conditional checks live
- [ ] Include guidance on naming and registration (`copy_static_templates()`)

### Phase 2: Update contributor instructions

- [ ] Update AGENTS rule #8 to mandate atomic playbooks and Rust-driven conditional execution
- [ ] Add red flags and the pre-implementation checklist to [docs/contributing/templates/ansible.md](../contributing/templates/ansible.md)
- [ ] Cross-link the ADR from AGENTS and the Ansible guide

### Phase 3: Verification

- [ ] Run `./scripts/pre-commit.sh`
- [ ] Validate links and references; ensure no changes under data/

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] ADR `docs/decisions/atomic-ansible-playbooks.md` exists with decision, rationale, examples (wrong vs right), and consequences
- [ ] AGENTS rule #8 explicitly mandates one playbook = one responsibility, and clarifies Rust-side conditional execution; mentions registering static playbooks
- [ ] Ansible guide includes a "Red Flags" section and a pre-implementation checklist that steer contributors to create new atomic playbooks instead of modifying existing ones
- [ ] Guidance prohibits using Ansible `when:` for service/feature enablement and directs that gating to Rust steps
- [ ] Links between AGENTS, the Ansible guide, and the ADR are present and correct

## Related Documentation

- [AGENTS.md](../AGENTS.md)
- [docs/contributing/templates/ansible.md](../contributing/templates/ansible.md)
- [docs/decisions/README.md](../decisions/README.md)
- [docs/codebase-architecture.md](../codebase-architecture.md)
- [docs/contributing/error-handling.md](../contributing/error-handling.md)

## Notes

- Scope includes documentation and ADR only; no changes to runtime-managed data/.
- Emphasize that new playbooks are copied via `copy_static_templates()` and gated via Rust steps instead of Ansible `when:` clauses.
