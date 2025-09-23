# Docker Provisioned Instance Configuration

This directory contains the Docker configuration representing a **provisioned instance** - the state of a VM after provisioning but before configuration in the deployment lifecycle.

## Overview

This Docker configuration provides an Ubuntu 24.04 container that simulates a freshly provisioned VM:

- **SSH Server**: For Ansible connectivity (via supervisor)
- **Base System**: Clean Ubuntu 24.04 LTS installation
- **Sudo User**: `torrust` user with passwordless sudo access
- **Network Access**: For package downloads during configuration phase
- **No App Dependencies**: Docker, Docker Compose, etc. not yet installed (that's the configure phase)

## Files

- `Dockerfile`: Main container configuration for provisioned instance state
- `supervisord.conf`: Supervisor configuration for SSH service management
- `entrypoint.sh`: Container initialization script
- `README.md`: This documentation file

## Deployment Phase Context

This container represents the **provisioned** state in the deployment lifecycle:

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Provision    │───▶│   Configure     │───▶│    Release      │───▶│      Run        │
│                 │    │                 │    │                 │    │                 │
│ • VM Created    │    │ • Install Docker│    │ • Deploy Apps   │    │ • Start Services│
│ • SSH Ready     │    │ • Install Deps  │    │ • Config Files  │    │ • Validate      │
│ • User Setup    │    │ • System Config │    │ • Certificates  │    │ • Monitor       │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
       ▲
   THIS CONTAINER
```

**Future Expansion**: Additional containers can represent later phases:

- `docker/configured-instance/` - After Ansible configuration
- `docker/released-instance/` - After application deployment

## Usage

### Building the Container

From the project root directory:

```bash
# Build the provisioned instance Docker image
docker build -f docker/provisioned-instance/Dockerfile -t torrust-provisioned-instance:latest .
```

### Running the Container

#### Basic Run (for testing)

```bash
# Run provisioned instance container with SSH access
docker run -d \
  --name torrust-provisioned \
  -p 2222:22 \
  torrust-provisioned-instance:latest
```

#### Connect via SSH

```bash
# Connect using password authentication (initial setup)
sshpass -p "torrust123" ssh -p 2222 -o StrictHostKeyChecking=no torrust@localhost

# Or copy SSH key and use key authentication
sshpass -p "torrust123" scp -P 2222 -o StrictHostKeyChecking=no fixtures/testing_rsa.pub torrust@localhost:~/.ssh/authorized_keys
ssh -i fixtures/testing_rsa -p 2222 -o StrictHostKeyChecking=no torrust@localhost
```

### Integration with E2E Configuration Tests

The provisioned instance container simulates the state after VM provisioning and is designed for E2E configuration testing:

1. **Container Lifecycle**: Tests manage container creation and cleanup
2. **SSH Authentication**: Initial password authentication (`torrust:torrust123`)
3. **SSH Key Setup**: Tests copy SSH public key during setup phase
4. **Port Mapping**: SSH port (22) is mapped to host for Ansible connectivity
5. **Inventory Generation**: Container IP is added to Ansible inventory

### Configuration Details

#### User Configuration

- **Username**: `torrust` (matches LXD VM configuration)
- **Password**: `torrust123` (for initial SSH access)
- **Groups**: `sudo`
- **Shell**: `/bin/bash`
- **Sudo**: Passwordless sudo access (`NOPASSWD:ALL`)
- **SSH**: Password authentication enabled initially, key-based authentication supported

#### SSH Configuration

- **Port**: 22 (standard SSH port)
- **Authentication**: Password authentication enabled (`torrust123`)
- **Public Key**: Key-based authentication supported (tests copy public key)
- **Root Login**: Disabled

#### Supervisor Configuration

- **Process Manager**: Supervisor instead of systemd (container-friendly)
- **Services**: SSH service managed by supervisor
- **Logging**: Supervisor handles service logging
- **No Privileges**: No `--privileged` flag required

## Requirements

### For Building

- Docker installed on the build system
- Project repository with `fixtures/testing_rsa.pub` file

### For Running

- Docker installed on the system
- No special privileges required (no `--privileged` flag needed)
- SSH client for connectivity testing

## Troubleshooting

### Container Won't Start

1. Check if Docker daemon is running
2. Verify no port conflicts on port 2222
3. Check container logs: `docker logs <container-name>`

### SSH Connection Fails

1. Verify SSH port mapping: `-p 2222:22`
2. Test password authentication: `sshpass -p "torrust123" ssh -p 2222 torrust@localhost`
3. Check if SSH service is running inside container
4. Verify container is accessible: `docker exec -it <container-name> bash`

### Key Authentication Issues

1. Ensure public key is copied correctly to container
2. Verify SSH key file permissions (should be 600)
3. Check authorized_keys file in container: `~/.ssh/authorized_keys`

## Architecture

This container configuration supports the E2E test split architecture:

```text
┌─────────────────────────────────────────┐
│         E2E Config Tests Binary         │
│                                         │
│  ┌─────────────────────────────────────┐│
│  │       Docker Container              ││
│  │  ┌─────────────────────────────────┐││
│  │  │      Ubuntu 24.04 LTS           │││
│  │  │  - SSH Server (port 22)         │││
│  │  │  - Supervisor (process mgmt)    │││
│  │  │  - torrust user (sudo access)   │││
│  │  │  - Package management (apt)     │││
│  │  └─────────────────────────────────┘││
│  └─────────────────────────────────────┘│
│             ▲                           │
│             │ SSH (port 2222)           │
│             ▼                           │
│  ┌─────────────────────────────────────┐│
│  │         Ansible Client              ││
│  │  - install-docker.yml               ││
│  │  - install-docker-compose.yml       ││
│  │  - Dynamic inventory generation     ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

## Related Documentation

- [E2E Tests Split Plan](../../docs/refactors/split-e2e-tests-provision-vs-configuration.md)
- [Docker Configuration Testing Research](../../docs/research/e2e-docker-config-testing.md)
- [E2E Testing Guide](../../docs/e2e-testing.md)
