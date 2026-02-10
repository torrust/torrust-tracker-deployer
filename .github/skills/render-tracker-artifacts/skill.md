---
name: render-tracker-artifacts
description: Render Torrust Tracker deployment artifacts without provisioning infrastructure. Use for previewing templates, manual deployment, validating configurations, and verifying template changes. Enables fast create→render workflow for contributors to verify template modifications without full deployment. Triggers on "render", "generate artifacts", "preview deployment", "verify templates", "manual deployment", or "check template changes".
metadata:
  author: torrust
  version: "1.0"
---

# Render Tracker Artifacts

This skill helps you generate deployment artifacts for the Torrust Tracker without provisioning infrastructure.

## When to Use This Skill

Use this skill when you need to:

### For Manual Deployment

- **Generate artifacts for manual deployment** - Create deployment files to use with external tools
- **Preview deployment configuration** - Inspect what will be deployed before provisioning
- **Deploy to existing infrastructure** - Use generated artifacts with pre-existing servers
- **Custom deployment workflows** - Integrate generated artifacts into your own automation

### For Contributors and Template Development

- **Verify template changes quickly** - Test template modifications without full deployment
- **Fast development iteration** - Use `create → render` instead of slow `create → provision → configure`
- **Template validation** - Ensure templates render correctly with different configurations
- **Compare configurations** - Generate artifacts for multiple setups to compare differences

### For Validation and Inspection

- **Configuration validation** - Verify config files produce valid artifacts
- **Troubleshooting** - Inspect generated files to diagnose deployment issues
- **Documentation** - Generate example artifacts for documentation purposes
- **Security review** - Review generated configurations before deployment

## How Render Works

The render command generates **all deployment artifacts** without creating infrastructure:

```text
Configuration → Template Rendering → Artifacts (No Infrastructure)
```

**Generated Artifacts:**

- **OpenTofu** - Infrastructure as code definitions
- **Ansible** - Playbooks and inventory files
- **Docker Compose** - Service orchestration configuration
- **Tracker** - Tracker configuration (tracker.toml)
- **Prometheus** - Monitoring configuration
- **Grafana** - Dashboard provisioning
- **Caddy** - Reverse proxy configuration (if HTTPS enabled)
- **Backup** - Backup scripts (if backup enabled)

## Prerequisites

### Required Tools

- None (render is a read-only operation requiring no external tools)

### Input Requirements

**Choose one input mode:**

1. **Existing environment** (Created state):
   - Environment must exist in `data/{env-name}/`
   - Configuration must be valid

2. **Configuration file** (no environment needed):
   - Valid JSON configuration file
   - See `schemas/environment-config.json` for schema

### Output Requirements

- **Output directory must not exist** (or use `--force` to overwrite)
- **Write permissions** to output directory location
- **Instance IP address** required (IPv4 or IPv6)

## Command Syntax

### From Existing Environment

```bash
torrust-tracker-deployer render \
  --env-name <NAME> \
  --instance-ip <IP> \
  --output-dir <PATH>
```

### From Configuration File

```bash
torrust-tracker-deployer render \
  --env-file <PATH> \
  --instance-ip <IP> \
  --output-dir <PATH>
```

### With Force Overwrite

```bash
torrust-tracker-deployer render \
  --env-name <NAME> \
  --instance-ip <IP> \
  --output-dir <PATH> \
  --force
```

## Common Workflows

### Workflow 1: Preview Before Provisioning

**Scenario**: You want to see what artifacts will be generated before committing to infrastructure provisioning.

```bash
# Step 1: Create environment (no infrastructure created)
torrust-tracker-deployer create environment -f envs/my-config.json

# Step 2: Preview artifacts with test IP
torrust-tracker-deployer render \
  --env-name my-env \
  --instance-ip 192.168.1.100 \
  --output-dir ./preview-my-env

# Step 3: Inspect generated artifacts
ls -la preview-my-env/
cat preview-my-env/ansible/inventory.ini
cat preview-my-env/docker-compose/docker-compose.yml
cat preview-my-env/tracker/tracker.toml

# Step 4: If satisfied, provision for real
torrust-tracker-deployer provision my-env
```

### Workflow 2: Verify Template Changes (Contributors)

**Scenario**: You've modified templates and want to verify they render correctly without going through the slow deployment process.

```bash
# Step 1: Create test environment
torrust-tracker-deployer create environment -f envs/lxd-local-example.json

# Step 2: Render artifacts to verify template changes
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./template-test

# Step 3: Check the specific template you modified
cat template-test/ansible/playbooks/your-playbook.yml
cat template-test/docker-compose/docker-compose.yml

# Step 4: Make template changes and re-render
# Edit templates in templates/ directory
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./template-test \
  --force

# Step 5: Verify changes
diff -u /tmp/old-render/ansible/playbooks/your-playbook.yml \
        template-test/ansible/playbooks/your-playbook.yml
```

**Why This Is Fast:**

- ✅ **No infrastructure provisioning** (skips LXD VM creation, cloud API calls)
- ✅ **No remote operations** (skips SSH, Ansible execution)
- ✅ **Instant feedback** (only template rendering, ~1-2 seconds)
- ✅ **Repeatable** (use `--force` to regenerate instantly)

**Comparison:**

```text
Full Deployment: create → provision (~5 min) → configure (~3 min) → release (~2 min)
Template Verify: create → render (~2 sec) → inspect → modify → render (~2 sec)

Time saved: ~10 minutes per iteration
```

### Workflow 3: Generate from Config File (No Environment)

**Scenario**: You want to generate artifacts directly from a configuration file without creating a persistent environment.

```bash
# Generate artifacts directly from config file
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 10.0.0.5 \
  --output-dir /tmp/production-artifacts

# Inspect artifacts (no environment created in data/)
ls -la /tmp/production-artifacts/
cat /tmp/production-artifacts/docker-compose/docker-compose.yml

# Use artifacts with external deployment tools
scp -r /tmp/production-artifacts/ user@target-host:/opt/deployment/
```

### Workflow 4: Compare Multiple Configurations

**Scenario**: You want to compare artifacts generated with different IPs or configurations.

```bash
# Render with different IPs
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
diff -u preview-ip-10/ansible/inventory.ini \
        preview-ip-20/ansible/inventory.ini
```

### Workflow 5: Template Development with E2E Verification

**Scenario**: You're developing templates and want to verify they work end-to-end with the deployment workflow.

```bash
# Step 1: Create environment
torrust-tracker-deployer create environment -f envs/lxd-local-example.json

# Step 2: Quick render to verify syntax
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./quick-check

# Step 3: Inspect for obvious errors
cat quick-check/ansible/playbooks/your-new-playbook.yml
yamllint quick-check/ansible/playbooks/your-new-playbook.yml

# Step 4: If templates look good, test with real deployment
torrust-tracker-deployer provision lxd-local-example
torrust-tracker-deployer configure lxd-local-example
# ... continue full workflow
```

### Workflow 6: Manual Deployment with Rendered Artifacts

**Scenario**: You want to deploy manually using the generated artifacts.

```bash
# Step 1: Generate artifacts for your target
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 203.0.113.50 \
  --output-dir ./manual-deploy

# Step 2: Copy artifacts to target server
scp -r ./manual-deploy/ admin@203.0.113.50:/tmp/deploy/

# Step 3: Execute deployment manually on target
ssh admin@203.0.113.50
cd /tmp/deploy/

# Run Ansible playbooks manually
ansible-playbook -i ansible/inventory.ini ansible/playbooks/*.yml

# Or manually execute Docker Compose
docker compose -f docker-compose/docker-compose.yml up -d
```

## Understanding Output Directory Requirement

**Why `--output-dir` is required:**

1. **Prevents conflicts** - Avoids overwriting `build/{env}/` used by provision command
2. **Enables preview** - Generate artifacts without affecting deployment state
3. **Allows multiple renders** - Create artifacts with different IPs/configs simultaneously
4. **Clear separation** - Distinguishes preview (render) from deployment (provision)

**Output directory structure:**

```text
<output-dir>/
├── opentofu/           # Infrastructure code
├── ansible/            # Configuration playbooks
│   ├── inventory.ini   # Instance IP configured here
│   ├── ansible.cfg
│   └── playbooks/
├── docker-compose/     # Service definitions
│   ├── docker-compose.yml
│   └── .env
├── tracker/            # Tracker configuration
│   └── tracker.toml
├── prometheus/         # Monitoring
│   └── prometheus.yml
├── grafana/            # Dashboards
│   └── provisioning/
├── caddy/              # Reverse proxy (if HTTPS)
│   └── Caddyfile
└── backup/             # Backup scripts (if enabled)
    └── backup.sh
```

## Tips and Best Practices

### For Contributors

1. **Use render for template iteration** - Much faster than full deployment workflow
2. **Test with real config** - Use actual environment configs from `envs/` directory
3. **Verify all artifact types** - Check not just the template you modified but all dependent files
4. **Use force flag** - `--force` for quick iteration without manual deletion
5. **Compare before/after** - Keep old renders to diff against new changes

### For Manual Deployment

1. **Version your renders** - Name output directories with timestamps or versions
2. **Document instance IP** - Record which IP was used for artifact generation
3. **Verify before deploy** - Always inspect critical files (inventory, docker-compose, tracker.toml)
4. **Test in staging** - Render for staging environment first, test, then render for production

### For Validation

1. **Render with multiple IPs** - Ensure artifacts work with different network configurations
2. **Check file permissions** - Verify generated files have correct permissions
3. **Validate syntax** - Use linters on generated YAML/TOML files
4. **Inspect secrets** - Ensure API tokens and passwords are correctly populated

### Performance Tips

1. **Render is fast** (~1-2 seconds) - Use it liberally for verification
2. **Use `--force`** - Faster than manually deleting output directories
3. **Keep renders small** - Delete old preview directories when done
4. **Parallel renders** - Can render multiple configs simultaneously to different output dirs

## Error Handling

### Common Errors and Solutions

#### Error: Output directory already exists

```text
Error: Output directory already exists: ./preview
```

**Solution**: Use `--force` flag or delete the directory:

```bash
# Option 1: Use force
torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1 --output-dir ./preview --force

# Option 2: Delete manually
rm -rf ./preview
torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1 --output-dir ./preview
```

#### Error: Environment not found

```text
Error: Environment 'my-env' not found in Created state
```

**Solution**: Create environment first or use `--env-file`:

```bash
# Option 1: Create environment
torrust-tracker-deployer create environment -f envs/my-config.json

# Option 2: Use config file directly
torrust-tracker-deployer render --env-file envs/my-config.json --instance-ip 10.0.0.1 --output-dir ./preview
```

#### Error: Invalid IP address

```text
Error: Invalid IP address: '999.999.999.999'
```

**Solution**: Provide valid IPv4 or IPv6 address:

```bash
# Valid IPv4
torrust-tracker-deployer render --env-name my-env --instance-ip 192.168.1.100 --output-dir ./preview

# Valid IPv6
torrust-tracker-deployer render --env-name my-env --instance-ip 2001:db8::1 --output-dir ./preview
```

#### Error: Configuration file not found

```text
Error: Configuration file not found: envs/missing.json
```

**Solution**: Check file path and existence:

```bash
# List available configs
ls -la envs/

# Use correct path
torrust-tracker-deployer render --env-file envs/lxd-local-example.json --instance-ip 10.0.0.1 --output-dir ./preview
```

## Related Documentation

- [Render Command User Guide](../../docs/user-guide/commands/render.md) - Complete command documentation
- [Create Command Guide](../../docs/user-guide/commands/create.md) - Creating environments
- [Template System Architecture](../../docs/contributing/templates/template-system-architecture.md) - Template internals
- [Configuration Schema](../../schemas/environment-config.json) - Environment configuration format
- [Quick Start Guide](../../docs/user-guide/quick-start/README.md) - Getting started tutorials

## See Also

- **run-linters** skill - Lint generated artifacts before using
- **add-new-command** skill - Implement new commands similar to render
- **create-issue** skill - Report template rendering issues
