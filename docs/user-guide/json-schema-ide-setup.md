# JSON Schema IDE Setup Guide

This guide shows you how to get autocomplete, validation, and documentation tooltips in your IDE when editing environment configuration files.

## Quick Start

### 1. Generate the Schema

```bash
cargo run --bin torrust-tracker-deployer -- create schema > envs/environment-schema.json
```

This creates a JSON Schema file that describes the structure and validation rules for environment configurations.

### 2. VS Code Setup (Already Configured)

The repository includes `.vscode/settings.json` with JSON schema mappings. VS Code will automatically provide:

- **Autocomplete**: Press `Ctrl+Space` to see available fields
- **Validation**: Red underlines for invalid values or missing required fields
- **Documentation**: Hover over fields to see descriptions and examples
- **Enum values**: Dropdown suggestions for fields with limited options

### 3. Test It Out

Open any environment file in the `envs/` directory:

```bash
code envs/example.json
```

Try these actions:

- **Start typing** a field name - you'll see autocomplete suggestions
- **Hover** over a field - you'll see documentation from the schema
- **Remove** a required field - you'll see a validation error
- **Type** an invalid value - you'll get instant feedback

## File Patterns Covered

The schema automatically applies to:

- `envs/*.json` - User-provided environment configuration files

**Important**: The schema does NOT apply to `data/*/environment.json` files. Those are internal application state files with a different structure containing additional runtime information beyond the user-provided configuration.

## Manual Schema Association

If you want to use the schema in a file outside the `envs/` directory, add this at the top of your JSON file:

```json
{
  "$schema": "../envs/environment-schema.json",
  "environment": {
    ...
  }
}
```

## Other IDEs

### IntelliJ IDEA / CLion / RustRover

1. Open **Settings** → **Languages & Frameworks** → **Schemas and DTDs** → **JSON Schema Mappings**
2. Click **+** to add a new mapping
3. Set **Schema file or URL**: `envs/environment-schema.json`
4. Add file pattern: `envs/*.json`

### Neovim with LSP

Add to your LSP configuration:

```lua
require('lspconfig').jsonls.setup({
  settings = {
    json = {
      schemas = {
        {
          fileMatch = { "envs/*.json" },
          url = "file:///absolute/path/to/envs/environment-schema.json"
        }
      }
    }
  }
})
```

## Keeping Schema Updated

Regenerate the schema whenever you:

- Add new configuration fields
- Change validation rules
- Update enum values
- Modify field types

```bash
# Quick regeneration
cargo run --bin torrust-tracker-deployer -- create schema > envs/environment-schema.json
```

## Benefits

✅ **Fewer errors**: Catch configuration mistakes before running commands
✅ **Faster editing**: Autocomplete reduces typing and lookups
✅ **Self-documenting**: Descriptions and examples right in your editor
✅ **Type safety**: Validation ensures correct types for all fields
✅ **Discoverability**: See all available options without reading docs

## Troubleshooting

### Schema not working in VS Code

1. Reload the window: `Ctrl+Shift+P` → "Developer: Reload Window"
2. Check `.vscode/settings.json` exists with correct schema mapping
3. Verify the schema file exists at `envs/environment-schema.json`

### Autocomplete not showing

1. Make sure you're editing a `.json` file
2. Check the file is in the `envs/` directory and matches the pattern `envs/*.json`
3. Try `Ctrl+Space` to manually trigger autocomplete

### Validation errors seem wrong

The schema might be outdated. Regenerate it:

```bash
cargo run --bin torrust-tracker-deployer -- create schema > envs/environment-schema.json
```
