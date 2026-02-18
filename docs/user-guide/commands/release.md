# `release` - Deploy Application Configuration

Deploy application files and configuration to a configured environment.

## Purpose

Deploys the Torrust Tracker application configuration, storage directories, and Docker Compose files to the provisioned and configured VM. This command takes an environment from the "Configured" state to the "Released" state with all application files in place.

The release command prepares the application layer without starting services - that's the job of the `run` command.

## Command Syntax

```bash
torrust-tracker-deployer release <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to release

## Prerequisites

1. **Environment configured** - Must run `configure` command first
2. **VM accessible** - SSH connectivity to the provisioned instance
3. **Docker installed** - Docker and Docker Compose must be installed (done by `configure`)

## State Transition

```text
[Configured] --release--> [Released]
```

## What Happens

When you release an environment:

1. **Creates storage directories** - Sets up tracker data directories (`/opt/torrust/storage/tracker/`)
   - `etc/` - Configuration files
   - `lib/database/` - SQLite database
   - `log/` - Log files

2. **Initializes SQLite database** - Creates empty tracker database file

3. **Renders tracker templates** - Generates configuration from environment settings
   - `tracker.toml` - Tracker configuration
   - `.env` - Docker Compose environment variables

4. **Deploys configuration files** - Copies files to VM
   - `/opt/torrust/storage/tracker/etc/tracker.toml`
   - `/opt/torrust/.env`

5. **Deploys Docker Compose files** - Synchronizes docker-compose stack
   - `/opt/torrust/docker-compose.yml`

## Directory Structure Created

```text
/opt/torrust/
├── .env                                    # Docker Compose environment variables
├── docker-compose.yml                      # Docker Compose service definitions
└── storage/
    └── tracker/
        ├── etc/
        │   └── tracker.toml               # Tracker configuration
        ├── lib/
        │   └── database/
        │       └── tracker.db             # SQLite database
        └── log/                           # Log files (created at runtime)
```

### Backup Configuration Deployment

If backup is enabled in your environment configuration, the release command also deploys backup service configuration:

```text
/opt/torrust/storage/backup/
├── etc/
│   ├── backup.conf                   # Backup service configuration
│   └── backup-paths.txt              # Paths to backup
└── sqlite/                           # SQLite database backups (created at runtime)
    └── (backup files created during run)
```

**What gets deployed for backup:**

- Backup configuration file with database type and retention settings
- Backup paths file with list of directories to backup
- Empty backup directories (sqlite/, mysql/, config/) for backup files
- Crontab entry for scheduled backup execution (installed by `run` command)

**Note**: Initial backup files are created when the `run` command executes, not during release.

## Example Usage

### Basic Release

```bash
# Release after configuration
torrust-tracker-deployer release full-stack-docs
```

**Output**:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: full-stack-docs (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 27.4s)
✅ Release command completed successfully for 'full-stack-docs'
```

### Complete Workflow

```bash
# 1. Create environment
torrust-tracker-deployer create template --provider lxd > my-env.json
# Edit my-env.json with your settings
torrust-tracker-deployer create environment --env-file my-env.json

# 2. Provision infrastructure
torrust-tracker-deployer provision my-environment

# 3. Configure system
torrust-tracker-deployer configure my-environment

# 4. Release application
torrust-tracker-deployer release my-environment

# 5. Start services (next step)
torrust-tracker-deployer run my-environment
```

## Verbosity Levels

The release command supports multiple verbosity levels to control the amount of progress detail displayed:

### Default (Normal) - Essential Progress Only

Shows only the essential progress and results:

```bash
torrust-tracker-deployer release my-environment
```

**Output**:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: my-environment (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 45.8s)
✅ Release command completed successfully for 'my-environment'
```

### Verbose (`-v`) - Show Service Release Steps

Shows all 7 service-specific release steps:

```bash
torrust-tracker-deployer release my-environment -v
```

**Output**:

```text
⏳ [2/2] Releasing application...
📋   [Step 1/7] Releasing Tracker service...
📋   [Step 2/7] Releasing Prometheus service...
📋   [Step 3/7] Releasing Grafana service...
📋   [Step 4/7] Releasing MySQL service...
📋   [Step 5/7] Releasing Backup service...
📋   [Step 6/7] Releasing Caddy service...
📋   [Step 7/7] Deploying Docker Compose configuration...
⏳   ✓ Application released successfully (took 43.2s)
```

**Use Case**: When you want visibility into which service is being deployed.

### Very Verbose (`-vv`) - Show Detailed Operations

Shows template rendering, file paths, and deployment details:

```bash
torrust-tracker-deployer release my-environment -vv
```

**Output** (excerpt):

```text
📋   [Step 1/7] Releasing Tracker service...
📋      → Creating storage directories: /opt/torrust/storage/tracker/{lib,log,etc}
📋      → Initializing database: tracker.db
📋      → Rendering tracker.toml from template
📋      → Deploying config to /opt/torrust/storage/tracker/etc/tracker.toml
📋   [Step 2/7] Releasing Prometheus service...
📋      → Rendering prometheus.yml from template
📋      → Deploying config to /opt/torrust/storage/prometheus/etc/prometheus.yml
📋   [Step 7/7] Deploying Docker Compose configuration...
📋      → Rendering docker-compose.yml (7 services enabled)
📋      → Rendering .env file (12 environment variables)
📋      → Deploying to /opt/torrust/docker-compose.yml
```

**Use Case**: Troubleshooting release issues or verifying what files are being deployed where.

### Debug (`-vvv`) - Show Technical Details

Shows Ansible commands, working directories, and full execution details:

```bash
torrust-tracker-deployer release my-environment -vvv
```

**Output** (excerpt):

```text
📋   [Step 1/7] Releasing Tracker service...
🔍      → Ansible working directory: ./build/my-environment/ansible
🔍      → Executing playbook: ansible-playbook create-tracker-storage.yml
📋      → Creating storage directories: /opt/torrust/storage/tracker/{lib,log,etc}
🔍      → Executing playbook: ansible-playbook init-tracker-database.yml
📋      → Initializing database: tracker.db
🔍      → Template source: ./templates/tracker/tracker.toml.tera
🔍      → Template output: ./build/my-environment/tracker/tracker.toml
📋      → Rendering tracker.toml from template
🔍      → Executing playbook: ansible-playbook deploy-tracker-config.yml
📋      → Deploying config to /opt/torrust/storage/tracker/etc/tracker.toml
```

**Use Case**: Deep troubleshooting, debugging, or when you need to understand exactly what commands are being executed.

### Symbol Legend

| Symbol | Meaning                          | Verbosity Level |
| ------ | -------------------------------- | --------------- |
| ⏳     | Operation in progress            | Normal+         |
| ✅     | Operation completed successfully | Normal+         |
| 📋     | Detailed contextual information  | VeryVerbose     |
| 🔍     | Technical implementation detail  | Debug           |

### Combining with Other Options

Verbosity flags can be combined with other command options:

```bash
# Very verbose release with trace logging
RUST_LOG=trace torrust-tracker-deployer release my-environment -vv
```

**Note**: Verbosity flags (`-v`, `-vv`, `-vvv`) control user-facing progress output, while `RUST_LOG` controls internal application logging for debugging purposes.

```bash
# 1. Create environment
torrust-tracker-deployer create template --provider lxd > my-env.json
# Edit my-env.json with your settings
torrust-tracker-deployer create environment --env-file my-env.json

# 2. Provision infrastructure
torrust-tracker-deployer provision my-environment

# 3. Configure system
torrust-tracker-deployer configure my-environment

# 4. Release application
torrust-tracker-deployer release my-environment

# 5. Start services (next step)
torrust-tracker-deployer run my-environment
```

## What Gets Configured

### Tracker Configuration (`tracker.toml`)

The release command generates a complete tracker configuration based on your environment settings:

- **Database**: SQLite database path and settings
- **UDP Trackers**: Bind addresses for BitTorrent UDP announce
- **HTTP Trackers**: Bind addresses for BitTorrent HTTP announce
- **HTTP API**: Admin API endpoint and authentication
- **Core Settings**: Private/public mode, announce intervals, policies

### Environment Variables (`.env`)

Docker Compose environment variables are configured:

- `TORRUST_TRACKER_CONFIG_TOML_PATH` - Path to tracker configuration
- `TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN` - API admin token

### Docker Compose Stack

The docker-compose.yml defines:

- **Tracker service**: Torrust Tracker container with proper ports and volumes
- **Network**: Backend network for service communication
- **Volumes**: Persistent storage for database, logs, and configuration

## Verification

After releasing, you can verify the deployment:

```bash
# Get VM IP address
VM_IP=$(torrust-tracker-deployer show my-environment | grep 'IP Address' | awk '{print $3}')

# SSH into VM and check files
ssh -i ~/.ssh/your-key user@$VM_IP "ls -la /opt/torrust/"

# Expected output shows .env and docker-compose.yml files

# Check tracker storage directories
ssh -i ~/.ssh/your-key user@$VM_IP "find /opt/torrust/storage/tracker -type f"

# Expected: tracker.toml and tracker.db files
```

## Troubleshooting

### Release Fails with "Environment not configured"

**Problem**: Trying to release before running configure command.

**Solution**:

```bash
# Run configure first
torrust-tracker-deployer configure my-environment
# Then try release again
torrust-tracker-deployer release my-environment
```

### Release Fails with SSH Connection Error

**Problem**: Cannot connect to VM via SSH.

**Solution**:

```bash
# Verify VM is running
torrust-tracker-deployer show my-environment

# Test SSH connectivity manually
ssh -i path/to/your-key user@<vm-ip> "echo test"

# Check firewall rules allow SSH (port 22)
```

### Files Not Deployed to VM

**Problem**: Template rendering succeeds but files not on VM.

**Solution**:

```bash
# Check build directory has rendered files
ls -la build/my-environment/tracker/
ls -la build/my-environment/docker-compose/

# Re-run release with verbose logging
RUST_LOG=debug torrust-tracker-deployer release my-environment

# Check Ansible playbook execution in logs
```

## Configuration Customization

The release command uses your environment configuration from the JSON file:

```json
{
  "environment": {
    "name": "my-environment"
  },
  "tracker": {
    "core": {
      "database_name": "tracker.db",
      "private": false
    },
    "udp_trackers": [
      { "bind_address": "0.0.0.0:6868" },
      { "bind_address": "0.0.0.0:6969" }
    ],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    }
  }
}
```

To customize tracker behavior, edit your environment JSON file and re-run `release`.

## Next Steps

After releasing:

1. **Start services** - Use `run` command to start the tracker
2. **Verify tracker** - Check tracker API responds to health checks
3. **Test announce** - Verify BitTorrent clients can announce to tracker

## Related Commands

- [`configure`](configure.md) - Configure system (required before release)
- [`run`](run.md) - Start tracker services (next step after release)
- [`create`](create.md) - Create environment configuration
- [`destroy`](destroy.md) - Clean up deployment

## Technical Details

The release command executes these steps in order:

1. **Render tracker templates** (`RenderTrackerTemplatesStep`)
2. **Render Docker Compose templates** (`RenderDockerComposeTemplatesStep`)
3. **Create tracker storage directories** (`CreateTrackerStorageStep`)
4. **Initialize tracker database** (`InitTrackerDatabaseStep`)
5. **Deploy tracker configuration** (`DeployTrackerConfigStep`)
6. **Deploy Docker Compose files** (`DeployComposeFilesStep`)

All steps are idempotent - you can safely re-run `release` to update configuration.
