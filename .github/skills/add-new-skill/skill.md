---
name: add-new-skill
description: Guide for creating effective Agent Skills for the torrust-tracker-deployer project. Use when you need to create a new skill (or update an existing skill) that extends AI agent capabilities with specialized knowledge, workflows, or tool integrations. Triggers on "create skill", "add new skill", "how to add skill", or "skill creation".
metadata:
  author: torrust
  version: "1.0"
---

# Creating New Agent Skills

This skill guides you through creating effective Agent Skills for the Torrust Tracker Deployer project.

## About Skills

**What are Agent Skills?**

Agent Skills are specialized instructionsets that extend AI agent capabilities with domain-specific knowledge,workflows, and tool integrations. They follow the [agentskills.io](https://agentskills.io) open format and work with multiple AI coding agents (Claude Code, VS Code Copilot, Cursor, Windsurf).

### Progressive Disclosure

Skills use a three-level loading strategy to minimize context window usage:

1. **Metadata** (~100 tokens): `name` and `description` loaded at startup for all skills
2. **skill.md Body** (<5000 tokens): Loaded when a task matches the skill's description
3. **Bundled Resources**: Loaded on-demand only when referenced (scripts, references, assets)

### When to Create a Skill vs Updating AGENTS.md

| Use AGENTS.md for...            | Use Skills for...               |
| ------------------------------- | ------------------------------- |
| Always-on rules and constraints | On-demand workflows             |
| "Always do X, never do Y"       | Multi-step repeatable processes |
| Baseline conventions            | Specialist domain knowledge     |
| Rarely changes                  | Can be added/refined frequently |

**Example**: "Use lowercase for skill filenames" → AGENTS.md rule. "How to deploy a tracker instance" → Skill.

## Core Principles

### 1. Concise is Key

**Context window is shared** between system prompt, conversation history, other skills, and your actual request.

**Default assumption**: Claude is already very smart. Only add context Claude doesn't already have.

Challenge each piece of information:

- "Does Claude really need this explanation?"
- "Can I assume Claude knows this?"
- "Does this paragraph justify its token cost?"

**Example**:

✅ **Good (50 tokens)**:

````markdown
## Extract PDF text

Use pdfplumber for text extraction:

```text
import pdfplumber
with pdfplumber.open("file.pdf") as pdf:
    text = pdf.pages[0].extract_text()
```
````

❌ **Bad (150 tokens)**:

```markdown
## Extract PDF text

PDF (Portable Document Format) files are a common file format that contains
text, images, and other content. To extract text from a PDF, you'll need to
use a library. There are many libraries available for PDF processing, but we
recommend pdfplumber because it's easy to use...
```

### 2. Set Appropriate Degrees of Freedom

Match specificity to task fragility:

**High freedom** (text-based instructions):

- Multiple approaches are valid
- Decisions depend on context
- Heuristics guide the approach

**Medium freedom** (pseudocode or scripts with parameters):

- A preferred pattern exists
- Some variation is acceptable
- Configuration affects behavior

**Low freedom** (specific scripts, few/no parameters):

- Operations are fragile and error-prone
- Consistency is critical
- Specific sequence must be followed

**Analogy**: Think of Claude as a robot exploring a path:

- **Narrow bridge with cliffs**: Provide exact instructions (low freedom)
- **Open field with no hazards**: Give general direction (high freedom)

### 3. Anatomy of a Skill

A skill consists of:

- **skill.md**: Frontmatter (metadata) + body (instructions)
- **Optional bundled resources**: scripts/, references/, assets/

Keep skill.md concise (<500 lines). Move detailed content to reference files.

### 4. Progressive Disclosure

**Keep skill.md under 500 lines**. Split detailed content into reference files that are loaded only when needed.

**Pattern**: Main skill.md provides overview with links to detailed materials:

```markdown
## Advanced Features

**Full specification**: See [specification.md](references/specification.md) for Agent Skills spec
**Proven patterns**: See [patterns.md](references/patterns.md) for workflow patterns
**Examples**: See [examples.md](references/examples.md) for real skill examples
```

### 5. Content Strategy: Duplication vs Linking

**Three-tier approach** for organizing skill content relative to official project documentation:

#### Tier 1: Self-Contained in skill.md (Core Workflows)

Include essential commands and workflows directly:

- Command syntax: `cargo run --bin linter all`
- Step-by-step workflows
- Quick reference tables

**Why**: Agent executes immediately without extra file reads.

#### Tier 2: Progressive Disclosure via references/ (Supporting Details)

Place detailed information in `references/` directory:

- Detailed descriptions (what each tool does)
- Configuration options and flags
- Troubleshooting guides
- Examples and patterns

**Why**: Keeps skill.md concise; agent loads selectively when needed.

#### Tier 3: Links to Official Docs (Deep Context & Authority)

Link to official documentation for:

- Architecture: `../../docs/codebase-architecture.md`
- Guidelines: `../../docs/contributing/`
- Decisions: `../../docs/decisions/`
- Principles: `../../docs/development-principles.md`

**Why**: Official docs are the single source of truth.

**Decision Tree**:

```text
Is this essential to execute the workflow immediately?
├─ YES → Include in skill.md (Tier 1)
└─ NO → Is this supporting detail agent may need?
    ├─ YES → Include in references/ (Tier 2)
    └─ NO → Link to official docs (Tier 3)
```

See [ADR: Agent Skills Content Strategy](../../../docs/decisions/skill-content-strategy-duplication-vs-linking.md) for full rationale.

## Skill Creation Process

### Step 1: Understanding the Skill

Start with **concrete examples** of when the skill should activate:

**Questions to answer**:

- What specific queries should trigger this skill?
- What tasks does it help accomplish?
- What domain knowledge does it provide?

**Example**:

> "I want a skill that helps with deploying tracker instances. It should activate when users ask 'deploy tracker', 'provision environment', or 'full deployment workflow'."

### Step 2: Planning Reusable Contents

Identify what resources the skill needs:

**Scripts** (`scripts/`):

- Validation scripts
- Code generation utilities
- Deterministic operations

**References** (`references/`):

- API documentation
- Detailed specifications
- Domain-specific guides
- Configuration schemas

**Assets** (`assets/`):

- Templates
- Boilerplate code
- Images/diagrams
- Example files

### Step 3: Creating Directory Structure

#### Choosing the Right Folder

Skills are organized under `.github/skills/` by **audience** and **category**:

```text
.github/skills/
├── add-new-skill/              # Meta skill (stays at root)
├── dev/                        # For contributors/developers
│   ├── cli/                    # CLI commands and user output
│   ├── git-workflow/           # Git, commits, PRs, linters
│   ├── infrastructure/         # Templates, Ansible, Tofu, config architecture
│   ├── planning/               # Issues, ADRs, specs, docs
│   ├── rust-code-quality/      # Error handling, DDD, domain types
│   ├── sdk/                    # SDK methods, examples, integration tests
│   └── testing/                # Unit tests, E2E tests, LXD cleanup
└── usage/                      # For end-users running the deployer
    └── operations/             # Deployment workflows, config creation
```

**Classification decision tree** — walk through in order:

1. **Is this a meta skill about skills themselves?** → `.github/skills/` (root)
2. **Does the skill target end-users** who run the deployer (create environments, render artifacts, manage deployments)? → `usage/operations/`
3. **Is this a developer/contributor skill?** → `dev/{category}/`
   - CLI commands, user output → `dev/cli/`
   - Git operations, PRs, linting → `dev/git-workflow/`
   - Templates, Ansible, Tofu, config DTOs → `dev/infrastructure/`
   - Issues, ADRs, specs, docs → `dev/planning/`
   - Rust patterns, error handling, DDD → `dev/rust-code-quality/`
   - SDK client library → `dev/sdk/`
   - Testing strategies → `dev/testing/`

**If a skill serves both audiences**, split it into two focused skills (one under `dev/`, one under `usage/`). See `create-environment-config` and `environment-config-architecture` for an example.

Create the skill directory:

```bash
mkdir -p .github/skills/dev/{category}/skill-name/{scripts,references,assets}
touch .github/skills/dev/{category}/skill-name/skill.md
```

**Naming convention**: Use gerund form (verb + -ing) or noun phrases:

- ✅ Good: `processing-pdfs`, `analyzing-spreadsheets`, `deploying-tracker`
- ❌ Avoid: `helper`, `utils`, `tools` (too vague)

### Step 4: Writing skill.md

#### Frontmatter (Required)

```yaml
---
name: skill-name
description: What this skill does AND when to use it. Include trigger phrases here because the body is only loaded AFTER the skill is triggered. Triggers on "keyword1", "keyword2", "keyword3".
metadata:
  author: torrust
  version: "1.0"
---
```

> **Important**: `description` must be a plain single-line string. Do **not** use YAML block scalars (`|`) or folded scalars (`>`). The IDE skill-file parser treats multi-line forms as unexpected indentation and reports unsupported attributes, causing the skill to fail to load.

**Critical**: The `description` must include:

1. **What** the skill does
2. **When** to use it (trigger conditions)
3. **Key terms** and trigger phrases

Why? Claude uses the description to decide whether to load the skill. If trigger phrases aren't in the description, the skill won't activate.

#### Body Structure

**Recommended sections**:

1. **Overview** - Brief introduction
2. **When to Use** - Specific scenarios
3. **Workflow/Instructions** - Step-by-step process
4. **Common Patterns** - Frequently used approaches
5. **References** - Links to bundled resources

**Keep instructions**:

- Imperative/infinitive form ("Run this", "Check that")
- Focused and actionable
- Free of unnecessary explanations

### Step 5: Adding Bundled Resources

Create reference files for detailed information:

```bash
# Example: API documentation
cat > .github/skills/skill-name/references/api-reference.md << 'EOF'
# API Reference

Complete API documentation...
EOF
```

**Structure longer reference files** with table of contents:

```markdown
# API Reference

## Contents

- Authentication and setup
- Core methods (create, read, update, delete)
- Advanced features
- Error handling patterns

## Authentication and setup

...
```

**Keep references one level deep** from skill.md. Avoid nested references.

### Step 6: Validation

Install the validation tool (first time only):

```bash
pip install agentskills
```

Validate your skill:

```bash
skills-ref validate .github/skills/skill-name
```

**Fix any validation errors** before testing.

### Step 7: Testing and Iteration

1. **Test with trigger phrases**: Ask Claude various questions that should activate the skill
2. **Observe behavior**: Does Claude load the skill? Follow instructions correctly?
3. **Iterate based on usage**: Refine instructions, add missing information, improve clarity
4. **Get feedback**: Ask team members to try the skill

**Example test queries**:

- Direct: "How do I use the [skill-name] skill?"
- Implicit: "[trigger phrase from description]"
- Edge cases: "[uncommon but valid trigger]"

## skill.md Frontmatter Reference

### Required Fields

**name** (required):

- Maximum 64 characters
- Lowercase letters, numbers, hyphens only
- No XML tags, no reserved words ("anthropic", "claude")

**description** (required):

- Non-empty, maximum 1024 characters
- **Must be a plain single-line string** — never use YAML block (`|`) or folded (`>`) scalars
- Include WHAT the skill does
- Include WHEN to use it
- List trigger phrases/keywords
- Use third person ("Processes Excel files" not "I can help you process")

### Optional Fields

**metadata** (optional):

```yaml
metadata:
  author: torrust
  version: "1.0"
  last-updated: "2026-02-06"
```

**allowed-tools** (optional):

```yaml
allowed-tools:
  - Read
  - Bash(cargo build)
  - Bash(python scripts/validate.py)
```

**compatibility** (optional):

```yaml
compatibility:
  agents:
    - claude-code
    - github-copilot
    - cursor
```

## skill.md Body Patterns

### Pattern 1: High-Level Guide with References

````markdown
# PDF Processing

## Quick Start

Extract text with pdfplumber:

```python
import pdfplumber
with pdfplumber.open("file.pdf") as pdf:
    text = pdf.pages[0].extract_text()
```
````

## Advanced Features

```text
**Form filling**: See forms.md for complete guide
**API reference**: See reference.md for all methods
**Examples**: See examples.md for common patterns
```

_Note: Example pattern - replace with your skill's actual reference files._

### Pattern 2: Domain-Specific Organization

For skills with multiple domains, organize content by domain:

```text
# BigQuery Data Analysis

## Available Datasets

**Finance**: Revenue, ARR → See finance.md
**Sales**: Pipeline, opportunities → See sales.md
**Product**: API usage, features → See product.md
```

_Note: Hypothetical domain-specific example - adapt to your skill's needs._

Claude loads only the relevant domain's reference file.

### Pattern 3: Conditional Details

```text
# DOCX Processing

## Creating Documents

Use docx-js for new documents. See docx-js.md.

## Editing Documents

For simple edits, modify the XML directly.

**For tracked changes**: See redlining.md
**For OOXML details**: See ooxml.md
```

_Note: Hypothetical example showing conditional progressive disclosure._
**Purpose**: Executable code for deterministic operations

**Examples**:

- Validation scripts
- Code generators
- File processors

**Guidelines**:

- Make scripts solve problems, not punt to Claude
- Handle errors explicitly
- Document parameters

### References Directory

**Purpose**: Detailed documentation loaded on-demand

**Examples**:

- API references
- Configuration schemas
- Domain-specific guides

**Guidelines**:

- Keep each file focused on one topic
- Use table of contents for files >100 lines
- Link clearly from skill.md

### Assets Directory

**Purpose**: Files used in skill output

**Examples**:

- Templates
- Boilerplate code
- Configuration examples

**Guidelines**:

- Reference assets explicitly
- Keep files small and focused
- Version templates when needed

## Validation and Integration

### Validation with skills-ref

```bash
# Validate single skill
skills-ref validate .github/skills/skill-name

# Validate all skills
skills-ref validate .github/skills
```

**Common validation errors**:

| Error                      | Fix                                  |
| -------------------------- | ------------------------------------ |
| Invalid frontmatter YAML   | Check YAML syntax, required fields   |
| Name too long              | Shorten to ≤64 characters            |
| Invalid characters in name | Use only lowercase, numbers, hyphens |
| Empty description          | Add comprehensive description        |
| XML tags in metadata       | Remove `<` and `>` characters        |

### Testing with GitHub Copilot

1. **Enable skills in VS Code**: Settings → `chat.useAgentSkills` → enable
2. **Reload VS Code**: Command Palette → "Reload Window"
3. **Test activation**: Ask questions using trigger phrases from description
4. **Verify behavior**: Check that Claude follows skill instructions
5. **Iterate**: Refine based on observed behavior

### Integrating with AGENTS.md

Add to the "Auto-Invoke Skills" table using the full path under `.github/skills/`:

```markdown
## Auto-Invoke Skills

| Task               | Skill to Load                                       |
| ------------------ | --------------------------------------------------- |
| [Task description] | `.github/skills/dev/{category}/skill-name/skill.md` |
```

Keep entries alphabetically sorted by task name.

## Examples and Patterns

For detailed examples, see:

- [references/specification.md](references/specification.md) - Full Agent Skills spec
- [references/patterns.md](references/patterns.md) - Proven patterns and workflows
- [references/examples.md](references/examples.md) - Example skills from this project

## Quick Reference

| Task                    | Command/Action                                      |
| ----------------------- | --------------------------------------------------- |
| Create skill directory  | `mkdir -p .github/skills/dev/{category}/skill-name` |
| Validate skill          | `skills-ref validate .github/skills/skill-name`     |
| Test with Copilot       | Enable `chat.useAgentSkills` in VS Code             |
| Add to AGENTS.md        | Update "Auto-Invoke Skills" table                   |
| Install validation tool | `pip install agentskills`                           |

## Tips

- **Start simple**: Begin with just skill.md, add resources as needed
- **Test early**: Validate and test before adding complex content
- **Be specific in descriptions**: Include all trigger phrases and use cases
- **Use progressive disclosure**: Keep skill.md focused, move details to references
- **Iterate based on usage**: Refine after observing how Claude uses the skill
- **Learn from examples**: Study skills in `references/examples.md`
