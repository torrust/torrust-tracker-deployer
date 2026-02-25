# Example Skills

This document provides concrete examples of Agent Skills from the Torrust Tracker Deployer project and Anthropic's skills repository.

## Table of Contents

- [From This Project](#from-this-project)
- [From Anthropic Repository](#from-anthropic-repository)
- [Anatomy Breakdown](#anatomy-breakdown)

## From This Project

### Example 1: run-linters Skill

**Location**: `.github/skills/dev/git-workflow/run-linters/`

**Purpose**: Guide users through running code quality checks and linters

**Structure**:

```text
run-linters/
├── skill.md              # Main skill (350 lines)
└── references/
    └── linters.md        # Detailed linter docs (450 lines)
```

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

**Key Sections**:

1. When to Use This Skill
2. Available Linters (individual and comprehensive)
3. Common Workflows (pre-commit, fix specific, fast dev)
4. Fixing Common Issues (with examples)
5. Integration with Development Workflow

**What makes it effective**:

- ✅ Clear trigger phrases in description
- ✅ Actionable workflows with specific commands
- ✅ Common issues addressed with solutions
- ✅ Links to detailed reference (linters.md)
- ✅ Quick reference table at end

**Progressive disclosure**: Main skill.md has essentials, linters.md has comprehensive details about each linter.

### Example 2: add-new-skill Skill (Meta-Skill)

**Location**: `.github/skills/add-new-skill/`

**Purpose**: Document how to create new Agent Skills (self-documenting system)

**Structure**:

```text
add-new-skill/
├── skill.md              # Main skill (400 lines)
└── references/
    ├── specification.md  # Full Agent Skills spec
    ├── patterns.md       # Proven design patterns
    └── examples.md       # This file!
```

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

**Key Sections**:

1. About Skills (what they are, progressive disclosure, when to use)
2. Core Principles (concise, degrees of freedom, anatomy, progressive disclosure)
3. Skill Creation Process (7 steps end-to-end)
4. Frontmatter Reference
5. Body Patterns
6. Bundled Resources Best Practices
7. Validation and Integration

**What makes it effective**:

- ✅ Follows Anthropic's skill-creator pattern
- ✅ Comprehensive 7-step process
- ✅ Examples throughout
- ✅ Links to detailed references
- ✅ Self-documenting (uses skills to explain skills)

**Pattern used**: High-level guide with references (Pattern 1 from patterns.md)

## From Anthropic Repository

### Example 3: skill-creator (Anthropic)

**Source**: https://github.com/anthropics/skills/tree/main/skills/skill-creator

**Purpose**: Meta-skill for creating new Agent Skills

**Structure** (simplified):

```text
skill-creator/
└── SKILL.md              # Self-contained skill
```

**SKILL.md Frontmatter** (adapted):

```yaml
---
name: skill-creator
description: |
  Creating effective Skills for Claude that extend capabilities with
  specialized knowledge and workflows. Use when creating new Skills
  or improving existing ones.
---
```

**Key Sections** (adapted):

1. **About Skills**
   - What Skills provide
   - When to create a Skill

2. **Core Principles**
   - Concise is key
   - Set appropriate degrees of freedom
   - Test with all models

3. **Skill Structure**
   - YAML frontmatter requirements
   - Naming conventions
   - Writing effective descriptions
   - Progressive disclosure patterns

4. **Workflows and Feedback Loops**
   - Use workflows for complex tasks
   - Implement feedback loops

5. **Content Guidelines**
   - Avoid time-sensitive information
   - Use consistent terminology
   - Common patterns (templates, examples, conditional workflows)

6. **Evaluation and Iteration**
   - Build evaluations first
   - Develop skills iteratively with Claude
   - Observe how Claude navigates skills

**What makes it effective**:

- ✅ Proven structure used by Anthropic
- ✅ Balance of principles and practical guidance
- ✅ Emphasis on iteration and testing
- ✅ Clear examples throughout
- ✅ Self-contained but references additional docs

**Pattern used**: Self-contained with external references

### Example 4: pdf-processing (Hypothetical from Docs)

**Purpose**: Extract text and tables from PDFs, fill forms

**Structure**:

```text
pdf-processing/
├── SKILL.md              # Main skill
├── FORMS.md              # Form filling guide
├── reference.md          # API reference
├── examples.md           # Usage examples
└── scripts/
    ├── analyze_form.py
    ├── fill_form.py
    └── validate.py
```

**SKILL.md Frontmatter**:

```yaml
---
name: pdf-processing
description: |
  Extracts text and tables from PDF files, fills forms, and merges documents.
  Use when working with PDF files or when the user mentions PDFs, forms, or
  document extraction.
---
```

**Key Sections**:

1. Quick Start (basic text extraction)
2. Advanced Features (with links to FORMS.md, reference.md, examples.md)

**What makes it effective**:

- ✅ Simple quick start for common case
- ✅ Progressive disclosure for advanced features
- ✅ Scripts for deterministic operations
- ✅ References for detailed docs

**Pattern used**: High-level guide with references + script-assisted workflow

### Example 5: bigquery-skill (Hypothetical from Docs)

**Purpose**: Analyze BigQuery data across multiple domains

**Structure**:

```text
bigquery-skill/
├── SKILL.md              # Overview + navigation
└── reference/
    ├── finance.md        # Revenue, billing metrics
    ├── sales.md          # Opportunities, pipeline
    ├── product.md        # API usage, features
    └── marketing.md      # Campaigns, attribution
```

**SKILL.md**:

````markdown
# BigQuery Data Analysis

## Available Datasets

**Finance**: Revenue, ARR → [finance.md](reference/finance.md)
**Sales**: Pipeline, accounts → [sales.md](reference/sales.md)
**Product**: API usage → [product.md](reference/product.md)
**Marketing**: Campaigns → [marketing.md](reference/marketing.md)

## Quick Search

Find metrics:

```bash
grep -i "revenue" reference/finance.md
grep -i "pipeline" reference/sales.md
```
````

**What makes it effective**:

- ✅ Domain-specific organization
- ✅ Only relevant domain loaded
- ✅ Clear navigation structure
- ✅ Search guidance provided

**Pattern used**: Domain-specific organization (Pattern 1 from Resource Organization)

## Anatomy Breakdown

### Complete Skill Anatomy: run-linters

Let's break down the run-linters skill component by component:

#### 1. Frontmatter (Discovery Phase)

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

**Purpose**: Loaded at startup, enables discovery  
**Key elements**:

- Name: Simple, descriptive (gerund form)
- Description: What it does + when to use + trigger phrases
- Metadata: Optional but helpful for maintenance

#### 2. Quick Reference (High-Level Orientation)

```markdown
## When to Use This Skill

- Before committing code (mandatory)
- After making code changes
- When fixing CI failures
- When code quality issues are reported
```

**Purpose**: Orients user to skill's purpose  
**Pattern**: Bulleted list of scenarios

#### 3. Main Content (Core Instructions)

````markdown
## Available Linters

### Run All Linters

```bash
cargo run --bin linter all
```
````

### Run Individual Linters

```text
cargo run --bin linter markdown
cargo run --bin linter yaml
...
```

**Purpose**: Actionable instructions with commands  
**Pattern**: Hierarchical with code examples

#### 4. Workflows (Common Use Cases)

````markdown
## Common Workflows

### Workflow 1: Pre-Commit Checks (Required)

```bash
./scripts/pre-commit.sh
```
````

### Workflow 2: Fix Specific Linter Errors

```bash
cargo run --bin linter markdown
```

**Purpose**: Guide users through common scenarios  
**Pattern**: Numbered workflows with clear steps

#### 5. Troubleshooting (Common Issues)

```markdown
## Fixing Common Issues

### Markdown Formatting Issues

**Error**: Lines too long
**Solution**: Break into multiple lines
```

**Purpose**: Address anticipated problems  
**Pattern**: Error + solution pairs

#### 6. Progressive Disclosure (Link to Details)

```markdown
For detailed configuration, see:

- [references/linters.md](references/linters.md)
- [docs/contributing/linting.md](../../../docs/contributing/linting.md)
```

**Purpose**: Point to detailed information  
**Pattern**: Bulleted links with brief descriptions

#### 7. Quick Reference Table (Summary)

```markdown
| Task                | Command                    |
| ------------------- | -------------------------- |
| Run all linters     | cargo run --bin linter all |
| Fix Rust formatting | `cargo fmt`                |
```

**Purpose**: Fast lookup for common tasks  
**Pattern**: Table format for scannability

### Structure Analysis

**Token-efficient design**:

1. **Frontmatter** (~75 tokens): Loaded for all skills
2. **Main skill.md** (~2500 tokens): Loaded when activated
3. **references/linters.md** (~3000 tokens): Loaded only if user needs details

**Total**: ~5575 tokens, but typically only ~2575 loaded (46% savings)

## Comparison: Simple vs Complex Skills

### Simple Skill Example

**Use case**: Environment variable naming guidance

````yaml
---
name: name-env-vars
description: |
  Guidelines for naming environment variables. Use when creating new
  environment variables or need naming convention guidance. Triggers
  on "environment variable", "env var naming", "variable convention".
---

# Environment Variable Naming

## Convention

Use `TORRUST_TD_PREFIX` for all project variables:

```bash
TORRUST_TD_DEBUG=1
TORRUST_TD_LOG_LEVEL=info
````

## Framework

1. Condition-based: `TORRUST_TD_ENABLE_FEATURE`
2. Action-based: `TORRUST_TD_SKIP_VALIDATION`

See [docs/contributing/environment-variables-naming.md] for details.

**Characteristics**:

- Single skill.md file
- ~150 lines
- Straightforward guidance
- Links to existing docs

### Complex Skill Example

**Use case**: Full deployment workflow

```text
---
name: deploy-tracker
description: |
  Complete workflow for deploying Torrust Tracker instances. Covers
  create, provision, configure, release, and run phases. Use for
  full deployments or when user asks about deployment process.
  Triggers on "deploy tracker", "provision environment", "full deployment".
---

# Deploy Tracker

[High-level overview]

## Workflows

**New deployment**: See [new-deployment.md](references/new-deployment.md)
**Update existing**: See [update-deployment.md](references/update-deployment.md)
**Troubleshooting**: See [troubleshooting.md](references/troubleshooting.md)

## Phase 1: Configuration

...with links to references/configuration.md

## Phase 2: Provisioning

...with links to references/provisioning.md

[etc.]
```

**Structure**:

```text
deploy-tracker/
├── skill.md                    # Overview + navigation
├── references/
│   ├── new-deployment.md       # Complete new deployment guide
│   ├── update-deployment.md    # Update existing deployment
│   ├── troubleshooting.md      # Common issues
│   ├── configuration.md        # Configuration details
│   ├── provisioning.md         # Provisioning details
│   └── phases.md               # Phase-by-phase breakdown
└── scripts/
    ├── validate-config.py      # Configuration validation
    └── check-prerequisites.sh  # Prerequisite checker
```

**Characteristics**:

- Multiple reference files
- Scripts for validation
- Domain-specific organization
- Progressive disclosure heavily used
- ~500 lines in skill.md, ~3000 in references

## Key Takeaways

1. **Start simple**: Begin with basic skill.md, add complexity as needed
2. **Follow patterns**: Use proven patterns from this document
3. **Progressive disclosure**: Keep skill.md focused, use references for details
4. **Test and iterate**: Skills improve based on real usage
5. **Learn from examples**: Study skills that work well

## References

- [Agent Skills Specification](specification.md)
- [Proven Patterns](patterns.md)
- [Anthropic Skills Repository](https://github.com/anthropics/skills)
- [Claude Platform Documentation](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
