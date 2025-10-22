# Add User Documentation

**Issue**: [#24](https://github.com/torrust/torrust-tracker-deployer/issues/24)
**Parent Epic**: #10 - UI Layer Destroy Command
**Related**: Issue #23: Add Clap Subcommand Configuration

## Overview

Create comprehensive user-facing documentation for the destroy command. This includes usage examples, command options, troubleshooting guidance, and safety considerations to ensure users can effectively and safely use the destroy functionality.

## Goals

- [ ] Create user guide section for destroy command
- [ ] Document command usage with clear examples
- [ ] Provide troubleshooting section for common issues
- [ ] Document safety considerations and best practices
- [ ] Ensure documentation passes all markdown linting

## Specifications

### Existing Documentation

There's already a comprehensive draft document at `docs/user-guide/commands/destroy.md` that needs to be updated for the CLI implementation. The current document has:

#### Current Content

- Implementation status warning (needs updating to reflect CLI availability)
- Planned CLI usage examples (needs updating to actual usage)
- Comprehensive troubleshooting section (mostly complete)
- Safety considerations and best practices (good foundation)
- Automation examples (needs review for current capabilities)

#### Updates Needed

**Remove Implementation Status Warning**:
The document starts with a warning that CLI interface is not available. This needs to be removed and replaced with actual usage instructions.

**Update CLI Examples**:
All examples currently show "Planned CLI usage (not yet available)" comments. These need to be updated to show actual working commands.

**Add User Output Format**:
The document doesn't describe the expected user output format with progress messages. Needs to add section on what users will see when running the command.

**Remove Unimplemented Command References**:

- Remove references to `provision` command (not implemented yet)
- Remove references to `list` command (not implemented yet)
- Update troubleshooting to not suggest using unimplemented commands

**Update Logging Integration**:
The document mentions logging options that may not match the current UserOutput implementation. Needs alignment with the dual-channel approach (stdout/stderr).

### Update User Guide Integration

Check if `docs/user-guide/README.md` or `docs/user-guide/commands.md` need updates to reference the destroy command. Ensure no references to unimplemented commands (provision, list) are added.

### Help Text Enhancement

Ensure the CLI help text is comprehensive in `src/app.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Destroy an existing deployment environment
    ///
    /// This command permanently removes all infrastructure, configuration,
    /// and associated resources for the specified environment.
    ///
    /// WARNING: This operation is irreversible.
    Destroy {
        /// Name of the environment to destroy
        ///
        /// Must be alphanumeric with hyphens allowed.
        /// Examples: dev, test-env, staging-01
        environment: String,
    },
}
```

## Implementation Plan

### Subtask 1: Update Existing Documentation Status (45 minutes)

- [ ] Remove implementation status warning from `docs/user-guide/commands/destroy.md`
- [ ] Update introduction to indicate CLI is now available
- [ ] Remove all "Planned CLI usage (not yet available)" comments throughout document
- [ ] Update command examples to show actual working syntax

### Subtask 2: Add User Output Format Documentation (1 hour)

- [ ] Add section describing expected user output with progress messages
- [ ] Document the dual-channel approach (stdout/stderr)
- [ ] Add examples of what users will see during destroy operation
- [ ] Explain separation between user output and internal logging
- [ ] Update any logging-related sections to match UserOutput implementation

### Subtask 3: Remove Unimplemented Command References (30 minutes)

- [ ] Remove references to `provision` command throughout document
- [ ] Remove references to `list` command in troubleshooting section
- [ ] Update troubleshooting suggestions to not use unimplemented commands
- [ ] Update "Related Commands" section to only include available functionality

### Subtask 4: Enhance CLI Help Text (30 minutes)

- [ ] Update help text in `src/app.rs` for destroy subcommand
- [ ] Add detailed descriptions and examples
- [ ] Ensure help text matches updated documentation
- [ ] Test help output with `cargo run -- help destroy`

### Subtask 5: Review and Update Integration (45 minutes)

- [ ] Check `docs/user-guide/README.md` for destroy command references
- [ ] Ensure consistent formatting and style throughout
- [ ] Run markdown linting on updated documentation
- [ ] Verify all internal links work correctly
- [ ] Remove any automation examples that reference unimplemented commands

## Acceptance Criteria

- [ ] Implementation status warning removed from `docs/user-guide/commands/destroy.md`
- [ ] All "Planned CLI usage" comments replaced with actual working examples
- [ ] User output format documented with progress message examples
- [ ] Dual-channel approach (stdout/stderr) documented for user output
- [ ] All references to unimplemented commands (`provision`, `list`) removed
- [ ] Troubleshooting section updated to not suggest unimplemented commands
- [ ] CLI help text enhanced with detailed descriptions
- [ ] Documentation aligns with UserOutput implementation from Issue 10.2
- [ ] All markdown linting passes successfully
- [ ] Internal documentation links verified and working

## Related Documentation

- [User Guide Structure](../user-guide/README.md)
- [Documentation Guidelines](../documentation.md)
- [Contributing Documentation](../contributing/README.md)
- [Epic #10](./10-epic-ui-layer-destroy-command.md) - Parent epic context

## Notes

**Estimated Time**: 3.5 hours

**Documentation Focus**: Since most content already exists, this task focuses on:

- **Updating Status**: Remove "not implemented" warnings and comments
- **Adding User Output**: Document what users will see when running commands
- **Clean References**: Remove mentions of unimplemented features
- **Align with Implementation**: Ensure documentation matches the UserOutput approach

**Key Update Priorities**:

1. **Remove Implementation Warnings**: Most visible change - show CLI is available
2. **User Output Examples**: Critical for user experience
3. **Clean References**: Prevent user confusion about unavailable commands
4. **Technical Alignment**: Ensure logging/output documentation is accurate

**Testing Documentation**:

```bash
# Test markdown linting
cargo run --bin linter markdown

# Test CLI help text
cargo run -- help destroy
cargo run -- destroy --help

# Verify documentation links
# (Manual verification of internal links)
```

**Style Guidelines**:

- Use consistent formatting with existing user guide
- Include emoji for visual clarity (⚠️ for warnings, ✅ for success)
- Provide code blocks for all command examples
- Use proper markdown headers for navigation
- Include cross-references to related commands and documentation
