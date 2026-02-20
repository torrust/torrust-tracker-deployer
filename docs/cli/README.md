# CLI Documentation

This directory contains machine-readable documentation of the CLI interface.

## Files

- **`commands.json`** - Complete CLI structure including all commands, arguments, and options

## Format

The JSON file uses a custom format specifically designed for CLI documentation:

```json
{
  "format": "cli-documentation",
  "format_version": "1.0",
  "cli": {
    "name": "torrust-tracker-deployer",
    "version": "0.1.0",
    "about": "Deploy and manage Torrust Tracker instances",
    "commands": [...]
  }
}
```

## Usage

### For AI Agents

AI coding assistants can read this file to understand all available commands:

```bash
jq '.cli.commands[] | {name, description}' docs/cli/commands.json
```

### For Documentation

Track CLI interface changes across versions:

```bash
git log -p docs/cli/commands.json
```

### For Validation

Verify CLI structure in automated tests:

```bash
jq '.cli.commands | length' docs/cli/commands.json
```

## Regenerating

After modifying the CLI interface, regenerate this file:

```bash
cargo run -- docs docs/cli/commands.json
```

The documentation is automatically generated from the actual CLI code using Clap's introspection capabilities.

## Version History

This file should be committed whenever the CLI interface changes, creating a version history of the command structure.
