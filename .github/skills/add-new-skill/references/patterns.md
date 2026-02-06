# Proven Skill Patterns

This document provides proven patterns and workflows for creating effective Agent Skills, drawn from Anthropic's best practices and the Claude platform documentation.

## Table of Contents

- [Content Patterns](#content-patterns)
- [Workflow Patterns](#workflow-patterns)
- [Output Patterns](#output-patterns)
- [Resource Organization Patterns](#resource-organization-patterns)
- [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

## Content Patterns

### Pattern 1: Template Pattern

Provide templates for output format. Match strictness to requirements.

**For strict requirements** (API responses, data formats):

````markdown
## Report Structure

ALWAYS use this exact template structure:

```text
# [Analysis Title]

## Executive Summary

[One-paragraph overview]

## Key Findings

- Finding 1 with data
- Finding 2 with data

## Recommendations

1. Specific recommendation
2. Specific recommendation
```
````

**For flexible guidance** (when adaptation is useful):

````text
## Report Structure

Here is a sensible default format (adapt to context):

```markdown
# [Analysis Title]

## Executive Summary

[Overview]

## Key Findings

[Adapt sections based on analysis]
````

Adjust sections as needed.

### Pattern 2: Examples Pattern

For skills where output quality depends on seeing examples:

````markdown
## Commit Message Format

Generate commit messages following these examples:

**Example 1:**
Input: Added user authentication with JWT tokens
Output:

```text
feat(auth): implement JWT-based authentication

Add login endpoint and token validation middleware
```
````

**Example 2:**
Input: Fixed bug where dates displayed incorrectly
Output:

```text
fix(reports): correct date formatting in timezone conversion

Use UTC timestamps consistently
```

Follow this style: type(scope): brief description, then detailed explanation.

### Pattern 3: Decision Tree Pattern

Guide through decision points:

```markdown
## Deployment Workflow

1. Determine deployment type:

   **New environment?** → Follow "Initial Setup" below
   **Existing environment?** → Follow "Update Workflow" below

2. Initial Setup:
   - Create configuration
   - Provision infrastructure
   - Deploy application

3. Update Workflow:
   - Backup current state
   - Apply changes
   - Verify deployment
```

### Pattern 4: Checklist Pattern

For complex multi-step workflows:

````markdown
## Database Migration Workflow

Copy this checklist and track your progress:

```text
Migration Progress:

- [ ] Step 1: Backup database
- [ ] Step 2: Run migration script
- [ ] Step 3: Verify data integrity
- [ ] Step 4: Update application
- [ ] Step 5: Monitor for errors
```
````

### Step 1: Backup Database

Create backup before migration:

```bash
pg_dump database_name > backup.sql
```

[Continue with detailed steps...]

## Workflow Patterns

### Pattern 1: Linear Workflow

For sequential processes:

```markdown
## Form Filling Workflow

1. **Analyze the form**
   Run: `python scripts/analyze_form.py input.pdf`
2. **Create field mapping**
   Edit `fields.json` with values

3. **Validate mapping**
   Run: `python scripts/validate.py fields.json`

4. **Fill form**
   Run: `python scripts/fill_form.py input.pdf output.pdf`

5. **Verify output**
   Check output.pdf for correctness
```

### Pattern 2: Feedback Loop Workflow

For iterative processes with validation:

````markdown
## Code Review Process

1. Draft your changes following the style guide
2. Run validation:

   ```bash
   ./scripts/validate.sh
   ```
````

1. If issues found:
   - Note each issue
   - Fix the code
   - Go back to step 2
2. Only proceed when validation passes
3. Commit the changes

### Pattern 3: Conditional Workflow

For workflows with branches:

```markdown
## Document Processing

**Step 1**: Identify document type

- PDF document? → Use pdfplumber (see below)
- Word document? → Use python-docx (see below)
- Plain text? → Use standard file operations

**For PDF**:

1. Extract text with pdfplumber
2. Parse structure
3. Export to desired format

**For Word:**

1. Open with python-docx
2. Extract paragraphs
3. Process formatting
```

## Output Patterns

### Pattern 1: Structured Output

For consistent output format:

````markdown
## Test Report Format

Always structure test reports as:

```text
## Test Results

**Status**: [PASS/FAIL]
**Total Tests**: [N]
**Passed**: [N]
**Failed**: [N]

### Failed Tests

- test_name: reason for failure
- test_name: reason for failure

### Summary

[Brief analysis of results]
```
````

### Pattern 2: Progressive Output

For long-running processes:

````markdown
## Deployment Process

Show progress as you work:

```text
🔄 Phase 1: Provisioning infrastructure...
✅ Phase 1: Complete

🔄 Phase 2: Configuring network...
✅ Phase 2: Complete

🔄 Phase 3: Deploying application...
✅ Phase 3: Complete

✅ Deployment completed successfully!
```
````

### Pattern 3: Conditional Output

For context-dependent responses:

````markdown
## Error Reporting

**If successful**:

```text
✅ Operation completed successfully
Results: [details]
```
````

**If failed**:

```text
❌ Operation failed
Error: [error message]
Cause: [likely cause]
Fix: [specific steps to resolve]
```

## Resource Organization Patterns

### Pattern 1: Domain-Specific Organization

For skills covering multiple domains:

```markdown
.github/skills/
└── bigquery-analysis/
├── skill.md (overview + navigation)
└── references/
├── finance.md (revenue, billing)
├── sales.md (pipeline, opportunities)
├── product.md (API usage, features)
└── marketing.md (campaigns, attribution)
```

**skill.md**:

```markdown
# BigQuery Data Analysis

## Available Datasets

**Finance**: Revenue, ARR → [finance.md](references/finance.md)
**Sales**: Pipeline, accounts → [sales.md](references/sales.md)
**Product**: API usage → [product.md](references/product.md)
**Marketing**: Campaigns → [references/marketing.md)
```

Agent loads only the relevant domain's reference file.

### Pattern 2: Progressive Detail Organization

High-level guide with references:

```markdown
.github/skills/
└── pdf-processing/
├── skill.md (quick start + overview)
└── references/
├── forms.md (form filling details)
├── api-reference.md (complete API)
└── examples.md (usage examples)
```

**skill.md**:

````markdown
# PDF Processing

## Quick Start

Extract text:

```python
import pdfplumber
with pdfplumber.open("file.pdf") as pdf:
    text = pdf.pages[0].extract_text()
```
````

## Advanced Features

**Form filling**: [forms.md](references/forms.md)
**API reference**: [api-reference.md](references/api-reference.md)
**Examples**: [examples.md](references/examples.md)

### Pattern 3: Script-Assisted Workflow

Executable scripts with instructions:

```markdown
.github/skills/
└── form-processor/
├── skill.md
└── scripts/
├── analyze.py
├── validate.py
└── fill.py
```

**skill.md**:

````markdown
## Workflow

1. Analyze form:

   ```bash
   python scripts/analyze.py input.pdf
   ```
````

1. Validate data:

   ```bash
   python scripts/validate.py fields.json
   ```

2. Fill form:

   ```bash
   python scripts/fill.py input.pdf output.pdf
   ```

## Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: Overly Verbose Content

**Bad**:

```markdown
PDF files, which stands for Portable Document Format, are a standardized
file format developed by Adobe Systems. They are widely used for document
exchange because they preserve formatting across different systems...
```

**Good**:

````markdown
Extract text from PDFs using pdfplumber:

```python
import pdfplumber
with pdfplumber.open("file.pdf") as pdf:
    text = pdf.pages[0].extract_text()
```
````

**Why**: Assume Claude knows common concepts. Be concise.

### ❌ Anti-Pattern 2: Too Many Options

**Bad**:

```markdown
You can use pypdf, or pdfplumber, or PyMuPDF, or pdf2image, or camelot,
or tabula-py, or pdfminer, or...
```

**Good**:

````markdown
Use pdfplumber for text extraction:

```text
import pdfplumber
```
````

For scanned PDFs requiring OCR, use pdf2image with pytesseract instead.

**Why**: Provide a default recommendation with escape hatch for edge cases.

### ❌ Anti-Pattern 3: Deeply Nested References

**Bad**:

```markdown
# skill.md

See [advanced.md](references/advanced.md)...

# advanced.md

See [details.md](nested/details.md)...

# details.md

Here's the actual information...
```

**Good**:

```markdown
# skill.md

**Basic usage**: [instructions in skill.md]
**Advanced features**: [advanced.md](references/advanced.md)
**API reference**: [reference.md](references/reference.md)
```

**Why**: Claude may partially read nested references. Keep references one level deep.

### ❌ Anti-Pattern 4: Time-Sensitive Information

**Bad**:

```markdown
If you're doing this before August 2025, use the old API.
After August 2025, use the new API.
```

**Good**:

```markdown
## Current Method

Use the v2 API: `api.example.com/v2/`

## Old Patterns

<details>
<summary>Legacy v1 API (deprecated 2025-08)</summary>

The v1 API used: `api.example.com/v1/`
This endpoint is no longer supported.

</details>
```

**Why**: Dates become outdated. Use "current" vs "legacy" sections.

### ❌ Anti-Pattern 5: Inconsistent Terminology

**Bad**:

Mix of "API endpoint", "URL", "API route", "path"...

**Good**:

Always "API endpoint" throughout the skill.

**Why**: Consistency helps Claude understand and follow instructions.

### ❌ Anti-Pattern 6: Everything in skill.md

**Bad**:

```markdown
.github/skills/
└── bigquery/
└── skill.md (2000 lines with everything)
```

**Good**:

```markdown
.github/skills/
└── bigquery/
├── skill.md (overview 200 lines)
└── references/
├── finance.md
├── sales.md
└── product.md
```

**Why**: Keep skill.md <500 lines. Use progressive disclosure.

### ❌ Anti-Pattern 7: Vague Descriptions

**Bad**:

```yaml
description: Helps with documents
```

**Good**:

```yaml
description: |
  Processes Excel files and generates reports. Includes pivot tables
  and charts. Use when working with .xlsx files, spreadsheets, or
  tabular data. Triggers on "analyze excel", "process spreadsheet".
```

**Why**: Specific descriptions enable proper skill discovery.

## Pattern Selection Guide

| Use Case                           | Recommended Pattern             |
| ---------------------------------- | ------------------------------- |
| Output must follow exact format    | Template Pattern (strict)       |
| Quality depends on seeing examples | Examples Pattern                |
| Multiple valid approaches          | Decision Tree Pattern           |
| Complex multi-step workflow        | Checklist Pattern               |
| Iterative process with validation  | Feedback Loop Pattern           |
| Multiple domains/topics            | Domain-Specific Organization    |
| Detailed API/reference docs        | Progressive Detail Organization |
| Includes validation/generation     | Script-Assisted Workflow        |

## References

- [Agent Skills Specification](specification.md)
- [Claude Platform Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [Anthropic skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator)
