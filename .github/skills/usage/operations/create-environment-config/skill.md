---
name: create-environment-config
description: Guide for creating environment configuration files for Torrust Tracker deployments. Covers the JSON structure, required/optional sections, common deployment patterns, and validation. Use when creating config files, generating environment templates, setting up deployment configurations, or helping users configure tracker deployments. Triggers on "create config", "environment configuration", "config template", "setup environment", or "generate config".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Environment Configuration

This skill guides end-users through creating environment configuration files for Torrust Tracker deployments.

## What Are Environment Configurations?

Environment configurations are **user input files** (JSON) that specify deployment requirements:

- Infrastructure provider (LXD local VMs or Hetzner cloud)
- SSH credentials for remote access
- Tracker settings (database, ports, API tokens)
- Optional services (Prometheus monitoring, Grafana dashboards, automated backups)

**Storage**: Create these in `envs/` directory (e.g., `envs/my-deployment.json`)

**Usage**: Passed to `create environment --env-file envs/my-deployment.json`

### Critical Distinction: `envs/` vs `data/`

⚠️ **NEVER CONFUSE THESE TWO DIRECTORIES**:

| Directory | Purpose                   | Format               | Example                                   | Managed By  |
| --------- | ------------------------- | -------------------- | ----------------------------------------- | ----------- |
| `envs/`   | User environment configs  | DTO schema (input)   | `envs/manual-test-mysql.json`             | User/AI     |
| `data/`   | Application state machine | Domain model (state) | `data/manual-test-mysql/environment.json` | Application |

**Rule**: You MAY create/edit files in `envs/`. You MUST NEVER create/edit files in `data/` (read-only).

## Core Configuration Sections

```json
{
  "environment": { ... },        // Required: Name and optional description
  "ssh_credentials": { ... },    // Required: Private key, public key, username, port
  "provider": { ... },           // Required: LXD or Hetzner
  "tracker": { ... },            // Required: Core settings, UDP/HTTP trackers, API
  "prometheus": { ... },         // Optional: Monitoring
  "grafana": { ... },            // Optional: Dashboards (requires Prometheus)
  "backup": { ... },             // Optional: Automated backups
  "https": { ... }               // Optional: TLS settings (required if any service uses TLS)
}
```

### 1. Environment Section (Required)

```json
{
  "environment": {
    "name": "production", // 3-50 chars: lowercase, numbers, hyphens
    "instance_name": null, // Optional: VM instance name override
    "description": "Production tracker with MySQL and monitoring" // Optional
  }
}
```

### 2. SSH Credentials (Required)

```json
{
  "ssh_credentials": {
    "private_key_path": "/path/to/key", // Absolute paths recommended
    "public_key_path": "/path/to/key.pub",
    "username": "torrust", // Default: "torrust"
    "port": 22 // Default: 22
  }
}
```

**For testing**: Use fixture keys in `fixtures/testing_rsa` and `fixtures/testing_rsa.pub`

### 3. Provider Section (Required)

#### Option A: LXD (local/on-premises)

```json
{
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-dev" // Naming: torrust-profile-{env-name}
  }
}
```

#### Option B: Hetzner (cloud)

```json
{
  "provider": {
    "provider": "hetzner",
    "api_token": "your-api-token", // Read-write token
    "location": "nbg1", // nbg1, fsn1, hel1, ash
    "server_type": "cx22", // cx22, cx32, cx42
    "image": "ubuntu-22.04" // Operating system
  }
}
```

### 4. Tracker Section (Required)

**Minimal UDP-only tracker**:

```json
{
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [{ "bind_address": "0.0.0.0:6969" }]
  }
}
```

**Full-featured with HTTP and API**:

```json
{
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "mysql",
        "port": 3306,
        "database_name": "torrust",
        "username": "root",
        "password": "root_secret_password"
      },
      "private": false
    },
    "udp_trackers": [{ "bind_address": "0.0.0.0:6969" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "127.0.0.1:1313"
    }
  }
}
```

**Database drivers**: `sqlite3` (no external DB), `mysql` (requires host/port/credentials)

### 5. Optional Services

**Prometheus** (metrics collection):

```json
{
  "prometheus": {
    "scrape_interval_in_secs": 15 // Must be > 0
  }
}
```

**Grafana** (dashboards - requires Prometheus):

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin"
  }
}
```

**Backup** (automated database backups):

```json
{
  "backup": {
    "schedule": "0 3 * * *", // Cron format (3 AM daily)
    "retention_days": 7 // Must be > 0
  }
}
```

**HTTPS** (required if any service has TLS configured):

```json
{
  "https": {
    "admin_email": "admin@example.com" // For Let's Encrypt
  }
}
```

## Common Deployment Patterns

Reference `docs/ai-training/dataset/environment-configs/` for complete examples:

| Pattern                  | Database | Monitoring | Backup | Example File                              |
| ------------------------ | -------- | ---------- | ------ | ----------------------------------------- |
| **Minimal Dev**          | SQLite   | No         | No     | `01-minimal-lxd-sqlite-udp-only.json`     |
| **Full Local**           | MySQL    | Full       | Yes    | `05-lxd-mysql-full-stack-backup.json`     |
| **Production Cloud**     | MySQL    | Full       | Yes    | `07-hetzner-mysql-monitoring-backup.json` |
| **High Performance UDP** | SQLite   | Basic      | No     | `02-lxd-sqlite-udp-http.json`             |

**Decision factors**:

- **SQLite** → Fast setup, no external DB, single file
- **MySQL** → Production-ready, scalable, requires container
- **Prometheus/Grafana** → Adds monitoring containers, valuable for production
- **Backup** → Essential for production, cron-based schedule

## Creating a New Configuration

### Method 1: Generate from Template (Recommended)

Generate a **fully-featured template** with all sections, then remove what you don't need:

```bash
# Generate LXD template
torrust-tracker-deployer create template --provider lxd envs/my-deployment.json

# Or generate Hetzner template
torrust-tracker-deployer create template --provider hetzner envs/my-deployment.json

# Edit and replace placeholders
vim envs/my-deployment.json
```

**Template placeholders to replace**:

- `REPLACE_WITH_ENVIRONMENT_NAME` - Your environment name (lowercase, dashes allowed)
- `REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH` - Absolute path to private key
- `REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH` - Absolute path to public key
- `REPLACE_WITH_LXD_PROFILE_NAME` (LXD) or `REPLACE_WITH_HETZNER_API_TOKEN` (Hetzner)

### Method 2: Copy from Examples

Start from a pre-configured example matching your use case:

```bash
# Copy a complete example
cp docs/ai-training/dataset/environment-configs/01-minimal-lxd-sqlite-udp-only.json \
   envs/my-deployment.json

# Customize key fields
vim envs/my-deployment.json
```

## Validation

### Validate Command (Recommended)

**Fast syntax and domain validation** without creating infrastructure:

```bash
torrust-tracker-deployer validate --env-file envs/my-deployment.json
```

**Validates**: JSON syntax, environment name format, SSH key existence, ports, required fields, cross-field constraints, and domain rules.

### Create and Destroy (Full Validation)

```bash
torrust-tracker-deployer create environment --env-file envs/my-deployment.json
torrust-tracker-deployer show my-deployment
torrust-tracker-deployer destroy my-deployment
torrust-tracker-deployer purge my-deployment
```

### Common Validation Errors

- **"scrape_interval must be greater than 0"** → Use a positive integer
- **"Invalid profile name format"** → Must match `[a-z0-9-]+` pattern
- **"SSH private key does not exist"** → Path must be absolute and file must exist
- **"Grafana requires Prometheus"** → Cannot enable Grafana without Prometheus
- **"TLS configured but no admin_email"** → HTTPS section required when any service uses TLS

## Related Documentation

- **Create Command**: `docs/user-guide/commands/create.md`
- **Validate Command**: `docs/user-guide/commands/validate.md`
- **AI Training Examples**: `docs/ai-training/README.md` - 15 pre-configured deployment patterns
- **User Questionnaire**: `docs/ai-training/questionnaire.md` - Structured decision tree
- **JSON Schema**: `schemas/environment-config.json` - For IDE autocomplete

## See Also

- For **DTO architecture internals**: see the `environment-config-architecture` skill in `dev/infrastructure/`
