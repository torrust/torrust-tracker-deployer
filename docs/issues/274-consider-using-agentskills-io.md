# Consider Using agentskills.io

**Issue**: [#274](https://github.com/torrust/torrust-tracker-deployer/issues/274)
**Type**: Feature / Enhancement
**Status**: 🔄 Open

## Overview

[agentskills.io](https://agentskills.io) is an open format for extending AI agent capabilities with specialized knowledge and workflows. This issue proposes adopting the Agent Skills specification **as a supplement to our existing `AGENTS.md`** - not a replacement.

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

## Goals

- [ ] Evaluate which workflows would benefit from skills
- [ ] Create initial set of skills for common workflows
- [ ] Add an "Auto-Invoke Skills" section to AGENTS.md
- [ ] Validate skills using the official `skills-ref` tool

## Proposed Skills for Tracker Deployer

Based on analyzing our `AGENTS.md` and common workflows, here are the high-value skill candidates:

### Skill 1: `deploy-tracker` (High Priority)

**Trigger phrases**: "deploy tracker", "provision environment", "create LXD instance", "full deployment"

**Why a skill?**: This is a complex multi-step workflow (create → provision → configure → release → run → test) that requires specific command sequences, environment config generation, and validation steps.

**Would include**:

- Step-by-step deployment workflow
- Environment config template in `assets/`
- Scripts for validation checks
- Common troubleshooting steps

```yaml
---
name: deploy-tracker
description: |
  Deploy a Torrust Tracker to LXD or Hetzner. Complete workflow from 
  environment creation to running services. Triggers when user says 
  "deploy tracker", "provision environment", "full deployment", or 
  "create LXD instance".
---
```

### Skill 2: `create-environment-config` (High Priority)

**Trigger phrases**: "create config", "generate environment JSON", "setup deployment config"

**Why a skill?**: Environment configs have complex validation rules (see rule 20 in AGENTS.md). AI agents often hallucinate invalid configurations. A skill can bundle the schema, examples, and validation constraints.

**Would include**:

- JSON schema reference
- Example configs for SQLite, MySQL, LXD, Hetzner
- Rust type constraints (NonZeroU32, tagged enums)
- Common configuration patterns

```yaml
---
name: create-environment-config
description: |
  Generate valid environment configuration JSON files for the deployer.
  Triggers when user needs to create deployment configs or asks about 
  environment settings. Always validates against Rust type constraints.
---
```

### Skill 3: `add-feature-to-deployer` (High Priority)

**Trigger phrases**: "add command", "implement feature", "extend deployer", "add new step"

**Why a skill?**: This is our most complex workflow. It requires understanding DDD layers (rule 2), template system (rule 17), error handling (rule 9), output handling (rule 10), and testing conventions (rule 18). Currently all in AGENTS.md as rules.

**Would include**:

- DDD layer decision flowchart
- Module structure templates
- Error type boilerplate
- Test naming examples
- References to ADRs

```yaml
---
name: add-feature-to-deployer
description: |
  Add new features to the deployer following DDD architecture. 
  Includes command handlers, steps, actions, templates, and tests.
  Triggers when user says "add command", "implement feature", 
  "extend deployer", or "create new step".
allowed-tools:
  - Read
  - Write
---
```

### Skill 4: `troubleshoot-deployment` (Medium Priority)

**Trigger phrases**: "deployment failed", "tracker not starting", "debug deployment", "SSH error"

**Why a skill?**: Troubleshooting follows patterns. Common issues (SSH key problems, port conflicts, Docker not ready) have known solutions. A skill can guide systematic diagnosis.

**Would include**:

- Diagnostic command sequences
- Common error patterns and fixes
- Log inspection steps
- Known issues reference

```yaml
---
name: troubleshoot-deployment
description: |
  Diagnose and fix deployment issues. Covers SSH problems, Docker 
  failures, port conflicts, and service startup issues. Triggers 
  when deployment fails or services are not responding.
---
```

### Skill 5: `git-pr-workflow` (Medium Priority)

**Trigger phrases**: "create PR", "ready for review", "commit changes", "prepare branch"

**Why a skill?**: Rules 4, 5, 6 in AGENTS.md cover branching, commits, and pre-commit checks. A skill can bundle these into a single workflow with templates.

**Would include**:

- Branch naming template
- Commit message format
- Pre-commit checklist
- PR body template

```yaml
---
name: git-pr-workflow
description: |
  Create PRs following project conventions. Handles branch naming,
  conventional commits, pre-commit checks, and PR formatting.
  Triggers when user says "create PR", "commit changes", or 
  "ready for review".
---
```

### Skill 6: `write-issue-spec` (Medium Priority)

**Trigger phrases**: "create issue spec", "write issue specification", "document issue"

**Why a skill?**: We have templates in `docs/issues/` (SPECIFICATION-TEMPLATE.md, EPIC-TEMPLATE.md). A skill can guide using the correct template and filling it properly.

**Would include**:

- Template selection logic
- Example specs
- DDD layer guidance for architecture section

```yaml
---
name: write-issue-spec
description: |
  Create detailed issue specifications using project templates.
  Guides through SPECIFICATION-TEMPLATE.md or EPIC-TEMPLATE.md.
  Triggers when user needs to document a feature or task.
---
```

### Skill 7: `run-e2e-tests` (Lower Priority)

**Trigger phrases**: "run E2E tests", "test deployment", "verify changes"

**Why a skill?**: E2E testing has specific requirements (container setup, timeouts, expected warnings). A skill can guide the process and interpret results.

**Would include**:

- Test selection guidance
- Expected warning patterns (SSH host key warnings)
- Troubleshooting failed tests

```yaml
---
name: run-e2e-tests
description: |
  Run end-to-end tests for the deployer. Handles infrastructure 
  lifecycle tests and deployment workflow tests. Interprets results
  including expected warnings. Triggers when user says "run E2E tests"
  or "verify changes".
---
```

## Recommended AGENTS.md Addition

Following the best practice from the articles, add an "Auto-Invoke Skills" section:

```markdown
## Auto-invoke Skills

When performing these tasks, automatically load the corresponding skill:

| Task                         | Skill to Load                                       |
| ---------------------------- | --------------------------------------------------- |
| Deploying a tracker          | `.github/skills/deploy-tracker/SKILL.md`            |
| Creating environment configs | `.github/skills/create-environment-config/SKILL.md` |
| Adding new features          | `.github/skills/add-feature-to-deployer/SKILL.md`   |
| Troubleshooting deployments  | `.github/skills/troubleshoot-deployment/SKILL.md`   |
| Creating PRs                 | `.github/skills/git-pr-workflow/SKILL.md`           |
| Writing issue specs          | `.github/skills/write-issue-spec/SKILL.md`          |
| Running E2E tests            | `.github/skills/run-e2e-tests/SKILL.md`             |
```

## Implementation Plan

### Phase 1: Setup and First Skill (2 hours)

- [ ] Create `.github/skills/` directory structure
- [ ] Enable `chat.useAgentSkills` setting in VS Code
- [ ] Create `deploy-tracker` skill (highest value, most repeated workflow)
- [ ] Test with VS Code Copilot

### Phase 2: Config and Feature Skills (3 hours)

- [ ] Create `create-environment-config` skill with schema references
- [ ] Create `add-feature-to-deployer` skill with DDD guidance
- [ ] Add assets (templates, scripts) to skills

### Phase 3: Support Skills (2 hours)

- [ ] Create `troubleshoot-deployment` skill
- [ ] Create `git-pr-workflow` skill
- [ ] Create `write-issue-spec` skill

### Phase 4: Integration and Validation (1 hour)

- [ ] Add "Auto-Invoke Skills" section to AGENTS.md
- [ ] Install `skills-ref` validation tool
- [ ] Validate all created skills
- [ ] Test with different AI agents
- [ ] Document findings

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All skills validate with `skills-ref validate ./skill-name`

**Task-Specific Criteria**:

- [ ] At least 3 skills created and validated
- [ ] Skills follow progressive disclosure (under 500 lines, under 5000 tokens)
- [ ] Auto-invoke table added to AGENTS.md
- [ ] Skills tested with VS Code Copilot

## Related Documentation

- [agentskills.io Specification](https://agentskills.io/specification)
- [What are Skills?](https://agentskills.io/what-are-skills)
- [Agent Skills Getting Started (Aridane Martín)](https://aridanemartin.dev/blog/agent-skills-getting-started/)
- [Agent Skills Best Practices (Aridane Martín)](https://aridanemartin.dev/blog/agent-skills-best-practices-patterns-use-cases/)
- [Example Skills Repository](https://github.com/anthropics/skills)
- [skills-ref Validation Tool](https://github.com/agentskills/agentskills/tree/main/skills-ref)
- [VS Code Skills Documentation](https://code.visualstudio.com/docs/copilot/copilot-customization)
- [Current AGENTS.md](../../AGENTS.md)

## Notes

- Skills are **supplementary** to AGENTS.md, not a replacement
- Our AGENTS.md remains the "always-on" rules file
- Skills handle complex multi-step workflows that we explain repeatedly
- Progressive disclosure means skills only load when relevant, saving tokens
- Skills can bundle scripts and templates, not just instructions
- The "routing-grade" description is critical - it determines when the skill activates
