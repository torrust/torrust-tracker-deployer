# EPIC: Adopt Agent Skills Specification

**Issue**: [#274](https://github.com/torrust/torrust-tracker-deployer/issues/274)
**Type**: Epic
**Status**: 🔄 Open

## Overview

[agentskills.io](https://agentskills.io) is an open format for extending AI agent capabilities with specialized knowledge and workflows. This epic tracks the adoption of the Agent Skills specification **as a supplement to our existing `AGENTS.md`** - not a replacement.

## Key Insight: Skills vs Rules

After research, the relationship is clear:

| Aspect          | AGENTS.md (Rules)                 | Skills (SKILL.md)               |
| --------------- | --------------------------------- | ------------------------------- |
| **Loading**     | Always-on, every conversation     | On-demand, when task matches    |
| **Purpose**     | Baseline conventions, constraints | Specialist workflows, templates |
| **Token usage** | Constant overhead                 | Only when needed                |
| **Best for**    | "Always do X, never do Y"         | Multi-step repeatable workflows |
| **Updates**     | Rarely changes                    | Can be added/refined frequently |

**Our `AGENTS.md` is excellent for rules** - it has 20 essential rules that should apply to every AI interaction. **Skills would handle the complex workflows** that we find ourselves explaining repeatedly.

## What is agentskills.io?

Agent Skills is a lightweight, open format developed by Anthropic and adopted by multiple AI coding agents including Claude Code, VS Code Copilot, Cursor, Windsurf, and others.

A skill is a folder containing a `SKILL.md` file with:

- **Frontmatter**: `name` and `description` (required), plus optional `allowed-tools`, `triggers`, `version`
- **Body**: Markdown instructions for performing specific tasks
- **Optional directories**: `scripts/`, `references/`, `assets/`

### How Skills Work (Progressive Disclosure)

1. **Discovery**: At startup, agents load only the `name` and `description` of each skill (~100 tokens)
2. **Activation**: When a task matches a skill's description, the agent reads the full `SKILL.md` (<5000 tokens recommended)
3. **Execution**: The agent follows instructions, loading referenced files or executing scripts as needed

### Skill Structure

```text
skill-name/
├── SKILL.md          # Required: instructions + metadata
├── scripts/          # Optional: executable code (Python, Bash, JS)
├── references/       # Optional: documentation loaded on demand
└── assets/           # Optional: templates, images, data files
```

### SKILL.md Format

```yaml
---
name: skill-name
description: |
  What this skill does and when to use it.
  Triggers when user says "deploy tracker" or "provision environment".
allowed-tools:
  - Read
  - Bash(cargo build)
---

## Instructions

Step-by-step guidance for the agent.

## When to Use This Skill

- Trigger condition 1
- Trigger condition 2

## Workflow

1. First, do X
2. Then check Y
3. Finally, output Z
```

## Approach

This epic tracks the adoption of Agent Skills on an **on-demand basis**. Rather than committing to implement all potential skills upfront, we:

1. **Start with scaffolding** (#320): Set up infrastructure, tooling, and create two foundational skills
2. **Add skills as needed**: Create new skills when workflows become repetitive or complex
3. **Evolve organically**: The list of potential skills may change based on actual needs
4. **Close when mature**: Eventually close this epic and add new skills independently

## Tasks

### Completed

- [ ] #320 - Infrastructure Scaffolding and Foundational Skills (`run-linters` + `add-new-skill`)

### Future Skills (On-Demand)

These may be implemented as separate issues when needed:

- `create-environment-config` - Generate valid environment configurations
- `add-feature-to-deployer` - Add features following DDD architecture
- `deploy-tracker` - Full deployment workflow guide
- `troubleshoot-deployment` - Diagnose deployment issues
- `git-pr-workflow` - Create PRs following conventions
- `write-issue-spec` - Document features using templates
- `run-e2e-tests` - Run and interpret E2E test results

## Goals

The goals of this epic are to:

- [ ] Set up Agent Skills infrastructure and tooling
- [ ] Create foundational skills (`run-linters` and `add-new-skill`)
- [ ] Establish patterns for creating new skills
- [ ] Add an "Auto-Invoke Skills" section to AGENTS.md
- [ ] Validate all skills using the official `skills-ref` tool

## Potential Skills for Tracker Deployer

Based on analyzing our `AGENTS.md` and common workflows, here are potential skill candidates. These will be implemented **on-demand** when we identify the need:

### Foundational Skills (Issue #275)

#### `run-linters`

**Purpose**: Document how to run the project's comprehensive linting system.

**Why first?**: Simple, well-defined, frequently used workflow that validates the Agent Skills format.

#### `add-new-skill`

**Purpose**: Meta-skill that documents how to create new skills following the Agent Skills specification.

**Why included?**: Makes the system self-documenting and enables easy addition of future skills.

### Future Skills (Implement On-Demand)

#### `create-environment-config`

**Purpose**: Generate valid environment configuration JSON files.

**Value**: Environment configs have complex validation rules. AI agents often hallucinate invalid configurations.

#### `add-feature-to-deployer`

**Purpose**: Guide adding new features following DDD architecture, template system, error handling, and testing conventions.

**Value**: Most complex workflow requiring understanding of multiple AGENTS.md rules.

#### `deploy-tracker`

**Purpose**: Complete multi-step deployment workflow (create → provision → configure → release → run → test).

**Value**: Complex workflow requiring specific command sequences and validation steps.

#### `troubleshoot-deployment`

**Purpose**: Diagnose and fix deployment issues systematically.

**Value**: Common issues have known solutions; skill guides systematic diagnosis.

#### `git-pr-workflow`

**Purpose**: Create PRs following project conventions (branching, commits, pre-commit checks).

**Value**: Bundles rules 4, 5, 6 from AGENTS.md into single workflow.

#### `write-issue-spec`

**Purpose**: Create issue specifications using project templates.

**Value**: Guides proper use of SPECIFICATION-TEMPLATE.md or EPIC-TEMPLATE.md.

#### `run-e2e-tests`

**Purpose**: Run and interpret E2E test results, including expected warnings.

**Value**: E2E testing has specific requirements and expected warning patterns.

### Skill 5: `troubleshoot-deployment` (Phase 3 - Medium Priority)

## Recommended AGENTS.md Addition

Following the best practice, add an "Auto-Invoke Skills" section to AGENTS.md as skills are created:

```markdown
## Auto-invoke Skills

When performing these tasks, automatically load the corresponding skill:

| Task                         | Skill to Load                                       |
| ---------------------------- | --------------------------------------------------- |
| Running linters              | `.github/skills/run-linters/SKILL.md`               |
| Adding new skills            | `.github/skills/add-new-skill/SKILL.md`             |
| Creating environment configs | `.github/skills/create-environment-config/SKILL.md` |
| Adding new features          | `.github/skills/add-feature-to-deployer/SKILL.md`   |
| Deploying a tracker          | `.github/skills/deploy-tracker/SKILL.md`            |
| Troubleshooting deployments  | `.github/skills/troubleshoot-deployment/SKILL.md`   |
| Creating PRs                 | `.github/skills/git-pr-workflow/SKILL.md`           |
| Writing issue specs          | `.github/skills/write-issue-spec/SKILL.md`          |
| Running E2E tests            | `.github/skills/run-e2e-tests/SKILL.md`             |
```

## Epic Acceptance Criteria

**Quality Checks**:

- [ ] All skills validate with `skills-ref validate ./skill-name`
- [ ] Pre-commit checks pass for all changes

**Epic-Level Criteria**:

- [ ] Infrastructure and tooling set up
- [ ] Foundational skills created and validated (`run-linters`, `add-new-skill`)
- [ ] Auto-invoke table added to AGENTS.md
- [ ] Skills follow progressive disclosure (under 500 lines, under 5000 tokens)
- [ ] Skills tested with VS Code Copilot
- [ ] All skills follow progressive disclosure (under 500 lines, under 5000 tokens)
- [ ] Auto-invoke table added to AGENTS.md
- [ ] Skills tested with VS Code Copilot
- [ ] Documentation updated with skills usage guide

## Related Documentation

- [agentskills.io Specification](https://agentskills.io/specification)
- [What are Skills?](https://agentskills.io/what-are-skills)
- [Agent Skills Getting Started (Aridane Martín)](https://aridanemartin.dev/blog/agent-skills-getting-started/)
- [Agent Skills Best Practices (Aridane Martín)](https://aridanemartin.dev/blog/agent-skills-best-practices-patterns-use-cases/)
- [Example Skills Repository](https://github.com/anthropics/skills)
- [skills-ref Validation Tool](https://github.com/agentskills/agentskills/tree/main/skills-ref)
- [VS Code Skills Documentation](https://code.visualstudio.com/docs/copilot/copilot-customization)
- [Current AGENTS.md](../../AGENTS.md)

## Related Issues

**Child Tasks**:

- #275 - Phase 1: Setup and First Skill (`run-linters`)

## Notes

- Skills are **supplementary** to AGENTS.md, not a replacement
- Our AGENTS.md remains the "always-on" rules file
- Skills handle complex multi-step workflows that we explain repeatedly
- Progressive disclosure means skills only load when relevant, saving tokens
- Skills can bundle scripts and templates, not just instructions
- The "routing-grade" description is critical - it determines when the skill activates
