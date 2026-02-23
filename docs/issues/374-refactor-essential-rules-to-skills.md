# Refactor Essential Rules into Agent Skills

**Issue**: [#374](https://github.com/torrust/torrust-tracker-deployer/issues/374)
**Parent Epic**: [#274](https://github.com/torrust/torrust-tracker-deployer/issues/274) - Adopt Agent Skills Specification
**Related**: [agentskills.io specification](https://agentskills.io/specification), [AGENTS.md](../../AGENTS.md)

## Overview

The `AGENTS.md` Essential Rules section currently contains 20 rules that are loaded into every AI agent interaction, consuming ~3000+ tokens of always-on context. Analysis against the [Agent Skills specification](https://agentskills.io/specification) reveals that most of these rules are **task-triggered** ("When X...", "Before Y...") and would be better served as on-demand skills using progressive disclosure.

This refactoring would:

- Reduce always-loaded context from ~3000 tokens to ~200 tokens
- Improve guidance quality through richer, on-demand skill instructions
- Remove 6 rules that are already duplicated by existing skills
- Align with the progressive disclosure model from the Agent Skills spec

## Goals

- [ ] Remove 6 Essential Rules already duplicated by existing skills
- [ ] Convert 12 task-triggered rules into new Agent Skills
- [ ] Slim down 2 borderline rules to concise always-on summaries
- [ ] Update the skills table in AGENTS.md to reference all new skills
- [ ] Validate that all new skills pass the `skills-ref validate` tool (if available)

## Analysis

### Category 1: Already Duplicated by Existing Skills (REMOVE)

These rules repeat content that is already covered in detail by existing skills. They should be removed from Essential Rules entirely.

| Rule | Topic                 | Existing Skill              |
| ---- | --------------------- | --------------------------- |
| 4    | Branch naming         | `create-feature-branch`     |
| 5    | Commit format         | `commit-changes`            |
| 6    | Pre-commit script     | `run-pre-commit-checks`     |
| 12   | Create ADRs           | `create-adr`                |
| 18   | Unit test naming      | `write-unit-test`           |
| 20   | Env config generation | `create-environment-config` |

### Category 2: Convert to New Skills (12 Rules)

These rules each start with "When..." or "Before...", indicating they are task-triggered. They fit the progressive disclosure model: don't load until the agent is performing the specific task.

| Rule | Topic                          | Suggested Skill Name           | Trigger                            |
| ---- | ------------------------------ | ------------------------------ | ---------------------------------- |
| 2    | DDD layer placement            | `place-code-in-ddd-layers`     | Writing/moving code between layers |
| 3    | Domain types with invariants   | `implement-domain-types`       | Creating value objects/entities    |
| 7    | Tera templates                 | `work-with-tera-templates`     | Editing `.tera` files              |
| 8    | Ansible playbooks              | `add-ansible-playbook`         | Creating playbooks                 |
| 9    | Error handling                 | `handle-errors-in-code`        | Writing error handling code        |
| 10   | Output handling (`UserOutput`) | `handle-user-output`           | Writing user-facing output         |
| 11   | Expected errors/known issues   | `debug-test-errors`            | Debugging test output              |
| 13   | Module organization            | `organize-rust-modules`        | Structuring Rust modules           |
| 15   | Markdown pitfalls              | `write-markdown-docs`          | Writing `.md` files                |
| 16   | Environment variable naming    | `create-environment-variables` | Adding env vars                    |
| 17   | Adding new templates           | `add-new-template`             | Adding to `templates/`             |
| 19   | Secret handling                | `handle-secrets`               | Working with credentials/tokens    |

### Category 3: Keep as Slim Rules (2 Rules)

These rules could stay in AGENTS.md but should be significantly condensed.

| Rule | Topic                   | Rationale                                                                                                                            |
| ---- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | `envs/` vs `data/`      | Universal safety constraint (never write to `data/`). Currently ~500 tokens — slim to ~50 tokens with a skill reference for details. |
| 14   | Rust import conventions | Applies to nearly all code changes. Slim to ~30 tokens.                                                                              |

### Proposed Slim Rules (After Refactoring)

The Essential Rules section would be reduced to approximately:

```markdown
## Essential Rules

1. **CRITICAL - `data/` is READ ONLY**: Never create/edit files in `data/` — it contains
   application-managed state. User configs go in `envs/`. These are completely different
   JSON structures with different purposes.

2. **Rust imports**: Imports always at top of file (std → external crates → internal crate).
   Prefer short imported names over fully-qualified paths.
```

## Implementation Plan

### Phase 1: Remove Duplicates

- [ ] Remove rules 4, 5, 6, 12, 18, 20 from `AGENTS.md` Essential Rules
- [ ] Verify the existing skills fully cover the removed content
- [ ] Update rule numbering

### Phase 2: Create New Skills (batch 1 — code conventions)

- [ ] Create `place-code-in-ddd-layers` skill (from rule 2)
- [ ] Create `implement-domain-types` skill (from rule 3)
- [ ] Create `handle-errors-in-code` skill (from rule 9)
- [ ] Create `handle-user-output` skill (from rule 10)
- [ ] Create `organize-rust-modules` skill (from rule 13)
- [ ] Create `handle-secrets` skill (from rule 19)

### Phase 3: Create New Skills (batch 2 — templates and infra)

- [ ] Create `work-with-tera-templates` skill (from rule 7)
- [ ] Create `add-ansible-playbook` skill (from rule 8)
- [ ] Create `add-new-template` skill (from rule 17)
- [ ] Create `create-environment-variables` skill (from rule 16)

### Phase 4: Create New Skills (batch 3 — docs and debugging)

- [ ] Create `write-markdown-docs` skill (from rule 15)
- [ ] Create `debug-test-errors` skill (from rule 11)

### Phase 5: Slim Down Remaining Rules and Update AGENTS.md

- [ ] Condense rule 1 (`envs/` vs `data/`) to ~50 tokens
- [ ] Condense rule 14 (Rust imports) to ~30 tokens
- [ ] Remove rules 2, 3, 7, 8, 9, 10, 11, 13, 15, 16, 17, 19 from Essential Rules
- [ ] Update the Auto-Invoke Skills table in AGENTS.md with all new skills
- [ ] Update the skill count references in documentation

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Essential Rules section reduced to 2 concise rules (~200 tokens)
- [ ] 6 duplicate rules removed without information loss
- [ ] 12 new skills created following Agent Skills spec format (YAML frontmatter + Markdown body)
- [ ] Each new skill contains step-by-step instructions, not just "read this doc" pointers
- [ ] All new skill names match their parent directory names (spec requirement)
- [ ] All new skill descriptions are 1-1024 characters and include trigger keywords
- [ ] Skills table in AGENTS.md updated with all new entries
- [ ] No existing skill is broken or modified in behavior
- [ ] Progressive disclosure validated: startup context reduced, on-demand content preserved

## Related Documentation

- [Agent Skills Specification](https://agentskills.io/specification) — the format standard
- [AGENTS.md](../../AGENTS.md) — current rules that will be refactored
- [Epic: Adopt Agent Skills Specification](https://github.com/torrust/torrust-tracker-deployer/issues/274) — parent epic
- [Add New Skill guide](.github/skills/add-new-skill/skill.md) — skill creation workflow

## Notes

- The skill creation should follow the existing `add-new-skill` skill workflow for consistency
- Each new skill should reference the corresponding `docs/contributing/` documentation as reference material rather than duplicating it — use the progressive disclosure pattern (SKILL.md body for quick instructions, `references/` for detailed docs)
- This can be implemented incrementally: Phase 1 (remove duplicates) is a quick win that can be done independently
- The token savings are significant: ~3000 tokens freed from every conversation, with the same information available on-demand through skills
