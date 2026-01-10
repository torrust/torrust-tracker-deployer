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

## Configuration Validation

### Recommended: Generate a Template First

The safest way to create a valid configuration is to generate a template:

```bash
# Generate a Hetzner configuration template
docker run --rm \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create template --provider hetzner envs/my-hetzner-env.json
```

Then edit the generated file to replace placeholder values with your actual configuration.

### JSON Schema Validation

A JSON schema is available at `schemas/environment-config.json` for validating configuration files before deployment:

**IDE Integration (VS Code)**:

The repository includes VS Code settings (`.vscode/settings.json`) that automatically validate files in the `envs/` directory against the schema. Open any `envs/*.json` file and the editor will highlight validation errors.

**AI Agent Integration**:

If using an AI coding agent, point it to the JSON schema at `schemas/environment-config.json`. The schema contains:

- All required and optional fields
- Field types and constraints
- Enum values for provider types
- Documentation for each field

**Command Line Validation**:

```bash
# Using ajv-cli (npm install -g ajv-cli)
ajv validate -s schemas/environment-config.json -d envs/your-config.json

# Using check-jsonschema (pip install check-jsonschema)
check-jsonschema --schemafile schemas/environment-config.json envs/your-config.json
```

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
  torrust/tracker-deployer:latest \
  provision my-hetzner-env
```

> **Note**: The Hetzner API token must be included in your environment configuration JSON file (`envs/my-hetzner-env.json`). See the [Configuration Validation](#configuration-validation) section for how to create valid configuration files.

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

## Build Arguments

The following build arguments can be used when building the image:

| Argument  | Description                                | Example                    |
| --------- | ------------------------------------------ | -------------------------- |
| `USER_ID` | User ID for file ownership (default: 1000) | `--build-arg USER_ID=1001` |

## Security: Treating Configuration as Secrets

All configuration files and generated directories should be treated as secrets:

- **`envs/`** - Contains API tokens and credentials in JSON configuration files
- **`data/`** - Contains persisted environment state with sensitive information
- **`build/`** - Contains generated configuration files

**Recommendations**:

1. Set restrictive permissions on these directories:

   ```bash
   chmod 700 ./envs ./data ./build
   chmod 600 ./envs/*.json
   ```

2. Never commit these directories to version control (they are gitignored by default)

3. Use volume mounts carefully - avoid exposing these directories unnecessarily

This follows the same security model as other infrastructure tools like Prometheus and Ansible, where configuration files are considered sensitive.

### AI Coding Agents Warning

> ⚠️ **If you use cloud-based AI coding agents** (GitHub Copilot, Cursor, etc.), any secret the agent can see is potentially transmitted to the AI provider.

This includes configuration files, terminal output, and environment variables. Consider:

- Using local AI models for sensitive work
- Excluding `envs/`, `data/`, `build/` from AI context (`.copilotignore`)
- Not opening secret-containing directories in AI-enabled editors

See the [ADR on Configuration as Secrets](../../docs/decisions/configuration-directories-as-secrets.md#security-warning-ai-coding-agents) for details.

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

### Environment Not Found After Container Restart

The container uses `./data` relative to `/var/lib/torrust/deployer` inside the container. Make sure you're mounting to the correct paths:

```bash
# Correct mount paths
-v $(pwd)/data:/var/lib/torrust/deployer/data
-v $(pwd)/build:/var/lib/torrust/deployer/build
```

### "No environments found" Error

If you created an environment but `list` shows nothing:

1. Verify the data directory is mounted correctly
2. Check the data directory on the host: `ls -la ./data/`
3. Ensure you're using the same volume mounts consistently

### LXD Provider Not Working

**This is expected behavior.** The Docker image only supports cloud providers (Hetzner). LXD requires system-level virtualization access that cannot be provided inside a container.

**Solution**: For LXD deployments, install the deployer directly on your host:

```bash
cargo run --bin dependency-installer install
cargo build --release
```

### Hetzner API Token Issues

If provisioning fails with authentication errors:

1. Verify your API token is valid in the Hetzner Console
2. Ensure the token is correctly set in your environment configuration JSON file:

   ```json
   {
     "provider": {
       "provider": "hetzner",
       "api_token": "your-actual-token-here",
       ...
     }
   }
   ```

3. Validate the configuration before running:
   - Use VS Code with the JSON schema validation
   - Or use `create template --provider hetzner` to generate a valid template

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
