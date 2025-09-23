# Docker Architecture for Deployment Phases

This document outlines the Docker architecture for representing different deployment phases in the Torrust infrastructure.

## Current Implementation

### Provisioned Instance (`docker/provisioned-instance/`)

**Purpose**: Represents the state after VM provisioning but before configuration.

**Contents**:

- Ubuntu 24.04 LTS base
- SSH server (via supervisor)
- `torrust` user with sudo access
- No application dependencies installed
- Ready for Ansible configuration

**Usage**: E2E configuration testing - simulates a freshly provisioned VM.

## Future Expansion Options

### Option 1: Multiple Dockerfiles (Recommended)

Create separate directories for each deployment phase:

```text
docker/
├── provisioned-instance/          # Current - post-provision
│   ├── Dockerfile
│   ├── supervisord.conf
│   ├── entrypoint.sh
│   └── README.md
├── configured-instance/           # Future - post-configure
│   ├── Dockerfile
│   ├── docker-compose.yml         # Example: Docker services
│   └── README.md
└── released-instance/             # Future - post-release
    ├── Dockerfile
    ├── app-configs/               # Application configurations
    └── README.md
```

**Benefits**:

- **Clear Separation**: Each phase has its own directory and concerns
- **Independent Evolution**: Each Dockerfile can evolve independently
- **Easier Maintenance**: Simpler to understand and debug individual phases
- **Flexible Building**: Can build any phase independently
- **Better Documentation**: Each directory can have phase-specific docs

**Usage Example**:

```bash
# Build any specific phase
docker build -f docker/provisioned-instance/Dockerfile -t torrust-provisioned:latest .
docker build -f docker/configured-instance/Dockerfile -t torrust-configured:latest .
docker build -f docker/released-instance/Dockerfile -t torrust-released:latest .
```

### Option 2: Multi-Stage Dockerfile

Single Dockerfile with multiple stages:

```dockerfile
# Stage 1: Provisioned Instance
FROM ubuntu:24.04 AS provisioned
RUN apt-get update && apt-get install -y openssh-server sudo supervisor
# ... provisioned setup

# Stage 2: Configured Instance
FROM provisioned AS configured
RUN apt-get install -y docker.io docker-compose
# ... configuration setup

# Stage 3: Released Instance
FROM configured AS released
COPY app-configs/ /opt/configs/
# ... application deployment
```

**Benefits**:

- **Single File**: All phases in one place
- **Layer Sharing**: Efficient layer reuse between stages
- **Consistent Base**: Guaranteed consistency across phases

**Drawbacks**:

- **Complexity**: Single file becomes large and complex
- **All-or-Nothing**: Must understand entire deployment to work on one phase
- **Build Coupling**: Changes to early stages affect all later stages

## Recommendation: Multiple Dockerfiles

For the Torrust deployment infrastructure, **multiple Dockerfiles** is the recommended approach:

### Rationale

1. **Phase Independence**: Each deployment phase has distinct concerns:

   - **Provisioned**: Base system setup, user management, SSH
   - **Configured**: Software installation, system configuration
   - **Released**: Application deployment, service configuration

2. **Testing Isolation**: E2E tests can target specific phases:

   - `e2e-provision-tests` → LXD VMs (infrastructure)
   - `e2e-config-tests` → `provisioned-instance` container
   - `e2e-release-tests` → `configured-instance` container
   - `e2e-run-tests` → `released-instance` container

3. **Development Workflow**: Teams can work on different phases independently:

   - Infrastructure team → `provisioned-instance`
   - Platform team → `configured-instance`
   - Application team → `released-instance`

4. **Debugging**: Phase-specific containers make it easier to isolate issues:
   - Configuration problems → Start from `provisioned-instance`
   - Deployment problems → Start from `configured-instance`

## Implementation Strategy

### Phase 1: ✅ COMPLETED

- [x] `docker/provisioned-instance/` - Base system ready for configuration

### Phase 2: Future

- [ ] `docker/configured-instance/` - System with Docker, dependencies installed
  - Build FROM `torrust-provisioned-instance:latest`
  - Add Ansible playbook execution
  - Verify Docker daemon, Docker Compose installation

### Phase 3: Future

- [ ] `docker/released-instance/` - System with applications deployed
  - Build FROM `torrust-configured-instance:latest`
  - Add application artifacts
  - Add service configurations

### Phase 4: Future

- [ ] `docker/running-instance/` - System with services started and validated
  - Build FROM `torrust-released-instance:latest`
  - Start all services
  - Run validation checks

## Benefits of This Architecture

1. **Test Coverage**: Complete deployment pipeline testing
2. **Fast Feedback**: Test individual phases quickly
3. **Debugging**: Isolate issues to specific deployment phases
4. **Scalability**: Easy to add new phases or modify existing ones
5. **Documentation**: Each phase self-documents its purpose and setup
6. **Reusability**: Containers can be used outside of testing (demos, development)

## Integration with E2E Testing

```text
┌─────────────────────────────────────────────────────────────────┐
│                    E2E Test Split Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐│
│  │ e2e-provision   │    │ e2e-config      │    │ e2e-release     ││
│  │                 │    │                 │    │                 ││
│  │ Tests: LXD VMs  │    │ Tests: Docker   │    │ Tests: Docker   ││
│  │ Infra: OpenTofu │    │ Container:      │    │ Container:      ││
│  │ Validation:     │    │ provisioned-    │    │ configured-     ││
│  │ • VM created    │    │ instance        │    │ instance        ││
│  │ • Cloud-init ✓  │    │                 │    │                 ││
│  │ • SSH ready     │    │ Validation:     │    │ Validation:     ││
│  └─────────────────┘    │ • Docker ✓      │    │ • Apps deployed ││
│                         │ • Compose ✓     │    │ • Services up   ││
│                         │ • Dependencies  │    │ • Certs valid   ││
│                         └─────────────────┘    └─────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

This architecture provides comprehensive testing while maintaining clear separation of concerns and development workflows.
