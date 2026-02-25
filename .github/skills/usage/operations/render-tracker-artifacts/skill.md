---
name: render-tracker-artifacts
description: Render Torrust Tracker deployment artifacts without provisioning infrastructure. Use for previewing templates, manual deployment, validating configurations, and comparing deployment options. Triggers on "render", "generate artifacts", "preview deployment", "manual deployment", or "render artifacts".
metadata:
  author: torrust
  version: "1.0"
---

# Render Tracker Artifacts

This skill helps you generate deployment artifacts for the Torrust Tracker without provisioning infrastructure.

## When to Use This Skill

- **Preview deployment configuration** — Inspect what will be deployed before provisioning
- **Generate artifacts for manual deployment** — Create deployment files to use with external tools
- **Deploy to existing infrastructure** — Use generated artifacts with pre-existing servers
- **Compare configurations** — Generate artifacts for multiple setups to compare differences
- **Validate configurations** — Verify config files produce valid artifacts
- **Troubleshoot** — Inspect generated files to diagnose deployment issues

## How Render Works

```text
Configuration → Template Rendering → Artifacts (No Infrastructure)
```

**Generated Artifacts:**

- **OpenTofu** — Infrastructure as code definitions
- **Ansible** — Playbooks and inventory files
- **Docker Compose** — Service orchestration configuration
- **Tracker** — Tracker configuration (tracker.toml)
- **Prometheus** — Monitoring configuration
- **Grafana** — Dashboard provisioning
- **Caddy** — Reverse proxy configuration (if HTTPS enabled)
- **Backup** — Backup scripts (if backup enabled)

## Prerequisites

- No external tools required (render is a read-only operation)
- **Input**: Existing environment (in `data/{env-name}/`) OR a config file (`envs/*.json`)
- **Output directory must not exist** (or use `--force` to overwrite)
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

### Preview Before Provisioning

```bash
# Create environment
torrust-tracker-deployer create environment -f envs/my-config.json

# Preview artifacts with test IP
torrust-tracker-deployer render \
  --env-name my-env \
  --instance-ip 192.168.1.100 \
  --output-dir ./preview-my-env

# Inspect generated artifacts
cat preview-my-env/ansible/inventory.ini
cat preview-my-env/docker-compose/docker-compose.yml
cat preview-my-env/tracker/tracker.toml

# If satisfied, proceed with real provisioning
torrust-tracker-deployer provision my-env
```

### Generate from Config File (No Environment)

```bash
# Generate artifacts directly from config file
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 10.0.0.5 \
  --output-dir /tmp/production-artifacts

# Inspect and use artifacts
cat /tmp/production-artifacts/docker-compose/docker-compose.yml
scp -r /tmp/production-artifacts/ user@target-host:/opt/deployment/
```

### Compare Multiple Configurations

```bash
# Render with different IPs
torrust-tracker-deployer render \
  --env-name my-env --instance-ip 192.168.1.10 --output-dir ./preview-ip-10

torrust-tracker-deployer render \
  --env-name my-env --instance-ip 10.0.1.20 --output-dir ./preview-ip-20

# Compare artifacts
diff -r preview-ip-10/ preview-ip-20/
```

### Manual Deployment

```bash
# Generate artifacts for your target server
torrust-tracker-deployer render \
  --env-file envs/production.json \
  --instance-ip 203.0.113.50 \
  --output-dir ./manual-deploy

# Copy to target server
scp -r ./manual-deploy/ admin@203.0.113.50:/tmp/deploy/

# Execute on target
ssh admin@203.0.113.50
cd /tmp/deploy/
ansible-playbook -i ansible/inventory.ini ansible/playbooks/*.yml
# Or: docker compose -f docker-compose/docker-compose.yml up -d
```

## Output Directory Structure

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

## Tips

1. **Version your renders** — Name output directories with timestamps or versions
2. **Document instance IP** — Record which IP was used for artifact generation
3. **Verify before deploy** — Always inspect critical files (inventory, docker-compose, tracker.toml)
4. **Test in staging** — Render for staging first, test, then render for production
5. **Use `--force`** — Faster than manually deleting output directories
6. **Render is fast** (~1-2 seconds) — Use it liberally for verification

## Error Handling

| Error | Solution |
| --- | --- |
| Output directory already exists | Use `--force` flag or delete the directory |
| Environment not found | Create environment first or use `--env-file` |
| Invalid IP address | Provide valid IPv4 or IPv6 address |
| Configuration file not found | Check file path with `ls envs/` |

## Related Documentation

- **Render Command**: `docs/user-guide/commands/render.md`
- **Create Command**: `docs/user-guide/commands/create.md`
- **Configuration Schema**: `schemas/environment-config.json`
- **Quick Start Guide**: `docs/user-guide/quick-start/README.md`

## See Also

- For **verifying template changes** (contributor workflow): see the `verify-template-changes` skill in `dev/infrastructure/`
