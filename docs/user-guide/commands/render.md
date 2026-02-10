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
torrust-tracker-deployer render --env-name <NAME> --instance-ip <IP>

# From configuration file (no environment creation)
torrust-tracker-deployer render --env-file <PATH> --instance-ip <IP>
```

## Arguments

### Input Mode (choose one)

- `--env-name <NAME>` - Name of existing environment in Created state
- `--env-file <PATH>` - Path to environment configuration file

**Note**: These options are mutually exclusive - use one or the other.

### Required Parameter

- `--instance-ip <IP>` (required) - Target instance IP address for deployment

The IP address is required because:

- In Created state, infrastructure doesn't exist yet (no real IP)
- With config file, no infrastructure is ever created
- IP is needed for Ansible inventory generation

## Prerequisites

### For `--env-name` Mode

1. **Environment created** - Environment must exist in "Created" state
2. **Configuration valid** - Environment configuration must be valid

### For `--env-file` Mode

1. **Configuration file** - Valid environment configuration JSON file
2. **No environment needed** - Does not require existing environment

### Common Requirements

- **Templates available** - Template files in `templates/` directory
- **Write permissions** - Ability to write to `build/` directory

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
5. **Writes artifacts** - Saves generated files to `build/<env-name>/`

## Examples

### Preview before provisioning

```bash
# Create environment
torrust-tracker-deployer create environment -f envs/my-config.json

# Preview artifacts with test IP
torrust-tracker-deployer render --env-name my-env --instance-ip 192.168.1.100

# Review generated artifacts
ls -la build/my-env/

# If satisfied, provision for real
torrust-tracker-deployer provision my-env
```

### Generate from config file without environment

```bash
# Generate artifacts directly from config
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 10.0.0.5

# Artifacts in build/production/ (no environment created in data/)
```

### Multiple target IPs for different environments

```bash
# Development environment
torrust-tracker-deployer render \
  --env-name dev \
  --instance-ip 192.168.1.10

# Staging environment
torrust-tracker-deployer render \
  --env-name staging \
  --instance-ip 10.0.1.20

# Production environment
torrust-tracker-deployer render \
  --env-name prod \
  --instance-ip 203.0.113.50
```

### Inspect specific artifacts

```bash
# Render artifacts
torrust-tracker-deployer render --env-name my-env --instance-ip 192.168.1.100

# Check OpenTofu configuration
cat build/my-env/tofu/main.tf

# Check Ansible inventory (should show 192.168.1.100)
cat build/my-env/ansible/inventory.yml

# Check Docker Compose services
cat build/my-env/docker-compose/docker-compose.yml

# Check tracker configuration
cat build/my-env/tracker/tracker.toml
```

## Output

The render command generates artifacts in `build/<env-name>/`:

### Directory Structure

```text
build/<env-name>/
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

## Use Cases

### 1. Preview Before Provisioning

```bash
# Create environment
cargo run -- create environment -f envs/staging.json

# Preview what will be deployed
cargo run -- render --env-name staging --instance-ip 203.0.113.10

# Review artifacts in build/staging/
# If satisfied, provision
cargo run -- provision staging
```

**Benefit**: Verify configuration correctness before committing to infrastructure costs.

### 2. Manual Deployment Workflow

```bash
# Generate artifacts without creating environment
cargo run -- render \
  --env-file envs/production.json \
  --instance-ip 203.0.113.50

# Manually deploy using generated artifacts
cd build/production/tofu
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
  --instance-ip 192.168.1.1

# Check for syntax errors in generated files
yamllint build/test-config/docker-compose/docker-compose.yml
tofu validate -chdir=build/test-config/tofu/
```

**Benefit**: Catch configuration errors early.

### 4. Artifact Comparison

```bash
# Render with SQLite
cargo run -- render --env-file envs/sqlite.json --instance-ip 10.0.0.1

# Render with MySQL
cargo run -- render --env-file envs/mysql.json --instance-ip 10.0.0.1

# Compare configurations
diff -r build/sqlite/ build/mysql/
```

**Benefit**: Understand configuration differences between setups.

## Comparison: Render vs Provision

| Aspect             | render                                 | provision                     |
| ------------------ | -------------------------------------- | ----------------------------- |
| **Purpose**        | Generate artifacts only                | Deploy infrastructure         |
| **Infrastructure** | None created                           | Creates VMs/servers           |
| **State Change**   | No change                              | Created → Provisioned         |
| **IP Address**     | User-provided (any IP)                 | From actual infrastructure    |
| **Cost**           | Free                                   | Provider charges apply        |
| **Time**           | Seconds                                | Minutes (depends on provider) |
| **Use Case**       | Preview, validation, manual deployment | Actual deployment             |

**Key Principle**: Render generates **identical** artifacts to provision (except IP addresses).

## Next Steps

### After Rendering with `--env-name`

If you used `--env-name` mode, you can continue with normal workflow:

```bash
# Review artifacts
ls -la build/my-env/

# If satisfied, provision infrastructure
torrust-tracker-deployer provision my-env

# Or continue manual deployment
```

### After Rendering with `--env-file`

If you used `--env-file` mode, artifacts are ready for manual deployment:

```bash
# Deploy infrastructure manually
cd build/<env-name>/tofu/
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
  --instance-ip 192.168.1.100

# Or relative path from working directory
torrust-tracker-deployer render \
  --env-file ./envs/my-config.json \
  --instance-ip 192.168.1.100
```

### Invalid IP address

**Problem**: IP address validation fails

**Solution**: Use valid IPv4 or IPv6 format

```bash
# Valid IPv4
torrust-tracker-deployer render --env-name test --instance-ip 192.168.1.100

# Valid IPv6
torrust-tracker-deployer render --env-name test --instance-ip 2001:db8::1

# Invalid (will fail)
torrust-tracker-deployer render --env-name test --instance-ip invalid-ip
```

### Environment already provisioned

**Problem**: Environment is in Provisioned state (not Created)

**Behavior**: Command provides informational message about existing artifacts

```text
ℹ️  Environment 'my-env' is already provisioned.
   Artifacts are available at: build/my-env/
   IP Address: 10.140.190.42
```

**Solution**: This is not an error - artifacts already exist from provisioning.

## Related Commands

- [`create`](create.md) - Create environment (prerequisite for `--env-name` mode)
- [`provision`](provision.md) - Provision infrastructure (uses same artifact generation)
- [`validate`](validate.md) - Validate configuration file (useful before rendering)
- [`show`](show.md) - Show environment state

## See Also

- [Manual E2E Testing: Render Verification](../../e2e-testing/manual/render-verification.md) - Step-by-step testing guide
- [Template System Architecture](../../contributing/templates/template-system-architecture.md) - How templates are rendered
- [Configuration Schema](../../../schemas/environment-config.json) - Configuration file format
