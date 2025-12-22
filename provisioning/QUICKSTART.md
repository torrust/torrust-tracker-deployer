# Torrust Tracker Provisioning - Quick Start Guide

Interactive configuration wizard for Torrust Tracker deployments using Nickel + TypeDialog.

## Prerequisites

Ensure you have these tools installed:

```bash
# Check dependencies
cargo run --bin dependency-installer check

# Install missing dependencies (if needed)
cargo run --bin dependency-installer install
```

## Three Configuration Options

### Option A: Interactive Wizard (Recommended)

Automated workflow with validation:

```bash
./provisioning/scripts/config.sh
```

**What happens:**
1. ✅ Interactive TypeDialog form collects your configuration
2. ✅ Converts responses to Nickel configuration file
3. ✅ Validates all rules with Nickel validators
4. ✅ Exports final JSON to `envs/your-env-name.json`

**Output files:**
- `provisioning/values/your-env-name.ncl` - Nickel source (editable, validated)
- `envs/your-env-name.json` - Final configuration (passed to Rust CLI)

### Option B: Manual JSON Edit

Direct approach (no wizard):

```bash
# 1. Copy example
cp envs/example.json envs/my-environment.json

# 2. Edit values
vim envs/my-environment.json

# 3. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-environment.json
```

### Option C: Advanced Nickel Edit

For users comfortable with Nickel configuration:

```bash
# 1. Create Nickel config
vim provisioning/values/my-environment.ncl

# 2. Validate
./provisioning/scripts/validate-nickel.sh provisioning/values/my-environment.ncl

# 3. Export to JSON
./provisioning/scripts/nickel-to-json.sh provisioning/values/my-environment.ncl envs/my-environment.json

# 4. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-environment.json
```

---

## Wizard Form Sections

When you run the interactive wizard, you'll configure:

### 1. **Environment Identification**
- Environment name (e.g., `dev`, `staging`, `production`)
- Optional instance/VM name (auto-generated if blank)

### 2. **Infrastructure Provider**
- **LXD** (local container):
  - Profile name for LXD configuration
- **Hetzner Cloud** (remote VM):
  - API token, server type, location, OS image

### 3. **SSH Credentials**
- Private key path (e.g., `~/.ssh/id_rsa`)
- Public key path
- SSH username (default: `torrust`)
- SSH port (default: 22)

### 4. **Database**
- **SQLite** (local file):
  - Database filename (default: `tracker.db`)
- **MySQL** (remote/local server):
  - Host, port, database name, username, password

### 5. **Tracker Configuration**
- Private mode (yes/no)
- UDP tracker bind address (e.g., `0.0.0.0:6969`)
- HTTP tracker bind address (e.g., `0.0.0.0:7070`)
- HTTP API bind address (e.g., `0.0.0.0:1212`)
- Admin API token (secret string)

### 6. **Optional Features**
- **Prometheus** monitoring (enable/disable + config)
- **Grafana** visualization (enable/disable + config)

### 7. **Review & Confirm**
- Summary before generation

---

## Generated Files

### After Wizard Completes

```
✅ Configuration Generation Complete!

Generated files:
   - Nickel: provisioning/values/dev.ncl
   - JSON:   envs/dev.json

Next steps:
   1. Review configuration: cat envs/dev.json | jq .
   2. Create environment:   cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/dev.json
   3. Provision:            cargo run --bin torrust-tracker-deployer -- provision dev
```

---

## Common Workflows

### Deploy to LXD (Local)

```bash
# 1. Run wizard (select LXD provider)
./provisioning/scripts/config.sh

# 2. Follow prompts → generates envs/my-env.json

# 3. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json

# 4. Provision tracker
cargo run --bin torrust-tracker-deployer -- provision my-env

# 5. Destroy when done
cargo run --bin torrust-tracker-deployer -- destroy my-env
```

### Deploy to Hetzner Cloud

```bash
# 1. Run wizard (select Hetzner provider)
./provisioning/scripts/config.sh
# → Enter API token, server type (cx22/cx32), location (fsn1/nbg1), image

# 2. Generates envs/prod.json with Hetzner config

# 3. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/prod.json

# 4. Provision
cargo run --bin torrust-tracker-deployer -- provision prod
```

### Multiple Environments

```bash
# Same project, different configurations
./provisioning/scripts/config.sh  # → creates dev.json
./provisioning/scripts/config.sh  # → creates staging.json
./provisioning/scripts/config.sh  # → creates prod.json

# Manage independently
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/dev.json
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/staging.json
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/prod.json
```

---

## Troubleshooting

### Wizard Validation Errors

**Error**: "Environment name cannot start with number"
- **Fix**: Use name like `dev-1` instead of `1-dev`

**Error**: "Port must be 1-65535"
- **Fix**: Use valid port range (1-65535), port 0 not allowed

**Error**: "Bind address must be IP:PORT format"
- **Fix**: Use format like `0.0.0.0:6969` or `127.0.0.1:8080`

### Nickel Validation Fails

```bash
# Debug: see specific validation error
nickel eval provisioning/values/my-env.ncl

# Common issues:
# - Typo in validator name
# - Missing import statement
# - Invalid JSON in generated file
```

### JSON Export Issues

```bash
# Verify Nickel is valid
./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl

# If valid, try export manually
nickel export --format json provisioning/values/my-env.ncl
```

---

## Configuration Files Reference

### Structure

```
provisioning/
├── config-form.toml               # Interactive wizard form
├── fragments/                      # Modular form sections
│   ├── environment-section.toml
│   ├── provider-*.toml
│   ├── database-*.toml
│   └── ...
├── schemas/                        # Type contracts
│   ├── environment.ncl
│   ├── provider.ncl
│   ├── database.ncl
│   └── ...
├── defaults/                       # Default values
│   ├── ssh.ncl
│   ├── tracker.ncl
│   └── features.ncl
├── validators/                     # Validation rules
│   ├── common.ncl
│   ├── environment.ncl
│   ├── username.ncl
│   └── ...
├── values/                         # Generated configs (gitignored)
│   ├── config.ncl
│   └── your-env.ncl
└── scripts/
    ├── config.sh                   # Main wizard (bash)
    ├── json-to-nickel.nu           # JSON → Nickel (Nushell)
    ├── nickel-to-json.nu           # Nickel → JSON (Nushell)
    └── validate-nickel.nu          # Validation (Nushell)
```

### Example Generated Configuration

```json
{
  "environment": {
    "name": "dev",
    "instance_name": "torrust-tracker-vm-dev"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-dev"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "tracker": {
    "core": {
      "private": false,
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      }
    },
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "your-secret-token"
    }
  },
  "features": {
    "prometheus": { "enabled": false },
    "grafana": { "enabled": false }
  }
}
```

---

## Validation Rules

All configurations are validated according to these rules:

| Field | Rules |
|-------|-------|
| Environment Name | Lowercase + numbers + dashes, no leading number/dash |
| Instance Name | 1-63 chars, alphanumeric + dashes, no leading digit/dash |
| SSH Username | 1-32 chars, starts with letter/underscore |
| SSH Port | 1-65535 (port 0 not allowed) |
| Bind Addresses | IP:PORT format with valid port range |
| Database Driver | `sqlite3` or `mysql` |

---

## Next Steps After Configuration

Once you have `envs/your-env.json`:

```bash
# 1. Verify configuration
cat envs/your-env.json | jq .

# 2. Create deployment environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/your-env.json

# 3. Provision infrastructure + software
cargo run --bin torrust-tracker-deployer -- provision your-env

# 4. Deploy tracker release
cargo run --bin torrust-tracker-deployer -- release your-env

# 5. Run tracker
cargo run --bin torrust-tracker-deployer -- run your-env

# 6. Cleanup (when done)
cargo run --bin torrust-tracker-deployer -- destroy your-env
```

See `docs/user-guide/commands/` for detailed command documentation.

---

## Tips & Best Practices

✅ **DO:**
- Use meaningful environment names (`staging`, `production`, not `test123`)
- Store SSH keys with restricted permissions (`chmod 600`)
- Use environment variables for sensitive tokens
- Review generated JSON before deployment

❌ **DON'T:**
- Commit `envs/` directory to version control (contains secrets!)
- Use port 0 (reserved for system)
- Mix providers in same environment
- Share API tokens in repositories

---

## Support

For issues or questions:
- Check `provisioning/README.md` for detailed documentation
- Review `provisioning/values/config.ncl` for advanced examples
- Read `docs/user-guide/` for CLI commands
- Inspect validator rules in `provisioning/validators/`
