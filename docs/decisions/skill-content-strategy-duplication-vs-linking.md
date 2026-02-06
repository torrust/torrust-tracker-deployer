# Decision: Agent Skills Content Strategy - Duplication vs Linking

## Status

Accepted

## Date

2026-02-06

## Context

With the adoption of Agent Skills (agentskills.io) in issue #274, we need to decide how to structure skill content relative to existing project documentation. Agent Skills provide on-demand workflows loaded when tasks match their descriptions, while our official documentation in `docs/` provides the authoritative source of project conventions, architecture, and guidelines.

The question: Should skills duplicate content from official docs, reference them via links, or use a hybrid approach?

### Key Considerations

1. **Progressive Disclosure**: Agent Skills use a three-tier loading model (metadata → skill.md → bundled resources)
2. **Token Efficiency**: Each file read consumes context tokens; skills should be concise
3. **Single Source of Truth**: Duplication creates maintenance burden and version drift
4. **Agent Speed**: File reads have latency; self-contained skills execute faster
5. **Maintenance Cost**: Updating content in multiple locations is error-prone

### Trade-offs Matrix

| Aspect             | Duplicate Content        | Link to Official Docs      |
| ------------------ | ------------------------ | -------------------------- |
| **Maintenance**    | Update 2+ places ❌      | Update 1 place ✅          |
| **Agent Speed**    | Fast (no extra reads) ✅ | Slower (2+ file reads) ❌  |
| **Self-Contained** | Yes ✅                   | No ❌                      |
| **Single Truth**   | No (drift risk) ❌       | Yes ✅                     |
| **Token Usage**    | Moderate ✅              | Higher (multiple files) ❌ |
| **Skill Size**     | Larger ❌                | Smaller ✅                 |

## Decision

Use a **three-tier content strategy** based on information type:

### Tier 1: Self-Contained in skill.md (Core Workflows)

**Include directly in skill.md**: Essential commands and workflows the agent needs to execute immediately.

**Examples**:

- Command syntax: `cargo run --bin linter all`
- Step-by-step workflows: "1. Run command X, 2. Check output Y, 3. Fix if needed"
- Quick reference tables: Common options, flags, error codes

**Why**: Agent can execute without additional file reads. Optimizes for speed and token efficiency.

**Size limit**: Keep skill.md under 500 lines (per Claude best practices)

### Tier 2: Progressive Disclosure via references/ (Supporting Details)

**Include in `references/` directory**: Detailed information loaded on-demand when agent needs deeper context.

**Examples**:

- Detailed descriptions: What each linter does, how it works
- Configuration options: Advanced flags, customization patterns
- Troubleshooting guides: Common errors and fixes
- Examples and patterns: Code samples, workflows

**Why**: Keeps skill.md concise while providing depth when needed. Agent loads selectively based on task requirements.

**Organization**: Use domain-specific files (`references/linters.md`, `references/patterns.md`)

### Tier 3: Links to Official Docs (Deep Context & Authority)

**Link to official docs**: Architectural guidance, contribution rules, decision rationale, and anything requiring authoritative source of truth.

**Examples**:

- Architecture documentation: `docs/codebase-architecture.md`
- Contribution guidelines: `docs/contributing/`
- Architectural Decision Records: `docs/decisions/`
- Development principles: `docs/development-principles.md`

**Why**: Official docs are the single source of truth. Skills provide workflows; docs provide context and rationale.

**Format**: Use relative paths: `../../docs/contributing/linting.md`

### Content Type Decision Tree

```text
Is this information essential to execute the workflow immediately?
├─ YES → Include in skill.md (Tier 1)
└─ NO
   └─ Is this supporting detail agent may need?
      ├─ YES → Include in references/ (Tier 2)
      └─ NO
         └─ Is this deep context, architecture, or policy?
            └─ YES → Link to official docs (Tier 3)
```

## Consequences

### Positive

- **Clear Guidelines**: Contributors know where content belongs
- **Optimal Performance**: Core workflows execute without extra file reads
- **Single Source of Truth**: Official docs remain authoritative
- **Maintainable**: Reduced duplication lowers maintenance burden
- **Scalable**: Pattern works for any skill complexity level
- **Token Efficient**: Progressive disclosure loads only what's needed

### Negative

- **Complexity**: Three-tier system requires understanding to apply correctly
- **Link Maintenance**: Official doc restructuring may break skill links
- **Judgment Calls**: Tier 1 vs Tier 2 boundary sometimes unclear
- **Initial Setup**: More structure upfront than pure duplication

### Mitigation Strategies

1. **Document in Meta-Skill**: Add decision tree to `add-new-skill` skill
2. **Validate Links**: Include link checking in pre-commit linting
3. **Examples**: Provide clear examples of each tier in `add-new-skill/references/examples.md`
4. **Reviews**: PR reviews verify correct tier placement

## Alternatives Considered

### Alternative 1: Pure Duplication

**Approach**: Copy all relevant content into skills.

**Rejected because**:

- Maintenance nightmare: Update 2+ places for every change
- Version drift: Skills and docs inevitably diverge over time
- File bloat: Skills exceed 500-line recommendation
- Violates DRY principle

### Alternative 2: Pure Linking

**Approach**: Skills contain only links to official docs.

**Rejected because**:

- Poor agent UX: Every skill requires 2+ file reads
- Slower execution: Latency compounds across multiple links
- Token inefficient: Loading full docs for simple commands
- Breaks self-contained skill principle

### Alternative 3: Hybrid Without Tiers

**Approach**: Mix duplication and linking without clear rules.

**Rejected because**:

- Inconsistent: No clear guidance for contributors
- Unpredictable: Agent can't optimize loading strategy
- Maintenance unclear: Which content to duplicate vs link?

## Related Decisions

- [ADR: Migration to AGENTS.md Standard](./agents-md-migration.md) - Adopted AGENTS.md for baseline rules
- [Issue #274: Adopt Agent Skills Specification](https://github.com/torrust/torrust-tracker-deployer/issues/274) - Parent epic for skills adoption

## References

- [Agent Skills Specification](https://agentskills.io/specification)
- [Skill Authoring Best Practices (Claude Platform)](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [Progressive Disclosure Pattern](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#progressive-disclosure-patterns)
- [Issue #320: Infrastructure Scaffolding and Foundational Skills](https://github.com/torrust/torrust-tracker-deployer/issues/320)
