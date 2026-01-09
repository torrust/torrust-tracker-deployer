# Create Docker Image for the Deployer

**Issue**: [#264](https://github.com/torrust/torrust-tracker-deployer/issues/264)
**Parent Epic**: #1 - Project Roadmap
**Related**:

- Dependency installer documentation: `packages/dependency-installer/README.md`
- Existing Docker configs: `docker/provisioned-instance/`, `docker/ssh-server/`
- User guide: `docs/user-guide/README.md`

## Overview

Create an official Docker image for the Torrust Tracker Deployer that bundles all required dependencies (OpenTofu, Ansible, etc.) so users can run the deployer without installing these tools locally. The image should support mounting user data and build directories for persistence and flexibility.

> ⚠️ **Important Limitation**: The Docker image only supports **cloud providers** (Hetzner). The **LXD provider is not supported** when running the deployer from within a container, because LXD requires system-level access to local virtualization that cannot be provided inside a container. LXD users should run the deployer directly on the host.

## Goals

- [ ] Provide a containerized version of the deployer with all dependencies pre-installed
- [ ] Enable users to run deployments without local dependency installation
- [ ] Support volume mounts for `data/`, `build/`, and `envs/` directories
- [ ] Publish the image to Docker Hub or GitHub Container Registry (GHCR)
- [ ] Document usage with clear examples

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (Docker configuration) + Documentation
**Module Path**: `docker/deployer/`
**Pattern**: Infrastructure as Code (Dockerfile)

### Module Structure Requirements

- [ ] Docker configuration follows project conventions (see existing `docker/` directory)
- [ ] Documentation integrates with user guide (`docs/user-guide/`)
- [ ] CI/CD pipeline for automated image building (GitHub Actions)

### Architectural Constraints

- [ ] Image must include: OpenTofu, Ansible, SSH client
- [ ] Image must include the compiled `torrust-tracker-deployer` binary
- [ ] Volume mounts required for stateful directories (data, build, envs)
- [ ] Image should be based on a minimal, secure base image
- [ ] Multi-stage build to minimize final image size

### Anti-Patterns to Avoid

- ❌ Hardcoding credentials or secrets in the Dockerfile
- ❌ Running as root user (use non-root user for security)
- ❌ Including development dependencies in the final image
- ❌ Baking configuration into the image (use environment variables and mounts)

## Specifications

### Dependencies to Include

Based on `packages/dependency-installer/README.md`:

1. **OpenTofu** - Infrastructure provisioning tool (v1.8.x or later)
2. **Ansible** - Configuration management tool (v2.15.x or later)
3. **SSH Client** - For remote connections to provisioned VMs
4. **Git** - For version control operations

### Docker Image Structure

```text
docker/deployer/
├── Dockerfile          # Multi-stage build for deployer image
├── entry_script_sh     # Container initialization script
└── README.md           # Usage documentation
```

### Dockerfile Design

Following the patterns from [torrust-tracker Containerfile](https://github.com/torrust/torrust-tracker/blob/develop/Containerfile):

```dockerfile
# syntax=docker/dockerfile:latest

# Torrust Tracker Deployer

## Builder Image
FROM docker.io/library/rust:docker/provisioned-instance/Dockerfile AS chef
WORKDIR /tmp
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-chef


## Chef Prepare (analyze project dependencies)
FROM chef AS recipe
WORKDIR /build/src
COPY . /build/src
RUN cargo chef prepare --recipe-path /build/recipe.json


## Cook (build dependencies - cached layer)
FROM chef AS dependencies
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --release --recipe-path /build/recipe.json


## Build Binary
FROM dependencies AS build
WORKDIR /build/src
COPY . /build/src
RUN cargo build --release --bin torrust-tracker-deployer
RUN mkdir -p /app/bin/; cp /build/src/target/release/torrust-tracker-deployer /app/bin/


## Runtime Image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (OpenTofu, Ansible, SSH)
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssh-client \
    curl \
    gnupg \
    python3 \
    python3-pip \
    pipx \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Ansible via pipx
ENV PIPX_HOME=/opt/pipx
ENV PIPX_BIN_DIR=/usr/local/bin
RUN pipx install ansible-core

# Install OpenTofu
RUN curl -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh \
    && chmod +x install-opentofu.sh \
    && ./install-opentofu.sh --install-method deb \
    && rm install-opentofu.sh

# Setup directories and permissions
ARG USER_ID=1000
ENV USER_ID=${USER_ID}
ENV TZ=Etc/UTC

RUN mkdir -p /var/lib/torrust/deployer/data \
             /var/lib/torrust/deployer/build \
             /var/lib/torrust/deployer/envs \
             /var/log/torrust/deployer

# Copy binary from build stage
COPY --from=build /app/bin/torrust-tracker-deployer /usr/bin/torrust-tracker-deployer

# Copy entrypoint script
COPY --chmod=0555 ./share/container/entry_script_sh /usr/local/bin/entry.sh

VOLUME ["/var/lib/torrust/deployer/data", "/var/lib/torrust/deployer/build", "/var/lib/torrust/deployer/envs"]

ENTRYPOINT ["/usr/local/bin/entry.sh"]
CMD ["--help"]
```

### Volume Mount Structure

| Host Path  | Container Path                    | Purpose                              |
| ---------- | --------------------------------- | ------------------------------------ |
| `./data/`  | `/var/lib/torrust/deployer/data`  | Environment state and persistence    |
| `./build/` | `/var/lib/torrust/deployer/build` | Generated configuration files        |
| `./envs/`  | `/var/lib/torrust/deployer/envs`  | User environment configuration files |
| `~/.ssh/`  | `/root/.ssh`                      | SSH keys for remote access           |

### Usage Examples

#### Basic Usage

```bash
# Pull the image
docker pull torrust/tracker-deployer:latest

# Run a command
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/root/.ssh:ro \
  torrust/tracker-deployer:latest \
  create --env-file /var/lib/torrust/deployer/envs/my-env.json
```

### CI/CD Integration

Following the patterns from [torrust-tracker container workflow](https://github.com/torrust/torrust-tracker/blob/develop/.github/workflows/container.yaml):

```yaml
# .github/workflows/container.yaml
name: Container

on:
  push:
    branches:
      - "develop"
      - "main"
      - "releases/**/*"
  pull_request:
    branches:
      - "develop"
      - "main"

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test (Docker)
    runs-on: ubuntu-latest

    steps:
      - id: setup
        name: Setup Toolchain
        uses: docker/setup-buildx-action@v3

      - id: build
        name: Build
        uses: docker/build-push-action@v6
        with:
          file: ./Dockerfile
          push: false
          load: true
          tags: torrust-tracker-deployer:local
          cache-from: type=gha
          cache-to: type=gha

      - id: inspect
        name: Inspect
        run: docker image inspect torrust-tracker-deployer:local

  context:
    name: Context
    needs: test
    runs-on: ubuntu-latest

    outputs:
      continue: ${{ steps.check.outputs.continue }}
      type: ${{ steps.check.outputs.type }}
      version: ${{ steps.check.outputs.version }}

    steps:
      - id: check
        name: Check Context
        run: |
          if [[ "${{ github.repository }}" == "torrust/torrust-tracker-deployer" ]]; then
            if [[ "${{ github.event_name }}" == "push" ]]; then
              if [[ "${{ github.ref }}" == "refs/heads/main" ]]; then
                echo "type=development" >> $GITHUB_OUTPUT
                echo "continue=true" >> $GITHUB_OUTPUT
              elif [[ "${{ github.ref }}" == "refs/heads/develop" ]]; then
                echo "type=development" >> $GITHUB_OUTPUT
                echo "continue=true" >> $GITHUB_OUTPUT
              fi
            fi
          fi

  publish_development:
    name: Publish (Development)
    environment: dockerhub-torrust
    needs: context
    if: needs.context.outputs.continue == 'true' && needs.context.outputs.type == 'development'
    runs-on: ubuntu-latest

    steps:
      - id: meta
        name: Docker Meta
        uses: docker/metadata-action@v5
        with:
          images: |
            "${{ secrets.DOCKER_HUB_USERNAME }}/${{ secrets.DOCKER_HUB_REPOSITORY_NAME }}"
          tags: |
            type=ref,event=branch

      - id: login
        name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKER_HUB_USERNAME }}
          password: ${{ secrets.DOCKER_HUB_ACCESS_TOKEN }}

      - id: setup
        name: Setup Toolchain
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          file: ./Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha
```

### Image Security Considerations

1. **Non-root user**: Container runs as `deployer` user, not root
2. **Minimal base**: Use `debian:bookworm-slim` for minimal attack surface
3. **No secrets baked in**: All sensitive data passed via mounts or environment variables
4. **Regular updates**: CI/CD rebuilds on main branch to include security patches
5. **Vulnerability scanning**: Integrate with existing Trivy workflow (#250)

## Implementation Plan

### Phase 1: Dockerfile and Basic Structure (2-3 hours)

- [ ] Task 1.1: Create `docker/deployer/` directory structure
- [ ] Task 1.2: Create multi-stage Dockerfile with cargo-chef caching
- [ ] Task 1.3: Create `entry_script_sh` entrypoint script
- [ ] Task 1.4: Test local image build with `docker build -f docker/deployer/Dockerfile .`
- [ ] Task 1.5: Verify all dependencies are correctly installed in image

### Phase 2: Testing and Validation (2 hours)

- [ ] Task 2.1: Test volume mounts with sample environment
- [ ] Task 2.2: Run basic deployer commands in container
- [ ] Task 2.3: Verify SSH connectivity from container to test VM
- [ ] Task 2.4: Test with existing E2E test environment

### Phase 3: CI/CD Pipeline (1-2 hours)

- [ ] Task 3.1: Create GitHub Actions workflow for image building (`.github/workflows/container.yaml`)
- [ ] Task 3.2: Configure Docker Hub publishing (following Torrust conventions)
- [ ] Task 3.3: Add image to Trivy security scanning workflow
- [ ] Task 3.4: Test automated builds on push to main/develop

### Phase 4: Documentation (1 hour)

- [ ] Task 4.1: Create `docker/deployer/README.md` with detailed usage
- [ ] Task 4.2: Update main `README.md` with Docker usage section
- [ ] Task 4.3: Update user guide (`docs/user-guide/`) with containerized deployment option
- [ ] Task 4.4: Add troubleshooting section for common Docker issues

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] Docker image builds successfully from Dockerfile
- [ ] Image includes OpenTofu, Ansible, SSH client, and deployer binary
- [ ] Deployer commands work correctly inside container
- [ ] Volume mounts persist data between container runs
- [ ] SSH keys can be mounted and used for remote connections
- [ ] Image runs as non-root user

**CI/CD Requirements**:

- [ ] GitHub Actions workflow builds and pushes image automatically
- [ ] Image is published to Docker Hub on push to main/develop branches
- [ ] Tagged releases publish versioned images

**Documentation Requirements**:

- [ ] `docker/deployer/README.md` documents all usage patterns
- [ ] Main README includes Docker quick start section
- [ ] User guide updated with containerized deployment option

**Security Requirements**:

- [ ] Image passes Trivy vulnerability scan (no HIGH/CRITICAL)
- [ ] No secrets or credentials in Dockerfile or image layers
- [ ] Container runs as non-root user by default

## Related Documentation

- [Dependency Installer Package](../../packages/dependency-installer/README.md) - Lists required dependencies
- [Existing Docker Configurations](../../docker/) - Reference implementations
- [User Guide](../user-guide/README.md) - Where to document usage
- [CI/CD Vulnerability Scanning](252-implement-dynamic-image-detection-for-scanning.md) - Security integration
- [Torrust Tracker Containerfile](https://github.com/torrust/torrust-tracker/blob/develop/Containerfile) - Reference for container patterns
- [Torrust Tracker Container Workflow](https://github.com/torrust/torrust-tracker/blob/develop/.github/workflows/container.yaml) - Reference for CI/CD patterns

## Notes

### Base Image Selection

The choice of `debian:bookworm-slim` balances:

- **Compatibility**: Debian has excellent package support for Ansible and OpenTofu
- **Size**: Slim variant minimizes image size
- **Security**: Long-term support with regular security updates

Alternative considerations:

- `alpine`: Smaller but may have compatibility issues with some Python packages
- `ubuntu`: More familiar but larger than Debian slim

### Provider-Specific Considerations

#### ⚠️ Important Limitation: LXD Provider Not Supported in Container

**The Docker image is designed for cloud providers (Hetzner) only.** LXD provider is not supported when running the deployer from within a container.

**Why LXD doesn't work in a container:**

- LXD requires system-level access to create and manage VMs/containers
- LXD needs access to Linux kernel virtualization features (KVM, etc.)
- Installing LXD inside a Docker container requires privileged mode and nested virtualization, which is complex and has security implications

**Supported configurations:**

| Provider    | Container Support  | Notes                                 |
| ----------- | ------------------ | ------------------------------------- |
| **Hetzner** | ✅ Fully supported | All communication via API and SSH     |
| **LXD**     | ❌ Not supported   | Run deployer directly on host instead |

**For LXD users:** Install the deployer and its dependencies directly on your host machine using the dependency installer:

```bash
cargo run --bin dependency-installer install
```

#### Hetzner Provider

Works perfectly from container since all communication is:

- **Infrastructure provisioning**: OpenTofu calls Hetzner API over HTTPS
- **Configuration**: Ansible connects to provisioned VMs via SSH
- **No local virtualization required**

### Future Enhancements

- Docker Compose file for running deployer as a service
- Kubernetes deployment manifests
- Support for podman as alternative container runtime
