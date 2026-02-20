# JSON Schemas

This directory contains the JSON Schema file used for validating user environment configuration files.

---

## Environment Configuration Schema

**File**: `environment-config.json`

**Purpose**: Validates the JSON files that users create to define deployment environments.

### What It's For

This schema validates user-provided environment configuration files stored in the `envs/` directory. These are the configuration files you create when you want to deploy a new Torrust Tracker instance.

**Example file**: `envs/my-deployment.json`

### What It Validates

- **Environment settings**: Name, instance name
- **SSH credentials**: Key paths, username, port
- **Provider configuration**: LXD profiles or Hetzner server settings
- **Tracker configuration**: Database, UDP/HTTP trackers, API settings

### How to Use It

**In your IDE** (VS Code, IntelliJ, etc.):

Configure your editor to associate `envs/*.json` files with this schema for autocomplete and validation. See the [JSON Schema IDE Setup Guide](../docs/user-guide/json-schema-ide-setup.md) for detailed instructions.

**Creating a new environment**:

```bash
# 1. Create your configuration file in envs/
vim envs/my-deployment.json

# 2. Your IDE will provide autocomplete using this schema

# 3. Deploy using your configuration
cargo run -- create environment --env-file envs/my-deployment.json
```

### Regenerating the Schema

```bash
cargo run -- create schema > schemas/environment-config.json
```

**When to regenerate:**

- After adding new configuration fields
- After changing validation rules or types
- After modifying enums or provider options

**Important Note**: This schema does NOT apply to internal application state files (`data/*/environment.json`), which have a different structure managed by the application.

---

## Notes

For CLI documentation, see the `docs` command which generates machine-readable documentation of the CLI interface.

## Additional Resources

📖 **[JSON Schema IDE Setup Guide](../docs/user-guide/json-schema-ide-setup.md)** - Configure your IDE for environment configuration autocomplete
