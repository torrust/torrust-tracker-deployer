# Torrust Tracker Deployer - Docker Image

This directory contains the Docker configuration for building a containerized version of the Torrust Tracker Deployer.

## ⚠️ Important Limitation

**This container only supports CLOUD PROVIDERS (e.g., Hetzner).**

The **LXD provider is NOT supported** when running from a container because:

- LXD manages local virtual machines through system-level APIs
- Requires access to host virtualization features (KVM, QEMU)
- Running LXD inside Docker requires privileged containers with full device access
- This defeats the purpose of containerization and introduces security risks

**For LXD deployments**: Install the deployer directly on the host using the native installation method.

## Quick Start

### Pull the Image

```bash
docker pull torrust/tracker-deployer:latest
```

### Build Locally

```bash
# From repository root
docker build --target release --tag torrust/tracker-deployer:release --file docker/deployer/Dockerfile .
```

### Run a Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  --help
```

## Volume Mounts

The container requires several volume mounts for proper operation:

| Host Path  | Container Path                    | Purpose                              | Required |
| ---------- | --------------------------------- | ------------------------------------ | -------- |
| `./data/`  | `/var/lib/torrust/deployer/data`  | Environment state and persistence    | Yes      |
| `./build/` | `/var/lib/torrust/deployer/build` | Generated configuration files        | Yes      |
| `./envs/`  | `/var/lib/torrust/deployer/envs`  | User environment configuration files | Yes      |
| `~/.ssh/`  | `/home/deployer/.ssh`             | SSH keys for remote access           | Yes      |

## Usage Examples

### Create an Environment

```bash
# First, create your environment config in ./envs/my-hetzner-env.json
# Then run:
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  create environment --env-file /var/lib/torrust/deployer/envs/my-hetzner-env.json
```

### Provision Infrastructure

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  -e HETZNER_API_TOKEN="your-api-token" \
  torrust/tracker-deployer:latest \
  provision my-hetzner-env
```

### List Environments

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  torrust/tracker-deployer:latest \
  list
```

### Show Environment Details

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  torrust/tracker-deployer:latest \
  show my-hetzner-env
```

## Environment Variables

The following environment variables can be passed to the container:

| Variable            | Description                                | Example                    |
| ------------------- | ------------------------------------------ | -------------------------- |
| `HETZNER_API_TOKEN` | Hetzner Cloud API token for provision      | `-e HETZNER_API_TOKEN=...` |
| `USER_ID`           | User ID for file ownership (default: 1000) | `--build-arg USER_ID=1001` |

## Build Targets

The Dockerfile supports multiple build targets:

### Release (Default)

Production-ready image with minimal footprint:

```bash
docker build --target release --tag torrust/tracker-deployer:release --file docker/deployer/Dockerfile .
```

### Debug

Includes additional debugging tools (vim, less, procps):

```bash
docker build --target debug --tag torrust/tracker-deployer:debug --file docker/deployer/Dockerfile .
```

## Included Tools

The container includes the following tools:

- **torrust-tracker-deployer** - The main deployer binary
- **OpenTofu** - Infrastructure as Code tool (Terraform fork)
- **Ansible** - Configuration management and automation
- **SSH Client** - For secure remote connections
- **Git** - Version control

## Security Considerations

1. **Non-root user**: The container runs as the `deployer` user (UID 1000 by default)
2. **Read-only SSH**: SSH keys are mounted read-only (`:ro`)
3. **No privileged mode**: The container does not require privileged access
4. **Minimal base image**: Uses `debian:bookworm-slim` for reduced attack surface

## Troubleshooting

### Permission Issues

If you encounter permission issues with mounted volumes, ensure the host directories exist and have correct permissions:

```bash
mkdir -p ./data ./build ./envs
chmod 755 ./data ./build ./envs
```

### SSH Key Issues

Ensure your SSH keys have the correct permissions:

```bash
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_rsa  # or id_ed25519
```

### Debug Mode

Run the debug image to troubleshoot issues:

```bash
docker run --rm -it \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:debug
```

## File Structure

```text
docker/deployer/
├── Dockerfile       # Multi-stage Dockerfile
├── entry_script_sh  # Container entrypoint script
└── README.md        # This file
```

## Related Documentation

- [User Guide](../../docs/user-guide/README.md)
- [Issue Specification](../../docs/issues/264-create-docker-image-for-deployer.md)
- [Dependency Installer](../../packages/dependency-installer/README.md)
