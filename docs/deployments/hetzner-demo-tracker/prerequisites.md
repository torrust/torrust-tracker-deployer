# Prerequisites

Checklist of everything needed before deploying the demo tracker to Hetzner Cloud.

## Accounts

- [ ] **Hetzner Cloud account** — [Sign up](https://www.hetzner.com/cloud) if you don't have one
- [ ] **Hetzner API token** — Created in [Hetzner Console](https://console.hetzner.cloud/) → Security → API Tokens (Read & Write)
- [ ] **Domain registrar access** — To configure DNS A records for `torrust-tracker-demo.com`

## SSH Keys

An SSH key pair is required for VM access. The deployer uses this to connect to the provisioned server.

```bash
# Generate a dedicated key pair (if you don't have one)
ssh-keygen -t ed25519 -C "torrust-tracker-deployer" -f ~/.ssh/torrust_tracker_deployer_ed25519
```

- [ ] SSH private key available (e.g., `~/.ssh/torrust_tracker_deployer_ed25519`)
- [ ] SSH public key available (e.g., `~/.ssh/torrust_tracker_deployer_ed25519.pub`)
- [ ] Key permissions are correct (`chmod 600` on private key)

## Tools

The deployer can run via Docker (simpler) or natively (more flexibility). We'll document which method we use.

### Option A: Docker (recommended for cloud providers)

- [ ] Docker installed and running

```bash
docker --version
```

- [ ] Deployer Docker image pulled

```bash
docker pull torrust/tracker-deployer:latest
```

### Option B: Native Installation

- [ ] Rust toolchain installed
- [ ] OpenTofu installed
- [ ] Ansible installed
- [ ] cargo-machete installed

```bash
# Verify all dependencies at once
cargo run --bin dependency-installer check
```

## Working Directories

```bash
mkdir -p data build envs
chmod 700 envs  # Contains sensitive configuration
```

- [ ] `data/` directory exists
- [ ] `build/` directory exists
- [ ] `envs/` directory exists with restricted permissions

## DNS (can be done after provisioning)

DNS records will be configured after we know the server IP address. We need:

- [ ] A records for `torrust-tracker-demo.com` subdomains pointing to the server IP

## Related Documentation

- [Hetzner Cloud Provider guide](../../user-guide/providers/hetzner.md)
- [Quick Start: Docker Deployment](../../user-guide/quick-start/docker.md)
- [Quick Start: Native Installation](../../user-guide/quick-start/native.md)
