# AI Training Outputs - Rendered Deployment Artifacts

This directory contains the **rendered deployment artifacts** for all AI training example configurations.

## Purpose

These outputs complete the AI training dataset with **input/output pairs**:

- **Input**: Environment configuration JSON files in [`../examples/`](../examples/)
- **Output**: Rendered deployment templates (this directory)

This mapping demonstrates how the deployer transforms high-level configuration into concrete deployment artifacts.

## Structure

Each subdirectory corresponds to an example configuration:

```text
outputs/
├── 01-minimal-lxd/          # Outputs for examples/01-minimal-lxd.json
├── 02-full-stack-lxd/       # Outputs for examples/02-full-stack-lxd.json
├── 03-minimal-hetzner/      # Outputs for examples/03-minimal-hetzner.json
└── ...                       # (15 total)
```

### Artifact Structure

Each output directory contains rendered templates for all deployment services:

```text
01-minimal-lxd/
├── ansible/                 # Ansible playbooks and inventory
│   ├── ansible.cfg
│   ├── inventory.yml
│   ├── variables.yml
│   ├── install-docker.yml
│   └── ...
├── docker-compose/          # Docker Compose configuration
│   ├── docker-compose.yml
│   └── .env
├── grafana/                 # Grafana dashboards and datasources
│   └── provisioning/
├── prometheus/              # Prometheus configuration
│   └── prometheus.yml
├── tofu/                    # OpenTofu infrastructure definitions
│   ├── lxd/                 # (for LXD provider)
│   └── hetzner/             # (for Hetzner provider)
└── tracker/                 # Torrust Tracker configuration
    └── tracker.toml
```

## Benefits

**For AI Agents:**

- **Few-shot learning**: Full input/output examples for configuration generation
- **Pattern recognition**: See how config options map to rendered templates
- **Diff analysis**: Compare outputs to understand configuration impact

**For Humans:**

- **Documentation**: Concrete examples of what gets deployed
- **Debugging**: Reference to verify expected template content
- **Learning**: Understand the deployer's transformation process

## Generation

These outputs are generated automatically using:

```bash
./scripts/generate-ai-training-outputs.sh
```

The script:

1. Reads each example from `docs/ai-training/examples/`
2. Replaces generic SSH paths with project fixture paths
3. Calls `render` command with placeholder IP `203.0.113.1` (RFC 5737 TEST-NET-1)
4. Outputs artifacts to corresponding subdirectory here

**Regeneration**: Run the script to update outputs when:

- Example configurations change
- Templates are modified
- New features are added

## Placeholder Values

All outputs use safe placeholder values:

- **IP Address**: `203.0.113.1` (RFC 5737 documentation range)
- **SSH Keys**: Project fixture keys from `fixtures/testing_rsa*`
- **API Tokens**: Placeholder strings (e.g., `PLACEHOLDER_API_TOKEN_FOR_TESTING`)
- **Passwords**: Example values (e.g., `admin-password`, `tracker_user_password`)

**These are NOT production values** - they're for documentation and training purposes only.

## Size

Total size: ~4.1 MB for all 15 examples (reasonable for git storage).

## Maintenance

When templates change significantly:

1. Run regeneration script: `./scripts/generate-ai-training-outputs.sh`
2. Review diffs to ensure changes are expected
3. Commit updated outputs with template changes

This keeps the dataset synchronized with the current deployer version.
