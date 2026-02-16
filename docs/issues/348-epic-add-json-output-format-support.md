# Add JSON Output Format Support

**Issue**: #348
**Parent Epic**: #1 - Project Roadmap
**Related**: [Roadmap Section 12](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support)

## Overview

This epic tracks the implementation of machine-readable JSON output format (`--json` flag) for selected commands in the Torrust Tracker Deployer. The goal is to improve automation capabilities and AI agent integration by providing structured, programmatically parsable output.

## Roadmap Reference

From [docs/roadmap.md](../roadmap.md) - Section 12:

> Add machine-readable JSON output format (`--json` flag) for selected commands to improve automation and AI agent integration. Initial phase focuses on commands where structured output provides the most value.
>
> **Context**: JSON output enables programmatic parsing, making it easier for scripts and AI agents to extract specific information (like IP addresses, service URLs, environment names) without parsing human-readable text.

## Goals

- [ ] Add `--json` flag support to high-value commands
- [ ] Enable automation workflows to extract specific information programmatically
- [ ] Improve AI agent reliability when parsing command output
- [ ] Provide structured output without breaking existing human-readable format
- [ ] Document JSON schemas for each command

## Context

### Why JSON Output Matters

Currently, all command output is designed for human readability (formatted text, tables, progress indicators). This creates challenges for:

- **Automation scripts**: Must use regex/text parsing to extract information (fragile, error-prone)
- **AI agents**: Must interpret human-readable text, leading to hallucinations and parsing errors
- **Integration workflows**: Different tools need structured data (IP addresses, URLs, service states)

JSON output provides:

- ✅ **Type safety**: Structured data with clear field types
- ✅ **Reliability**: No ambiguity in parsing
- ✅ **Extensibility**: Can add new fields without breaking parsers
- ✅ **Standards compliance**: Standard format supported by all tools

### Current Architecture

Thanks to the MVC pattern already in place (presentation layer separation), adding JSON output should be straightforward:

1. Commands already separate business logic from output formatting
2. Presentation layer (`src/presentation/`) handles all user-facing output
3. Most data structures already implement `Serialize` for state persistence

## Phase 1 - High-Value Commands

### 12.1 Add JSON output to `create` command

**Issue**: #349
**Specification**: [docs/issues/349-add-json-output-to-create-command.md](./349-add-json-output-to-create-command.md)

**Rationale**: Contains info about where to find more detailed information (paths, configuration references). Structured output helps automation track environment artifacts.

**Key fields**:

- Environment name
- Current state
- Data directory path
- Build directory path
- Configuration file path

**Use cases**:

- Automation tracking artifact locations
- CI/CD pipelines coordinating multiple environments
- Scripts that need to know where to find generated files

### 12.2 Add JSON output to `provision` command ✅ COMPLETED

**Issue**: [#352](https://github.com/torrust/torrust-tracker-deployer/issues/352) ✅ Merged in [PR #353](https://github.com/torrust/torrust-tracker-deployer/pull/353)
**Specification**: [docs/issues/352-add-json-output-to-provision-command.md](./352-add-json-output-to-provision-command.md)

**Rationale**: Contains the provisioned instance IP address - critical for automation workflows. Easier to parse and extract IP than regex matching console output.

**Key fields** (CRITICAL):

- Instance IP address ✅
- Provider information ✅
- Instance ID/name ✅
- SSH credentials (username, port, key paths) ✅
- Domains (for HTTPS configurations) ✅
- Provisioned timestamp ✅

**Use cases**:

- DNS automation (updating A records with new IP) ✅
- Inventory systems (registering new instances) ✅
- Next-step automation (configure, release, run) ✅
- SSH connection automation ✅

**Priority**: This is often the **most important** command for automation - the IP is needed for all subsequent workflow steps.

**Implementation Details**:

- Strategy Pattern (DTO + TextView + JsonView)
- 22 unit tests + 10 manual tests passing
- 6 automation examples in documentation (Shell, CI/CD, Python, Terraform)
- Backward compatible (text output remains default)

### 12.3 Add JSON output to `show` command

**Rationale**: Contains the instance IP and comprehensive environment state. Structured format makes it simple to query specific fields programmatically.

**Key fields**:

- Current state (Created, Provisioned, Configured, etc.)
- Instance IP address
- Service URLs (tracker, API, Grafana, etc.)
- Configuration summary
- Enabled services

**Use cases**:

- Status monitoring dashboards
- Health check systems
- Deployment verification scripts
- State inspection by AI agents

**Priority**: Most comprehensive output - provides complete deployment picture.

### 12.4 Add JSON output to `run` command

**Rationale**: Contains the list of enabled services and their URLs. Allows automation to verify which services are running and how to access them.

**Key fields**:

- List of enabled/started services
- Service access URLs
- Service status
- Port mappings

**Use cases**:

- Deployment verification (smoke tests)
- Service discovery
- Load balancer configuration
- Monitoring system registration

### 12.5 Add JSON output to `list` command

**Rationale**: Shows full environment names without truncation, enabling unambiguous identification. Table format truncates long names - JSON provides complete information.

**Key fields**:

- Environment names (full, no truncation)
- Current state for each
- Instance IPs (if provisioned)
- Provider information

**Use cases**:

- Environment discovery
- Programmatic environment selection
- Cleanup scripts (identifying destroyed environments)
- Dashboard/UI integration

## Implementation Approach

### Architecture Requirements

- **DDD Layer**: Presentation layer (`src/presentation/`)
- **Module Paths**:
  - Command output formatting: `src/presentation/console/subcommands/{command}/`
  - Shared JSON serialization: `src/presentation/formats/` (new module)
- **Patterns**:
  - Implement `OutputFormat` enum: `HumanReadable`, `Json`
  - Add `--json` flag to CLI argument parsing
  - Create serializable output DTOs for each command
  - Switch format in presentation layer based on flag

### General Implementation Steps

For each command:

1. **Add CLI flag**: Add `--json` flag to command arguments
2. **Create output DTO**: Define Rust struct for JSON output (must implement `Serialize`)
3. **Update presentation**: Add format switching logic (human vs JSON)
4. **Preserve existing**: Ensure default human-readable output unchanged
5. **Document schema**: Add JSON schema examples to user docs
6. **Test**: Verify JSON is valid and parsable

### Code Structure Example

```rust
// src/presentation/formats/mod.rs
pub enum OutputFormat {
    HumanReadable,
    Json,
}

// src/presentation/formats/json.rs
pub trait JsonOutput: Serialize {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// In each command's presentation code
if matches!(format, OutputFormat::Json) {
    let output = CreateCommandOutput { /* ... */ };
    println!("{}", output.to_json()?);
} else {
    // Existing human-readable output
}
```

### Dependencies

- `serde_json`: Already in `Cargo.toml` for state serialization
- No new external dependencies required

## Acceptance Criteria

### For Each Command

- [ ] Command accepts `--json` flag
- [ ] JSON output is valid (parsable by `jq`, `serde_json`, etc.)
- [ ] JSON contains all critical information from human-readable output
- [ ] Default behavior (no flag) unchanged - human-readable output preserved
- [ ] JSON schema documented in user guide
- [ ] Examples provided in documentation

### Quality Standards

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All linters pass (clippy, rustfmt)
- [ ] No new unused dependencies
- [ ] User documentation updated (`docs/user-guide/commands/`)

### Testing

- [ ] Manual testing: commands produce valid JSON with `--json` flag
- [ ] Automated testing: verify JSON structure in E2E tests (future)
- [ ] AI agent testing: validate AI agents can parse output reliably

## Future Enhancements

JSON output can be extended to all commands (`configure`, `release`, `test`, `destroy`, `validate`, `render`, `purge`) based on user demand and use cases. This epic focuses on the high-value commands first.

## Tasks

- [ ] #X - Add JSON output to `create` command (12.1)
- [ ] #X - Add JSON output to `provision` command (12.2)
- [ ] #X - Add JSON output to `show` command (12.3)
- [ ] #X - Add JSON output to `run` command (12.4)
- [ ] #X - Add JSON output to `list` command (12.5)

(Task issues will be created and linked as work progresses)

## Related Documentation

- [Roadmap Section 12](../roadmap.md#12-add-json-output-format-support)
- [Roadmap Section 11 - Improve AI agent experience](../roadmap.md#11-improve-ai-agent-experience)
- [Codebase Architecture](../codebase-architecture.md)
- [Output Handling Conventions](../contributing/output-handling.md)
- [User Guide - Commands](../user-guide/commands/)

## Notes

- This feature was moved from "Deferred Features" to active roadmap based on user feedback
- Implementation benefits from existing MVC architecture - presentation layer already separated
- Format flag pattern could be reused for future output formats (YAML, XML, etc.)
- Consider adding `--format` flag in the future for multiple format support
