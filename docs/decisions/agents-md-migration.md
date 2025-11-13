# Decision: Migration to AGENTS.md Standard

## Status

Accepted

## Date

2025-11-13

## Context

The project has been using `.github/copilot-instructions.md` to provide AI agent instructions specifically for GitHub Copilot. However, as AI coding agents have proliferated (Cursor, Aider, Jules, etc.), there is a need for a more universal, agent-agnostic format.

The `AGENTS.md` standard emerged as a collaborative effort across the AI software development ecosystem, supported by multiple major AI coding agents. This open format provides several advantages:

- **Multi-agent compatibility**: Works with GitHub Copilot, Cursor, Aider, Jules from Google, and many others
- **Industry standard**: Community-maintained open format, not tied to any vendor
- **Flexibility**: Can be nested in subdirectories for monorepo projects
- **Future-proof**: Not dependent on any specific vendor's decisions
- **Wide adoption**: Used by over 20,000 open-source projects

GitHub Copilot officially added support for `AGENTS.md` files on August 28, 2025, alongside their existing `.github/copilot-instructions.md` format.

### Related Issue

This decision addresses [issue #172](https://github.com/torrust/torrust-tracker-deployer/issues/172).

## Decision

We will **migrate to the AGENTS.md standard as the primary source** for AI agent instructions, while keeping `.github/copilot-instructions.md` as a minimal redirect file for backward compatibility.

**Implementation:**

1. Create `AGENTS.md` at repository root with complete AI agent instructions
2. Replace `.github/copilot-instructions.md` content with a brief notice that:
   - Points to `AGENTS.md` as the primary location
   - Explains the migration to the standard format
   - Provides links to resources about AGENTS.md
3. Add "AGENTS" to the project spell-check dictionary
4. Document the migration in this ADR

## Consequences

### Positive

- ✅ **Universal compatibility**: Instructions now work with any AI agent that supports the standard
- ✅ **Future-proof**: Not locked into GitHub-specific format
- ✅ **Industry alignment**: Following an emerging community standard
- ✅ **Backward compatible**: GitHub Copilot users can still find instructions via the redirect
- ✅ **Better developer experience**: Works seamlessly across different AI coding tools
- ✅ **Community benefit**: Contributes to the adoption of an open standard

### Negative

- ⚠️ **Two files to maintain**: Must keep redirect file in sync if major changes occur
- ⚠️ **Potential confusion**: Developers might be confused about which file to edit (mitigated by clear redirect message)
- ⚠️ **Learning curve**: Teams need to learn about the new standard (minimal - it's just Markdown)

### Neutral

- 📝 **File location change**: Instructions moved from `.github/` to root directory
- 📝 **Format unchanged**: Both formats use standard Markdown
- 📝 **Content identical**: Same instructions in both locations initially

## Alternatives Considered

### Option 1: Keep only `.github/copilot-instructions.md` (Rejected)

**Pros:**

- No migration needed
- GitHub Copilot users already familiar with this location

**Cons:**

- ❌ Locks project into GitHub-specific format
- ❌ Doesn't work with other AI coding agents
- ❌ Misses opportunity to adopt industry standard

**Rejected because:** This doesn't address the multi-agent compatibility goal.

### Option 2: Maintain both files with identical content (Considered)

**Pros:**

- Maximum compatibility
- No potential for confusion

**Cons:**

- ❌ Duplication of content
- ❌ Higher maintenance burden
- ❌ Risk of files getting out of sync

**Rejected because:** The redirect approach provides the same benefits with less maintenance overhead.

### Option 3: Delete `.github/copilot-instructions.md` entirely (Considered)

**Pros:**

- Single source of truth
- Minimal maintenance

**Cons:**

- ❌ Breaking change for users expecting GitHub-specific file
- ❌ Loses backward compatibility
- ❌ GitHub Copilot users might be confused

**Rejected because:** Keeping a redirect file provides better user experience with minimal cost.

## Migration Path

For teams adopting this pattern:

1. **Check current state**: Review existing `.github/copilot-instructions.md`
2. **Create AGENTS.md**: Copy instructions to repository root as `AGENTS.md`
3. **Update paths**: Change relative paths from `../` to match new location
4. **Update redirect**: Replace copilot-instructions.md with redirect notice
5. **Document change**: Update relevant documentation and team communications
6. **Test**: Verify both GitHub Copilot and other agents can read new instructions

## Related Decisions

- None yet - this is the first ADR related to AI agent instructions

## References

- [AGENTS.md Standard Website](https://agents.md/)
- [GitHub Copilot AGENTS.md Support Announcement](https://github.blog/changelog/2025-08-28-copilot-coding-agent-now-supports-agents-md-custom-instructions/)
- [GitHub Documentation: Repository Custom Instructions](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions)
- [Issue #172: Start using agents.md](https://github.com/torrust/torrust-tracker-deployer/issues/172)
- [OpenAI agents.md Repository](https://github.com/openai/agents.md)

## Implementation Notes

**Date implemented:** 2025-11-13

**Files changed:**

- Created: `AGENTS.md` (repository root)
- Modified: `.github/copilot-instructions.md` (now redirect)
- Modified: `project-words.txt` (added "AGENTS")
- Created: `docs/decisions/agents-md-migration.md` (this ADR)

**Testing:**

- Verify GitHub Copilot can read `AGENTS.md`
- Verify redirect message is clear in `.github/copilot-instructions.md`
- Confirm spell-check passes with new term
