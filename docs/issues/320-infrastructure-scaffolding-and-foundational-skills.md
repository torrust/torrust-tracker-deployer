# Infrastructure Scaffolding and Foundational Skills

**Issue**: #320
**Parent Epic**: #274 - Adopt Agent Skills Specification
**Related**: [Agent Skills Specification](https://agentskills.io/specification)

## Overview

This issue sets up the complete infrastructure for Agent Skills and creates two foundational skills. This includes directory structure, integration tooling, validation setup, and documentation.

**Two foundational skills**:

1. **`run-linters`**: Simple, frequently-used workflow to validate the format
2. **`add-new-skill`**: Meta-skill documenting how to create new skills (makes the system self-documenting)

**Why these two?**: `run-linters` proves the basic concept with a simple workflow, while `add-new-skill` enables easy addition of future skills without returning to this documentation.

## Goals

### Infrastructure

- [x] Create `.github/skills/` directory structure
- [ ] Install and configure `skills-ref` validation tool
- [ ] Document VS Code configuration for Agent Skills (`chat.useAgentSkills` setting)
- [x] Add "Auto-Invoke Skills" section to AGENTS.md

### Skills

- [x] Create `run-linters` skill with proper frontmatter and instructions
- [x] Create `add-new-skill` meta-skill
- [ ] Validate both skills using `skills-ref` tool

### Testing & Validation

- [ ] Test skills with GitHub Copilot
- [ ] Verify skills activate on trigger phrases
- [ ] Ensure all validations pass

## Specifications

### Infrastructure Directory Structure

Create the following structure:

```text
.github/
└── skills/
    ├── run-linters/
    │   ├── skill.md          # Main skill file
    │   └── references/       # Optional: detailed linter docs
    │       └── linters.md    # Reference for all available linters
    └── add-new-skill/
        ├── skill.md          # Main skill file
        └── references/       # Optional: detailed documentation
            ├── specification.md  # Agent Skills specification reference
            └── examples.md       # Example skills
```

### Skill 1: `run-linters`

The `run-linters` skill documents how to run the project's comprehensive linting system.

**skill.md Frontmatter**:

```yaml
---
name: run-linters
description: |
  Run code quality checks and linters for the deployer project. Includes
  markdown, YAML, TOML, spell checking, Rust clippy, rustfmt, and shellcheck.
  Use when user needs to lint code, check formatting, fix code quality issues,
  or prepare for commit. Triggers on "lint", "run linters", "check code quality",
  "fix formatting", or "run pre-commit checks".
metadata:
  author: torrust
  version: "1.0"
---
```

**Body sections**:

1. **When to Use This Skill**
   - Before committing code (mandatory)
   - After making code changes
   - When fixing CI failures
   - When code quality issues are reported

2. **Available Linters**
   - All linters: `cargo run --bin linter all`
   - Individual linters: markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck
   - Alternative wrapper: `./scripts/lint.sh`

3. **Common Workflows**
   - Run all linters before commit
   - Fix specific linter errors
   - Run only fast linters during development

4. **Fixing Common Issues**
   - Markdown formatting issues
   - YAML syntax errors
   - Spelling mistakes (project-words.txt)
   - Rust formatting and clippy warnings
   - Shell script issues

5. **Integration with Development Workflow**
   - Pre-commit hook: `./scripts/pre-commit.sh`
   - CI pipeline requirements
   - When linting checks are mandatory

**Optional Reference File**: `references/linters.md`

Create detailed information about each linter:

- Markdown linting (.markdownlint.json)
- YAML linting (.yamllint-ci.yml)
- TOML linting (.taplo.toml)
- Spell checking (cspell.json, project-words.txt)
- Rust linting (clippy, rustfmt)
- Shell script linting (shellcheck)

This file is loaded on-demand when the agent needs detailed linter configuration.

### Skill 2: `add-new-skill`

Meta-skill that documents how to create new Agent Skills following the specification. Modeled after Anthropic's [skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator).

**skill.md Frontmatter**:

```yaml
---
name: add-new-skill
description: |
  Guide for creating effective Agent Skills for the torrust-tracker-deployer project.
  Use when you need to create a new skill (or update an existing skill) that extends
  AI agent capabilities with specialized knowledge, workflows, or tool integrations.
  Triggers on "create skill", "add new skill", "how to add skill", or "skill creation".
metadata:
  author: torrust
  version: "1.0"
---
```

**Body sections** (following Anthropic's proven structure):

1. **About Skills**
   - What Agent Skills are and what they provide
   - Progressive disclosure concept (3-level loading: metadata → body → resources)
   - When to create a skill vs updating AGENTS.md

2. **Core Principles**
   - **Concise is Key**: Context window efficiency, assume Claude is smart, challenge each piece of information
   - **Set Appropriate Degrees of Freedom**: Match specificity to task fragility (high/medium/low freedom)
   - **Anatomy of a Skill**: skill.md (frontmatter + body) + optional bundled resources
   - **Progressive Disclosure**: Keep skill.md under 500 lines, split content into reference files

3. **Skill Creation Process** (7 steps)
   - **Step 1**: Understanding the skill with concrete examples (what queries should trigger it?)
   - **Step 2**: Planning reusable skill contents (scripts, references, assets)
   - **Step 3**: Creating the skill directory structure in `.github/skills/skill-name/`
   - **Step 4**: Writing skill.md (frontmatter with name + description, body with instructions)
   - **Step 5**: Adding bundled resources (scripts/, references/, assets/) if needed
   - **Step 6**: Validation using `skills-ref validate .github/skills/skill-name`
   - **Step 7**: Testing with GitHub Copilot and iteration based on usage

4. **skill.md Frontmatter**
   - Required: `name` (lowercase, hyphens, 1-64 chars), `description`
   - Description must include BOTH what the skill does AND when to use it
   - Include all trigger phrases in description (body is only loaded after triggering)
   - Optional: `metadata`, `allowed-tools`, `compatibility`

5. **skill.md Body**
   - Use imperative/infinitive form
   - Keep core workflow and selection guidance in skill.md
   - Move variant-specific details to references/
   - Link to reference files clearly ("See [reference.md](references/reference.md) for details")
   - Progressive disclosure patterns: high-level guide, domain-specific organization, conditional details

6. **Bundled Resources**
   - `scripts/`: Executable code (Python/Bash) for deterministic, repeatedly-rewritten tasks
   - `references/`: Documentation loaded on-demand (schemas, API docs, detailed guides)
   - `assets/`: Files used in output (templates, images, boilerplate code)
   - What NOT to include: readme.md, changelog.md, auxiliary documentation

7. **Validation and Integration**
   - Validate: `skills-ref validate .github/skills/skill-name`
   - Test with GitHub Copilot using various trigger phrases
   - Add to AGENTS.md "Auto-Invoke Skills" table
   - Document in `docs/user-guide/vscode-skills-setup.md` if needed

**Reference Files** (to be created):

- `references/specification.md` - Full Agent Skills spec from agentskills.io
- `references/patterns.md` - Proven design patterns (workflows, output patterns, multi-step processes)
- `references/examples.md` - Example skills from this project and Anthropic's repository

### Validation Tools

**Install skills-ref tool**:

```bash
pip install agentskills
# or
pipx install agentskills
```

**Validate skills**:

```bash
# Validate single skill
skills-ref validate .github/skills/run-linters

# Validate all skills
skills-ref validate .github/skills/*
```

### VS Code Configuration

Document the `chat.useAgentSkills` setting in a new file or update existing documentation:

**File**: `docs/user-guide/vscode-skills-setup.md` (new)

```markdown
# VS Code Agent Skills Setup

To enable Agent Skills support in VS Code with GitHub Copilot:

1. Open VS Code settings (Cmd/Ctrl + ,)
2. Search for "chat.useAgentSkills"
3. Enable the setting
4. Reload VS Code window

Skills are automatically discovered from the `.github/skills/` directory.

## Verifying Skills are Loaded

When you start a Copilot chat, skills should be available. You can test by asking:

**For `run-linters`**:

- "How do I run linters?"
- "Check code quality"
- "Fix formatting issues"

**For `add-new-skill`**:

- "How do I add a new skill?"
- "Create agent skill"
- "Add skill documentation"

Copilot should automatically load the appropriate skill and provide specific instructions.
```

### AGENTS.md Integration

Add the following section to `AGENTS.md` after creating the skills:

```markdown
## Auto-invoke Skills

When performing these tasks, automatically load the corresponding skill:

| Task              | Skill to Load                           |
| ----------------- | --------------------------------------- |
| Running linters   | `.github/skills/run-linters/skill.md`   |
| Adding new skills | `.github/skills/add-new-skill/skill.md` |
```

## Implementation Plan

### Step 1: Infrastructure Setup (20 minutes)

- [x] Create `.github/skills/` directory
- [x] Create skill subdirectories: `run-linters/` and `add-new-skill/`
- [ ] Install `skills-ref` tool: `pip install agentskills` or `pipx install agentskills`
- [ ] Create `docs/user-guide/vscode-skills-setup.md` documentation

### Step 2: Create `run-linters` Skill (30 minutes)

- [x] Create `.github/skills/run-linters/skill.md` with proper frontmatter
- [x] Write "When to Use This Skill" section
- [x] Document all available linters
- [x] Add common workflows section
- [x] Include troubleshooting for common issues
- [x] Link to existing documentation ([docs/linting.md](../linting.md))
- [x] Create `references/linters.md` with detailed linter information (optional)

### Step 3: Create `add-new-skill` Skill (40 minutes)

Following Anthropic's [skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator) pattern:

- [x] Create `.github/skills/add-new-skill/skill.md` with proper frontmatter
- [x] Write "About Skills" section (what they are, progressive disclosure)
- [x] Document "Core Principles" (concise, degrees of freedom, anatomy, progressive disclosure, content strategy)
- [x] Write "Skill Creation Process" (7 steps: understand → plan → create → write → add resources → validate → iterate)
- [x] Include frontmatter requirements (name + description with triggers)
- [x] Document body content guidelines (imperative form, progressive disclosure patterns)
- [x] Describe bundled resources (scripts/, references/, assets/)
- [x] Add validation and integration instructions
- [x] Create reference files: specification.md, patterns.md, examples.md (optional)

### Step 4: Validation (25 minutes)

- [ ] Validate `run-linters`: `skills-ref validate .github/skills/run-linters`
- [ ] Validate `add-new-skill`: `skills-ref validate .github/skills/add-new-skill`
- [ ] Fix any validation errors
- [ ] Ensure names match directory names
- [ ] Verify descriptions are under 1024 characters
- [ ] Check all frontmatter fields are valid YAML

### Step 5: Integration and Testing (25 minutes)

- [ ] Add "Auto-Invoke Skills" section to AGENTS.md
- [ ] Create `docs/user-guide/vscode-skills-setup.md` documentation
- [ ] Configure VS Code: enable `chat.useAgentSkills` setting
- [ ] Test `run-linters` skill with GitHub Copilot
- [ ] Test `add-new-skill` skill with GitHub Copilot
- [ ] Verify both skills activate on trigger phrases
- [ ] Document findings and any issues

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Both skills validate successfully: `skills-ref validate .github/skills/*`

**Infrastructure Criteria**:

- [x] `.github/skills/` directory exists
- [ ] `skills-ref` tool installed and functional
- [ ] `docs/user-guide/vscode-skills-setup.md` created with configuration instructions
- [x] "Auto-Invoke Skills" section added to AGENTS.md

**Skill 1: `run-linters` Criteria**:

- [x] `.github/skills/run-linters/skill.md` exists with valid frontmatter
- [x] Skill name matches directory name: `run-linters`
- [x] Description is clear, under 1024 chars, includes trigger phrases
- [x] Body content is under 500 lines and 5000 tokens (progressive disclosure)
- [x] Skill includes all required sections (when to use, workflows, common issues)
- [x] References to existing documentation are correct
- [ ] Skill tested with GitHub Copilot and activates correctly

**Skill 2: `add-new-skill` Criteria**:

- [x] `.github/skills/add-new-skill/skill.md` exists with valid frontmatter
- [x] Skill name matches directory name: `add-new-skill`
- [x] Description is clear, under 1024 chars, includes trigger phrases
- [x] Follows Anthropic's skill-creator pattern structure
- [x] Includes "About Skills", "Core Principles", and "Skill Creation Process" sections
- [x] Documents 7-step creation process
- [x] Covers frontmatter requirements, body guidelines, and bundled resources
- [x] Includes validation and testing instructions
- [x] References Agent Skills specification and Anthropic examples
- [ ] Skill tested with GitHub Copilot and activates correctly

**Validation Requirements**:

- [ ] Both skills pass `skills-ref validate` with no errors
- [ ] Frontmatter YAML is valid for both skills
- [ ] Names follow naming rules (lowercase, hyphens only, 1-64 chars)
- [ ] All file references use relative paths from skill root

## Related Documentation

- [agentskills.io Specification](https://agentskills.io/specification)
- [What are Skills?](https://agentskills.io/what-are-skills)
- [How to Create Custom Skills (Claude)](https://support.claude.com/en/articles/12512198-how-to-create-custom-skills) - Official Claude documentation
- [Skill Authoring Best Practices (Claude Platform)](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices) - Comprehensive guide to writing effective skills
- [Anthropic skill-creator Example](https://github.com/anthropics/skills/tree/main/skills/skill-creator) - Reference implementation
- [skills-ref Validation Tool](https://github.com/agentskills/agentskills/tree/main/skills-ref)
- [VS Code Skills Documentation](https://code.visualstudio.com/docs/copilot/copilot-customization)
- [Current Linting Documentation](../linting.md)
- [Linting Guide](../contributing/linting.md)
- [Pre-commit Process](../contributing/commit-process.md)

## Notes

**Progressive Disclosure**: Main `skill.md` files should be concise (<500 lines, <5000 tokens). Detailed information goes in `references/` and is loaded only when the agent needs it:

- `run-linters`: Detailed linter configuration in `references/linters.md`
- `add-new-skill`: Full specification and examples in `references/`

**Testing**: After creating both skills, test them by asking Copilot various questions:

**For `run-linters`**:

- "How do I run linters?"
- "Fix my markdown formatting"
- "What linters are available?"
- "Run code quality checks"

**For `add-new-skill`**:

- "How do I create a new skill?"
- "What's the skill format?"
- "Add agent skill for deployment"
- "Guide me through creating a skill"

Both skills should activate automatically based on their description triggers.

**Self-Documenting System**: The `add-new-skill` meta-skill makes the system self-documenting. It follows Anthropic's proven [skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator) pattern, which has been validated through extensive use. Once created, future skills can be added easily by AI agents using this skill as a guide, without requiring developers to remember or look up the specification details.

**On-Demand Expansion**: After validating the infrastructure with these two foundational skills, additional skills (environment configs, feature development, deployments, etc.) can be added on-demand as separate issues when workflows become repetitive or complex.
