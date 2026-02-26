# `render` - Generate Deployment Artifacts

Generate deployment artifacts without provisioning infrastructure.

## Purpose

Creates all deployment artifacts (OpenTofu, Ansible, Docker Compose, configuration files) for a deployment environment **without actually provisioning infrastructure**. This command is useful for:

- **Previewing artifacts** before committing to infrastructure provisioning
- **Manual deployment workflows** where you want to use generated artifacts with external tools
- **Validation and inspection** of what will be deployed
- **Generating from config files** without creating persistent environment state

## Command Syntax

```bash
# From existing environment (Created state)
torrust-tracker-deployer render --env-name <NAME> --instance-ip <IP> --output-dir <PATH>

# From configuration file (no environment creation)
torrust-tracker-deployer render --env-file <PATH> --instance-ip <IP> --output-dir <PATH>

# Overwrite existing output directory
torrust-tracker-deployer render --env-name <NAME> --instance-ip <IP> --output-dir <PATH> --force
```

## Arguments

### Input Mode (choose one)

- `--env-name <NAME>` - Name of existing environment in Created state
- `--env-file <PATH>` - Path to environment configuration file

**Note**: These options are mutually exclusive - use one or the other.

### Required Parameters

- `--instance-ip <IP>` (required) - Target instance IP address for deployment
- `--output-dir <PATH>` (required) - Output directory for generated artifacts

The IP address is required because:

- In Created state, infrastructure doesn't exist yet (no real IP)
- With config file, no infrastructure is ever created
- IP is needed for Ansible inventory generation

The output directory is required to:

- **Prevent conflicts** with provision artifacts in `build/{env}/`
- **Enable preview** without overwriting deployment artifacts
- **Allow multiple renders** with different IPs or configurations
- **Clear separation** between preview (render) and deployment (provision)

### Optional Flags

- `--force` - Overwrite existing output directory (without this, command fails if directory exists)

## Prerequisites

### For `--env-name` Mode

1. **Environment created** - Environment must exist in "Created" state
2. **Configuration valid** - Environment configuration must be valid

### For `--env-file` Mode

1. **Configuration file** - Valid environment configuration JSON file
2. **No environment needed** - Does not require existing environment

### Common Requirements

- **Templates available** - Template files in `templates/` directory
- **Write permissions** - Ability to write to output directory
- **Output directory** - Must not exist (unless `--force` specified)

## State Transition

```text
[Created] --render--> [Created]  (No state change)
```

**Important**: The render command is **read-only** - it never changes environment state.

## What Happens

When you render artifacts:

1. **Validates input** - Checks environment exists or config file valid
2. **Parses configuration** - Loads environment configuration
3. **Validates IP address** - Ensures IP is in valid format (IPv4/IPv6)
4. **Renders templates** - Generates all deployment artifacts:
   - **OpenTofu** infrastructure code
   - **Ansible** playbooks and inventory
   - **Docker Compose** service definitions
   - **Tracker** configuration (tracker.toml)
   - **Prometheus** monitoring configuration
   - **Grafana** dashboard provisioning
   - **Caddy** reverse proxy configuration (if HTTPS enabled)
   - **Backup** scripts (if backup enabled)
5. **Writes artifacts** - Saves generated files to specified output directory

## Examples

### Preview before provisioning

```bash
# Create environment
torrust-tracker-deployer create environment -f envs/my-config.json

# Preview artifacts with test IP in separate directory
torrust-tracker-deployer render --env-name my-env --instance-ip 192.168.1.100 --output-dir ./preview-my-env

# Review generated artifacts
ls -la preview-my-env/

# If satisfied, provision for real (writes to build/my-env/)
torrust-tracker-deployer provision my-env
```

### Generate from config file without environment

```bash
# Generate artifacts directly from config
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 10.0.0.5 \
  --output-dir /tmp/production-artifacts

# Artifacts in /tmp/production-artifacts/ (no environment created in data/)
```

### Multiple target IPs for comparison

```bash
# Preview with different IPs
torrust-tracker-deployer render \
  --env-name my-env \
  --instance-ip 192.168.1.10 \
  --output-dir ./preview-ip-10

torrust-tracker-deployer render \
  --env-name my-env \
  --instance-ip 10.0.1.20 \
  --output-dir ./preview-ip-20

# Compare artifacts
diff -r preview-ip-10/ preview-ip-20/
```

### Inspect specific artifacts

```bash
# Render artifacts
torrust-tracker-deployer render --env-name my-env --instance-ip 192.168.1.100 --output-dir ./inspect

# Check OpenTofu configuration
cat inspect/tofu/main.tf

# Check Ansible inventory (should show 192.168.1.100)
cat inspect/ansible/inventory.yml

# Check Docker Compose services
cat inspect/docker-compose/docker-compose.yml

# Check tracker configuration
cat inspect/tracker/tracker.toml
```

## Output

The render command generates artifacts in the specified output directory:

### Directory Structure

```text
<output-dir>/
├── tofu/                    # Infrastructure as code
│   └── main.tf              # OpenTofu configuration
├── ansible/                 # Configuration management
│   ├── inventory.yml        # Ansible inventory (with target IP)
│   └── playbooks/           # Ansible playbooks
├── docker-compose/          # Container orchestration
│   ├── docker-compose.yml   # Service definitions
│   └── .env                 # Environment variables
├── tracker/                 # Tracker configuration
│   └── tracker.toml         # Tracker settings
├── prometheus/              # Metrics collection
│   └── prometheus.yml       # Prometheus configuration
├── grafana/                 # Visualization
│   ├── dashboards/          # Dashboard JSON files
│   └── provisioning/        # Datasources
├── caddy/                   # Reverse proxy (if HTTPS enabled)
│   └── Caddyfile            # Caddy configuration
└── backup/                  # Backup (if enabled)
    └── backup.sh            # Backup script
```

### Key Files

- **Ansible inventory** (`ansible/inventory.yml`) - Contains the target IP you specified
- **OpenTofu state** - Infrastructure code ready for `tofu apply`
- **Docker Compose** - Complete service stack definition
- **Configuration files** - All service configurations rendered

### JSON Output

Use `--output-format json` (or `-o json`) to get machine-readable output. Progress messages go to stderr; the JSON result goes to stdout.

```bash
torrust-tracker-deployer render \
  --env-file envs/my-environment.json \
  --instance-ip 192.168.1.100 \
  --output-dir /tmp/build/my-environment \
  --output-format json 2>/dev/null
```

```json
{
  "environment_name": "my-environment",
  "config_source": "Config file: envs/my-environment.json",
  "target_ip": "192.168.1.100",
  "output_dir": "/tmp/build/my-environment"
}
```

| Field              | Type   | Description                                                       |
| ------------------ | ------ | ----------------------------------------------------------------- |
| `environment_name` | string | Name of the environment whose artifacts were generated            |
| `config_source`    | string | Description of the configuration source (env name or config file) |
| `target_ip`        | string | IP address used in artifact generation                            |
| `output_dir`       | string | Path to the directory containing generated artifacts              |

## Use Cases

### 1. Preview Before Provisioning

```bash
# Create environment
cargo run -- create environment -f envs/staging.json

# Preview what will be deployed
cargo run -- render --env-name staging --instance-ip 203.0.113.10 --output-dir ./preview-staging

# Review artifacts in preview-staging/
# If satisfied, provision (writes to build/staging/)
cargo run -- provision staging
```

**Benefit**: Verify configuration correctness before committing to infrastructure costs.

### 2. Manual Deployment Workflow

```bash
# Generate artifacts without creating environment
cargo run -- render \
  --env-file envs/production.json \
  --instance-ip 203.0.113.50 \
  --output-dir /tmp/manual-deploy

# Manually deploy using generated artifacts
cd /tmp/manual-deploy/tofu
tofu init
tofu apply

cd ../ansible
ansible-playbook -i inventory.yml deploy.yml
```

**Benefit**: Full control over deployment process using standard tools.

### 3. Configuration Validation

```bash
# Generate artifacts to validate configuration
cargo run -- render \
  --env-file envs/test-config.json \
  --instance-ip 192.168.1.1 \
  --output-dir /tmp/validate-config

# Check for syntax errors in generated files
yamllint /tmp/validate-config/docker-compose/docker-compose.yml
tofu validate -chdir=/tmp/validate-config/tofu/
```

**Benefit**: Catch configuration errors early.

### 4. Artifact Comparison

```bash
# Render with SQLite
cargo run -- render --env-file envs/sqlite.json --instance-ip 10.0.0.1 --output-dir /tmp/sqlite-artifacts

# Render with MySQL
cargo run -- render --env-file envs/mysql.json --instance-ip 10.0.0.1 --output-dir /tmp/mysql-artifacts

# Compare configurations
diff -r /tmp/sqlite-artifacts/ /tmp/mysql-artifacts/
```

**Benefit**: Understand configuration differences between setups.

## Comparison: Render vs Provision

| Aspect              | render                                 | provision                     |
| ------------------- | -------------------------------------- | ----------------------------- |
| **Purpose**         | Generate artifacts only                | Deploy infrastructure         |
| **Infrastructure**  | None created                           | Creates VMs/servers           |
| **State Change**    | No change                              | Created → Provisioned         |
| **IP Address**      | User-provided (any IP)                 | From actual infrastructure    |
| **Output Location** | User-specified directory               | build/{env}/ directory        |
| **Cost**            | Free                                   | Provider charges apply        |
| **Time**            | Seconds                                | Minutes (depends on provider) |
| **Use Case**        | Preview, validation, manual deployment | Actual deployment             |

**Key Principle**: Render generates **identical** artifacts to provision (except IP addresses).

## Next Steps

### After Rendering with `--env-name`

If you used `--env-name` mode, you can continue with normal workflow:

```bash
# Review artifacts
ls -la ./preview-my-env/

# If satisfied, provision infrastructure (writes to build/my-env/)
torrust-tracker-deployer provision my-env

# Or continue manual deployment from preview directory
```

### After Rendering with `--env-file`

If you used `--env-file` mode, artifacts are ready for manual deployment:

```bash
# Deploy infrastructure manually
cd <output-dir>/tofu/
tofu init && tofu apply

# Configure with Ansible
cd ../ansible/
ansible-playbook -i inventory.yml playbooks/configure.yml
```

## Troubleshooting

### Environment not found (`--env-name` mode)

**Problem**: Cannot find environment with specified name

**Solution**: Verify environment exists and is in Created state

```bash
# List environments
torrust-tracker-deployer list

# Check environment state
torrust-tracker-deployer show <env-name>

# Should show: State: Created
```

### Configuration file not found (`--env-file` mode)

**Problem**: Cannot read configuration file

**Solution**: Check file path

```bash
# Use absolute path
torrust-tracker-deployer render \
  --env-file /absolute/path/to/config.json \
  --instance-ip 192.168.1.100 \
  --output-dir /tmp/artifacts

# Or relative path from working directory
torrust-tracker-deployer render \
  --env-file ./envs/my-config.json \
  --instance-ip 192.168.1.100 \
  --output-dir ./artifacts
```

### Invalid IP address

**Problem**: IP address validation fails

**Solution**: Use valid IPv4 or IPv6 format

```bash
# Valid IPv4
torrust-tracker-deployer render --env-name test --instance-ip 192.168.1.100 --output-dir ./test-artifacts

# Valid IPv6
torrust-tracker-deployer render --env-name test --instance-ip 2001:db8::1 --output-dir ./test-artifacts-v6

# Invalid (will fail)
torrust-tracker-deployer render --env-name test --instance-ip invalid-ip --output-dir ./test
```

### Environment already provisioned

**Problem**: Environment is in Provisioned state (not Created)

**Behavior**: Command fails with an error explaining the state constraint

```text
❌ Environment 'my-env' is already in 'Provisioned' state.
   The 'render' command only works for environments in 'Created' state.
```

**Solution**:

- For provisioned environments, artifacts were generated during provision and are in `build/my-env/`
- To preview with different configuration, use `--env-file` mode instead:

  ```bash
  torrust-tracker-deployer render --env-file envs/new-config.json --instance-ip <ip> --output-dir ./preview
  ```

### Output directory already exists

**Problem**: The specified output directory already exists

**Behavior**: Command fails to prevent accidental overwrites

```text
❌ Output directory already exists: ./preview-artifacts
```

**Solution**: Choose one:

```bash
# Option 1: Use different directory
torrust-tracker-deployer render ... --output-dir ./preview-artifacts-2

# Option 2: Overwrite with --force (use with caution)
torrust-tracker-deployer render ... --output-dir ./preview-artifacts --force

# Option 3: Remove existing directory
rm -rf ./preview-artifacts
torrust-tracker-deployer render ... --output-dir ./preview-artifacts
```

## Related Commands

- [`create`](create.md) - Create environment (prerequisite for `--env-name` mode)
- [`provision`](provision.md) - Provision infrastructure (uses same artifact generation)
- [`validate`](validate.md) - Validate configuration file (useful before rendering)
- [`show`](show.md) - Show environment state

## See Also

- [Manual E2E Testing: Render Verification](../../e2e-testing/manual/render-verification.md) - Step-by-step testing guide
- [Template System Architecture](../../contributing/templates/template-system-architecture.md) - How templates are rendered
- [Configuration Schema](../../../schemas/environment-config.json) - Configuration file format
