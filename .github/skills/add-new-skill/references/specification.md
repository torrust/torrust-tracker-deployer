# Agent Skills Specification Reference

This document provides a comprehensive reference to the Agent Skills specification from [agentskills.io](https://agentskills.io).

## What is Agent Skills?

Agent Skills is an open format for extending AI agent capabilities with specialized knowledge and workflows. It's vendor-neutral and works with multiple AI coding agents including Claude Code, VS Code Copilot, Cursor, and Windsurf.

## Core Concepts

### Progressive Disclosure

Skills use a three-level loading strategy to minimize context window usage:

```text
Level 1: Metadata (name + description) - ~100 tokens - Loaded at startup for ALL skills
Level 2: skill.md body - <5000 tokens - Loaded when skill matches task
Level 3: Bundled resources - On-demand - Loaded only when referenced
```

**Why this matters**: Only relevant content is loaded into the AI agent's context window.

### Directory Structure

```text
.github/
└── skills/
    └── skill-name/
        ├── skill.md          # Required: frontmatter + instructions
        ├── README.md         # Optional: human-readable documentation
        ├── scripts/          # Optional: executable code
        ├── references/       # Optional: detailed docs loaded on-demand
        └── assets/           # Optional: templates, images, data
```

## skill.md Format

### Frontmatter (YAML)

Required fields:

```yaml
---
name: skill-name
description: |
  What the skill does and when to use it.
  Must include trigger phrases/keywords.
---
```

Optional fields:

```yaml
---
name: skill-name
description: |
  Skill description...
metadata:
  author: organization-name
  version: "1.0"
  last-updated: "2026-02-06"
allowed-tools:
  - Read
  - Bash(cargo build)
  - Bash(python scripts/validate.py)
compatibility:
  agents:
    - claude-code
    - github-copilot
    - cursor
    - windsurf
triggers:
  - "deploy tracker"
  - "provision environment"
---
```

### Frontmatter Validation Rules

**name**:

- Required
- Maximum 64 characters
- Must contain only lowercase letters, numbers, and hyphens
- Cannot contain XML tags (`<`, `>`)
- Cannot contain reserved words: "anthropic", "claude"

**description**:

- Required
- Maximum 1024 characters
- Must be non-empty
- Cannot contain XML tags
- Should describe WHAT the skill does AND WHEN to use it
- Should include trigger phrases

### Body (Markdown)

The body contains the actual skill instructions in Markdown format:

```markdown
---
name: example-skill
description: Example skill for demonstration
---

# Skill Title

Instructions in markdown format...

## Section 1

Content...

## Section 2

More content...
```

**Best Practices**:

- Keep under 500 lines
- Use clear, actionable language
- Link to reference files for detailed info
- Include examples where helpful
- Structure with headings for easy navigation

## Bundled Resources

### scripts/

**Purpose**: Executable code that agents can run

**Supported languages**:

- Python
- Bash
- JavaScript/Node.js
- Other languages supported by the agent runtime

**When to use**: For deterministic operations, validation, code generation

**Example**:

```bash
scripts/
├── validate.py      # Validation script
├── generate.sh      # Code generator
└── test.py          # Test runner
```

### references/

**Purpose**: Detailed documentation loaded on-demand

**When to use**: For API docs, schemas, detailed guides that would make skill.md too long

**Example**:

```bash
references/
├── api-reference.md    # Complete API documentation
├── schemas.md          # Configuration schemas
└── examples.md         # Detailed examples
```

**Best practice**: Link from skill.md:

```markdown
For complete API documentation, see [api-reference.md](references/api-reference.md).
```

### assets/

**Purpose**: Files used in skill output

**When to use**: Templates, boilerplate code, images, configuration examples

**Example**:

```bash
assets/
├── template.toml       # Configuration template
├── boilerplate.rs      # Code template
└── diagram.png         # Explanatory diagram
```

## Discovery and Activation

### How Agents Discover Skills

1. **Startup**: Agent scans `.github/skills/` directory
2. **Load metadata**: Reads `name` and `description` from all skill.md files
3. **Index**: Creates searchable index of skills

### How Skills Get Activated

1. **User query**: User asks a question or requests a task
2. **Matching**: Agent matches query against skill descriptions
3. **Loading**: If match found, agent reads full skill.md body
4. **Execution**: Agent follows instructions, loading resources as needed

**Key insight**: The `description` field is critical for discovery. It must contain trigger phrases and keywords.

## Validation

### Using skills-ref

Install the validation tool:

```bash
pip install agentskills
```

Validate a skill:

```bash
# Single skill
skills-ref validate .github/skills/skill-name

# All skills
skills-ref validate .github/skills
```

### Validation Checks

- Frontmatter YAML syntax
- Required fields present
- Field constraints (length, characters)
- No XML tags in metadata
- File structure

### Common Errors

| Error                  | Cause                                  | Fix                            |
| ---------------------- | -------------------------------------- | ------------------------------ |
| Missing required field | `name` or `description` not present    | Add required field             |
| Name too long          | `name` >64 characters                  | Shorten name                   |
| Invalid characters     | Name contains uppercase or underscores | Use lowercase and hyphens only |
| Empty description      | `description` field is empty           | Add comprehensive description  |
| XML tags               | `<` or `>` in name/description         | Remove XML characters          |
| Invalid YAML           | Syntax error in frontmatter            | Fix YAML syntax                |

## IDE Integration

### VS Code with GitHub Copilot

**Enable skills**:

1. Open Settings (Cmd/Ctrl + ,)
2. Search for "chat.useAgentSkills"
3. Enable the setting
4. Reload VS Code window

**Location**: Skills are discovered from `.github/skills/` in workspace root

### Claude Code

Skills are automatically discovered from `.github/skills/` directory.

### Cursor

Enable Agent Skills in settings. Skills are read from `.github/skills/`.

### Windsurf

Skills are automatically supported from `.github/skills/` directory.

## Token Budget Guidelines

### Metadata (Level 1)

- Target: ~100 tokens per skill
- Includes: name + description
- Loaded: At startup for ALL skills

### skill.md Body (Level 2)

- Target: <5000 tokens
- Recommendation: <500 lines
- Loaded: When skill matches task

### References (Level 3)

- No token limit (loaded on-demand)
- Can be extensively detailed
- Only loaded when agent accesses them

## Best Practices Summary

### Writing Effective Descriptions

✅ **Good**:

```yaml
description: |
  Processes Excel files and generates reports. Includes pivot tables,
  charts, and data analysis. Use when working with .xlsx files,
  spreadsheets, or tabular data. Triggers on "analyze excel",
  "process spreadsheet", "generate report from data".
```

❌ **Bad**:

```yaml
description: Helps with files
```

### Structuring skill.md

✅ **Good**:

- Clear sections with headings
- Links to reference files
- Concise, actionable instructions
- Examples where helpful

❌ **Bad**:

- Wall of text
- Unnecessary explanations
- Overly verbose
- No structure

### Using Bundled Resources

✅ **Good**:

- Scripts for deterministic operations
- References for detailed docs
- Clear links from skill.md

❌ **Bad**:

- Everything in skill.md
- No organization
- Unclear when resources are relevant

## Official Resources

- **Specification**: https://agentskills.io/specification
- **What are Skills?**: https://agentskills.io/what-are-skills
- **Validation Tool**: https://github.com/agentskills/agentskills/tree/main/skills-ref
- **Example Skills**: https://github.com/anthropics/skills
- **Claude Documentation**: https://support.claude.com/en/articles/12512198
- **Claude Platform Best Practices**: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
