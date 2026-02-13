---
name: create-adr
description: Guide for creating Architectural Decision Records (ADRs) in the torrust-tracker-deployer project. Covers the ADR template, file naming, index registration, and commit workflow. Use when documenting architectural decisions, recording design choices, or adding decision records. Triggers on "create ADR", "add ADR", "new decision record", "architectural decision", "document decision", or "add decision".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Architectural Decision Records

This skill guides you through creating ADRs for the Torrust Tracker Deployer project.

## Quick Reference

```bash
# 1. Create the ADR file
cp docs/decisions/TEMPLATE.md docs/decisions/{kebab-case-title}.md

# 2. Add entry to the index table in docs/decisions/README.md

# 3. Run pre-commit checks
./scripts/pre-commit.sh

# 4. Commit
git commit -m "docs: [#{issue}] add ADR for {short description}"
```

## When to Create an ADR

Create an ADR when making a decision that:

- Affects the project's architecture or design patterns
- Chooses one approach over alternatives that were considered
- Has consequences (positive or negative) worth documenting
- Would benefit future contributors who ask "why was this done this way?"

Do **not** create an ADR for trivial implementation choices or style preferences already covered by linting rules.

## ADR Template

Every ADR uses the structure from `docs/decisions/TEMPLATE.md`:

```markdown
# Decision: [Title]

## Status

[Proposed | Accepted | Rejected | Superseded]

## Date

YYYY-MM-DD

## Context

What is the issue motivating this decision?

## Decision

What change are we implementing?

## Consequences

What becomes easier or more difficult? What risks are introduced?

## Alternatives Considered

What other options were evaluated and why were they rejected?

## Related Decisions

Links to other relevant ADRs.

## References

Links to external resources, issues, or PRs.
```

## Step-by-Step Process

### Step 1: Choose a Filename

Use `kebab-case` matching the decision topic:

```text
docs/decisions/{kebab-case-title}.md
```

Examples: `concurrent-docker-image-builds-in-tests.md`, `caddy-for-tls-termination.md`

### Step 2: Write the ADR

Fill in every section of the template:

- **Status**: Use `✅ Accepted` for decisions being implemented now. Use `Proposed` if pending review.
- **Date**: Use today's date in `YYYY-MM-DD` format
- **Context**: Explain the problem thoroughly — include enough background for future readers who have no prior context. Include links to issues or PRs if applicable.
- **Decision**: State clearly what was decided and why. Include code examples if the decision involves specific patterns.
- **Consequences**: Document **both** positive and negative consequences. Be honest about trade-offs.
- **Alternatives Considered**: List each alternative with a clear explanation of why it was rejected. This is one of the most valuable sections — it prevents future contributors from re-exploring dead ends.
- **Related Decisions**: Link to other ADRs in the same directory
- **References**: Link to GitHub issues, PRs, external documentation

### Step 3: Add to the Decision Index

Add a new row to the table in `docs/decisions/README.md`, sorted by date (newest first):

```markdown
| ✅ Accepted | YYYY-MM-DD | [Title](./filename.md) | One-line summary (max ~85 chars) |
```

The table columns are: Status, Date, Decision (link), Summary.

### Step 4: Validate and Commit

```bash
# Lint the new ADR and the updated index
npx markdownlint-cli docs/decisions/{filename}.md
npx markdownlint-cli docs/decisions/README.md
npx cspell lint docs/decisions/{filename}.md

# Run full pre-commit checks
./scripts/pre-commit.sh

# Commit with conventional format
git add docs/decisions/{filename}.md docs/decisions/README.md
git commit -m "docs: [#{issue}] add ADR for {short description}"
```

## Guidelines

From `docs/decisions/README.md`:

- **One decision per file**: Each ADR focuses on a single architectural decision
- **Immutable**: Once accepted, ADRs should not be modified. Create new ADRs to supersede old ones
- **Context-rich**: Include enough background for future readers
- **Consequence-aware**: Document both positive and negative consequences
- **Linked**: Reference related decisions and external resources

## Status Definitions

| Status         | Meaning                                    |
| -------------- | ------------------------------------------ |
| **Proposed**   | Decision is under discussion               |
| **Accepted**   | Decision has been approved and implemented |
| **Rejected**   | Decision was considered but not approved   |
| **Superseded** | Decision has been replaced by a newer ADR  |

## Common Mistakes

- **Missing alternatives**: Always document what was considered and rejected — this is the most valuable part for future contributors
- **Vague consequences**: Be specific about trade-offs, not just "this is simpler"
- **Forgetting the index**: Every ADR must be added to the table in `docs/decisions/README.md`
- **Wrong sort order**: Index entries are sorted newest-first by date

## References

- ADR index and guidelines: `docs/decisions/README.md`
- ADR template: `docs/decisions/TEMPLATE.md`
- AGENTS.md rule 12: "Before making engineering decisions, document as ADRs"
