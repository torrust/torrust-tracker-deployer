# JSON Schemas

This directory contains JSON Schema files for validating configuration files used in this project.

## Environment Configuration Schema

**File**: `environment-config.json`

This schema validates user-provided environment configuration files (stored in the `envs/` directory). It ensures that configuration files have:

- Correct structure and required fields
- Valid provider configurations (LXD, Hetzner)
- Proper SSH credentials format
- Valid tracker configuration

## Regenerating the Schema

To regenerate the schema after code changes:

```bash
cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
```

**When to regenerate:**

- After adding new configuration fields
- After changing validation rules or types
- After modifying enums or provider options

## IDE Setup

For instructions on configuring your IDE to use this schema for autocomplete and validation, see:

📖 **[JSON Schema IDE Setup Guide](../docs/user-guide/json-schema-ide-setup.md)**

## What This Schema Validates

The schema applies to files matching the pattern `envs/*.json` and validates:

- **Environment settings**: Name, instance name
- **SSH credentials**: Key paths, username, port
- **Provider configuration**: LXD profiles or Hetzner server settings
- **Tracker configuration**: Database, UDP/HTTP trackers, API settings

**Note**: This schema does NOT apply to internal application state files (`data/*/environment.json`), which have a different structure.
