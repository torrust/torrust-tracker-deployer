# `docs` - Generate CLI Documentation

Generate machine-readable JSON documentation of the CLI interface structure.

## Purpose

Provides a complete, structured description of all available commands, arguments, and options in JSON format. This documentation is automatically generated from the actual CLI code, ensuring it always stays synchronized with the implementation.

## Command Syntax

```bash
torrust-tracker-deployer docs [OUTPUT_PATH]
```

## Arguments

- `[OUTPUT_PATH]` (optional) - Path to write JSON documentation file. If omitted, outputs to stdout.

## Prerequisites

None. This command can be run without any environment setup.

## Use Cases

### For AI Agents

AI coding assistants can consume this JSON to understand the complete CLI interface:

```bash
# Generate docs for AI consumption
torrust-tracker-deployer docs > cli-interface.json
```

The JSON structure includes:

- All available commands and subcommands
- Argument names, types, and descriptions
- Option flags with short/long forms
- Help text for every command

### For Documentation Tools

Generate versioned CLI documentation that can be tracked in version control:

```bash
# Generate documentation for version tracking
torrust-tracker-deployer docs docs/cli/commands.json
git add docs/cli/commands.json
git commit -m "docs: update CLI documentation for v0.2.0"
```

### For IDE Integration

Shell completion scripts or IDE plugins can use this to provide autocomplete:

```bash
# Generate for IDE tooling
torrust-tracker-deployer docs > ~/.config/torrust-cli/commands.json
```

### For Testing

Automated tests can validate CLI structure consistency:

```bash
# Generate for test validation
torrust-tracker-deployer docs | jq '.cli.commands | length'
```

## JSON Format

The generated JSON follows this structure:

```json
{
  "format": "cli-documentation",
  "format_version": "1.0",
  "cli": {
    "name": "torrust-tracker-deployer",
    "version": "0.1.0",
    "about": "Deploy and manage Torrust Tracker instances",
    "commands": [
      {
        "name": "create",
        "description": "Create environments and resources",
        "subcommands": [...],
        "args": [...]
      }
    ]
  }
}
```

Key fields:

- **`format`**: Document type identifier (`cli-documentation`)
- **`format_version`**: Documentation format version
- **`cli.name`**: Application name
- **`cli.version`**: Application version
- **`cli.commands`**: Array of all top-level commands with their structure

## Examples

### Output to stdout

```bash
torrust-tracker-deployer docs
```

**Output**: JSON printed to stdout (can be piped to other tools)

### Save to file

```bash
torrust-tracker-deployer docs docs/cli/commands.json
```

**Result**: Documentation written to `docs/cli/commands.json`

### Query with jq

```bash
# List all command names
torrust-tracker-deployer docs | jq '.cli.commands[].name'

# Find commands with subcommands
torrust-tracker-deployer docs | jq '.cli.commands[] | select(.subcommands) | .name'

# Get help text for a specific command
torrust-tracker-deployer docs | jq '.cli.commands[] | select(.name == "create")'
```

### Compare versions

```bash
# Compare CLI changes between versions
git show v0.1.0:docs/cli/commands.json > old.json
torrust-tracker-deployer docs > new.json
diff <(jq -S . old.json) <(jq -S . new.json)
```

## Output Format

### Text Output

Not applicable - this command only generates JSON output.

### JSON Output

Complete CLI structure in JSON format. Use `-o json` flag for other commands if you need JSON output from operational commands.

## Common Workflows

### Integrating with AI Tools

```bash
# 1. Generate CLI documentation
torrust-tracker-deployer docs > cli-docs.json

# 2. Provide to AI agent/tool
# AI can now understand all available commands and their usage
```

### Tracking CLI Changes

```bash
# 1. After modifying CLI structure
cargo build

# 2. Regenerate documentation
torrust-tracker-deployer docs docs/cli/commands.json

# 3. Review changes
git diff docs/cli/commands.json

# 4. Commit with version
git add docs/cli/commands.json
git commit -m "docs: update CLI interface for new validate command"
```

### Validation in CI/CD

```bash
# Verify CLI structure hasn't changed unexpectedly
torrust-tracker-deployer docs > generated.json
diff expected-cli.json generated.json || {
  echo "CLI interface has changed!"
  exit 1
}
```

## When to Use This Command

- **Before sharing with AI agents**: Help AI understand your CLI interface
- **After CLI changes**: Document interface modifications
- **For version releases**: Include CLI documentation snapshot
- **When building tooling**: Provide structured CLI data for autocomplete/plugins

## Related Commands

- **`create template`** - Generate environment configuration templates
- **`validate`** - Validate environment configuration files
- **`show`** - Display environment information

## Notes

- This command generates documentation from the **compiled** CLI code
- Documentation is always accurate and up-to-date with implementation
- No network access or environment setup required
- Fast operation (completes in milliseconds)
- Safe to run in any context (read-only, no side effects)

## Comparison: `docs` vs JSON Schema

This command is different from JSON schemas used elsewhere:

| Aspect       | `docs` command           | `create schema` command      |
| ------------ | ------------------------ | ---------------------------- |
| **Purpose**  | Document CLI structure   | Validate config files        |
| **Output**   | CLI command hierarchy    | JSON Schema for validation   |
| **Use case** | AI agents, documentation | IDE autocomplete, validation |
| **Format**   | Custom JSON format       | JSON Schema standard         |
| **Target**   | CLI interface            | User configuration files     |

For validating environment configuration files, use `create schema` instead.
