---
name: regenerate-cli-docs
description: Regenerate machine-readable CLI documentation JSON after modifying the CLI interface. Use when adding/removing commands, changing arguments, updating help text, or when CLI structure changes. Triggers on "regenerate CLI docs", "update CLI documentation", "generate CLI JSON", or "refresh commands.json".
metadata:
  author: torrust
  version: "1.0"
---

# Regenerate CLI Documentation

This skill helps you regenerate the machine-readable CLI documentation JSON after making changes to the CLI interface.

## When to Use This Skill

Use this skill when:

- **After adding new commands** - Document new functionality in the CLI
- **After removing commands** - Remove outdated command documentation
- **After modifying command arguments** - Update parameter documentation
- **After changing help text** - Ensure descriptions are current
- **Before releasing a new version** - Create documentation snapshot
- **When CLI tests pass** - Ensure documentation matches implementation

## What Gets Regenerated

This skill regenerates:

- **`docs/cli/commands.json`** - Complete CLI structure with all commands, subcommands, arguments, and options
- **Format**: `cli-documentation` v1.0
- **Source**: Automatically extracted from Clap CLI definitions

## Prerequisites

1. **CLI code compiles** - Run `cargo check` first
2. **All tests pass** - Run `cargo test` to verify CLI functionality
3. **Working CLI implementation** - The `docs` command must be functional

## Usage Instructions

### Step 1: Verify CLI Compiles

Before regenerating documentation, ensure the CLI code is valid:

```bash
cargo check
```

### Step 2: Run Tests (Optional but Recommended)

Verify CLI functionality:

```bash
cargo test
```

### Step 3: Regenerate Documentation

Generate the CLI documentation JSON:

```bash
cargo run -- docs docs/cli/commands.json
```

**Expected Output**:

```text
⏳ [1/1] Generating CLI JSON documentation...
⏳   ✓ CLI documentation written to file successfully (took 0ms)
✅ CLI documentation generation completed successfully
```

### Step 4: Verify Generation

Check the file was created/updated:

```bash
ls -lh docs/cli/commands.json
```

Verify JSON structure:

```bash
jq -r '.format, .format_version, .cli.version' docs/cli/commands.json
```

Count documented commands:

```bash
jq '.cli.commands | length' docs/cli/commands.json
```

### Step 5: Review Changes

If regenerating after changes, review what changed:

```bash
git diff docs/cli/commands.json
```

### Step 6: Commit Changes

Stage and commit the updated documentation:

```bash
git add docs/cli/commands.json
git commit -m "docs: update CLI documentation for new commands"
```

## Common Workflows

### Workflow 1: After Adding a New Command

```bash
# 1. Implement new command in src/presentation/input/cli/commands.rs
# 2. Add controller and routing
# 3. Verify compilation
cargo check

# 4. Run tests
cargo test

# 5. Regenerate documentation
cargo run -- docs docs/cli/commands.json

# 6. Review changes
git diff docs/cli/commands.json

# 7. Commit
git add docs/cli/commands.json
git commit -m "docs: update CLI docs for new validate command"
```

### Workflow 2: Before Version Release

```bash
# 1. Ensure all CLI changes are complete and tested
cargo test

# 2. Regenerate documentation for release
cargo run -- docs docs/cli/commands.json

# 3. Verify version in JSON matches release
jq '.cli.version' docs/cli/commands.json

# 4. Commit as part of release preparation
git add docs/cli/commands.json
git commit -m "docs: update CLI documentation for v0.2.0 release"
```

### Workflow 3: CI/CD Integration

```bash
# Generate and verify documentation hasn't changed unexpectedly
cargo run -- docs > generated.json
diff docs/cli/commands.json generated.json || {
  echo "❌ CLI documentation is out of sync!"
  echo "Run: cargo run -- docs docs/cli/commands.json"
  exit 1
}
```

## What the Documentation Includes

The generated JSON includes:

- **Command names** - All top-level commands
- **Subcommands** - Nested command structures (e.g., `create template`, `create environment`)
- **Arguments** - Positional parameters with descriptions
- **Options** - Flags with short/long forms, value types, and help text
- **Descriptions** - Help text for every command and option
- **Metadata** - CLI name, version, and about text

## Output Format

The JSON follows this structure:

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
      },
      {
        "name": "docs",
        "description": "Generate CLI documentation in JSON format",
        "args": [...]
      }
    ]
  }
}
```

## Troubleshooting

### Problem: Command fails with compilation error

**Solution**: Fix compilation errors first:

```bash
cargo check
# Fix any errors
cargo run -- docs docs/cli/commands.json
```

### Problem: JSON format is invalid

**Solution**: Verify with jq:

```bash
jq '.' docs/cli/commands.json
```

If invalid, check infrastructure layer implementation.

### Problem: Documentation missing new commands

**Solution**: Ensure commands are properly registered in Clap:

1. Check `src/presentation/input/cli/commands.rs`
2. Verify enum variants are public
3. Rebuild and regenerate

### Problem: Permission denied writing file

**Solution**: Check directory permissions:

```bash
ls -ld docs/cli/
mkdir -p docs/cli
cargo run -- docs docs/cli/commands.json
```

## Related Commands

- **`docs`** - The CLI command that generates documentation
- **`create schema`** - Generate JSON Schema for environment config (different purpose)
- **`validate`** - Validate environment configuration files

## Related Documentation

- [docs/user-guide/commands/docs.md](../../../docs/user-guide/commands/docs.md) - User guide for the docs command
- [docs/cli/README.md](../../../docs/cli/README.md) - CLI documentation directory overview
- [docs/experiments/cli-json-schema/](../../../docs/experiments/cli-json-schema/) - Implementation history

## Notes

- This command generates documentation from **compiled** code, ensuring accuracy
- The documentation is automatically synchronized with implementation
- No manual editing of `commands.json` should be needed
- The file should be committed to version control for history tracking
- Fast operation (completes in milliseconds)
- Safe to run multiple times (idempotent)

## Success Criteria

✅ `docs/cli/commands.json` file created/updated  
✅ Valid JSON structure  
✅ Format is `cli-documentation` v1.0  
✅ CLI version matches project version  
✅ All commands documented (currently 14 commands)  
✅ Git diff shows expected changes (if updating)

## Best Practices

1. **Regenerate after CLI changes** - Don't forget to update documentation
2. **Review before committing** - Use `git diff` to verify changes
3. **Version control** - Always commit the generated file
4. **CI validation** - Consider adding check in CI to ensure docs are current
5. **Release snapshots** - Commit documentation as part of version releases
