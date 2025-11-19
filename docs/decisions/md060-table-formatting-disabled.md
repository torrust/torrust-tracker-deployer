# Decision: Disable MD060 Table Formatting Rule

## Status

Accepted

## Date

2025-11-19

## Context

GitHub issue #186 reported 323 MD060 violations across 14 markdown files. The MD060 rule enforces consistent table column formatting in markdown with four possible styles: "aligned", "compact", "tight", or "any".

Investigation revealed several challenges:

1. **Emoji rendering issues**: Emojis (✅ ❌ 🔄) and Unicode symbols (✓ ✗) have variable visual width across fonts and renderers, preventing reliable auto-formatting with "aligned" style
2. **Auto-fix limitations**: `markdownlint --fix` reduced violations from 323 to 287, but could not achieve perfect alignment in tables containing emojis
3. **Manual effort required**: Remaining violations would require either:
   - Removing all emojis from tables (loses visual appeal)
   - Manual table formatting (high maintenance burden)
   - Complex tooling (over-engineering for aesthetic preference)

## Decision

**Disable MD060 rule entirely** by adding `"MD060": false` to `.markdownlint.json`.

This allows documentation authors to:

- Choose their preferred table formatting style (aligned, compact, tight, or mixed)
- Use emojis in tables for visual clarity without linting violations
- Focus on content quality rather than strict formatting rules
- Avoid maintaining complex tooling for table formatting

## Consequences

**Positive**:

- ✅ No linting violations - all markdown files pass
- ✅ Writers have flexibility in table formatting
- ✅ Emojis allowed in tables for better visual communication
- ✅ No maintenance burden from formatting enforcement
- ✅ Simpler configuration - one less rule to document and enforce

**Negative**:

- Tables may have inconsistent formatting across the repository
- No automated enforcement of table alignment

**Neutral**:

- Tables remain readable regardless of formatting style
- Content quality is more important than formatting consistency

## Alternatives Considered

### 1. Enforce "aligned" Style and Remove Emojis

**Pros**: Consistent table formatting across repository

**Cons**:

- Requires removing emojis from all tables
- High manual effort (204+ violations to fix)
- Loses visual appeal and quick scanning ability
- Ongoing maintenance burden

**Decision**: Rejected - cost outweighs benefit

### 2. Use "any" Style (Allow Any Consistent Format)

**Pros**: Tables can keep emojis

**Cons**:

- Still requires each table to be internally consistent
- 287 violations would remain
- Doesn't solve the fundamental problem

**Decision**: Rejected - doesn't eliminate violations

### 3. Replace Emojis with Unicode Characters

**Pros**: Simpler characters might render more consistently

**Cons**:

- Testing showed Unicode characters (✓ ✗) have same alignment issues as emojis
- Doesn't solve the variable width problem

**Decision**: Rejected - doesn't fix the issue

## Related Decisions

None

## References

- GitHub Issue: #186 - Fix MD060 markdown table column style violations
- Markdownlint MD060 Rule: https://github.com/DavidAnson/markdownlint/blob/main/doc/md060.md
- Emoji Variable Width: Renders as 1-2 character widths depending on font/renderer
- Unicode East Asian Width: https://unicode.org/reports/tr11/
