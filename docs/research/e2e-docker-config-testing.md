# E2E Configuration Testing with Docker Containers

## Executive Summary

For Phase B.1 of the E2E test split, this document outlines the research findings and approach for implementing Docker container-based configuration testing to replace LXD VM-based testing for the configuration, release, and run phases.

## Background & Problem Statement

- **Current Issue**: LXD VMs work for provisioning but fail during configuration phase due to network connectivity issues within VMs on GitHub Actions runners
- **Root Cause**: GitHub Actions runners are themselves VMs, creating nested virtualization issues that prevent network connectivity required for software installation
- **Solution**: Split testing into provision (LXD VMs) and configuration (Docker containers) phases

## Configuration Testing Requirements

Based on analysis of the current E2E workflow, the configuration tests need to validate:

### 1. Software Installation (Configure Phase)

- **Docker Installation**: Via Ansible playbook `install-docker.yml`
- **Docker Compose Installation**: Via Ansible playbook `install-docker-compose.yml`
- **APT Cache Updates**: Via `update-apt-cache.yml`
- **Network connectivity**: For package downloads

### 2. Infrastructure Validation (Test Phase)

- **Cloud-init completion**: Verify initialization completed
- **Docker service**: Verify Docker daemon is running
- **Docker Compose**: Verify Docker Compose binary is functional
- **SSH connectivity**: Ensure Ansible can connect and execute commands

### 3. Current Ansible Workflow Integration

- **Inventory management**: Dynamic inventory generation with container IP
- **SSH-based execution**: Ansible connects via SSH to execute playbooks
- **Privilege escalation**: Requires `sudo` access within container
- **Ubuntu 24.04 target**: Current templates target Ubuntu 24.04 LTS

## Docker Container Approach

### Container Requirements

1. **Base Image**: Ubuntu 24.04 LTS (to match current VM environment)
2. **SSH Server**: OpenSSH server for Ansible connectivity
3. **Systemd**: For service management (Docker daemon, etc.)
4. **Sudo Access**: For privilege escalation during software installation
5. **Network Access**: For package downloads and installations
6. **Init System**: Alternative to cloud-init for container initialization

### Container Configuration Strategy

#### Option 1: Custom Dockerfile (Recommended)

- **Base**: `ubuntu:24.04`
- **SSH Setup**: Install and configure OpenSSH server
- **Systemd**: Enable systemd for service management
- **User Setup**: Create user with sudo access
- **Network**: Default Docker networking (sufficient for GitHub Actions)

#### Option 2: Pre-built Image with SSH

- **Base**: Existing Ubuntu images with SSH enabled
- **Pros**: Faster setup, less maintenance
- **Cons**: Less control, may not match exact VM environment

### Cloud-init Alternative

Since cloud-init is VM-specific, containers need alternative initialization:

1. **Container Init Scripts**: Custom initialization via entrypoint script
2. **SSH Key Injection**: Mount SSH keys via Docker volumes or copy
3. **User Provisioning**: Direct user/key setup instead of cloud-init
4. **Service Initialization**: Direct systemd service management

## Research References

### Docker-in-VM Testing Research

- **[Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support)**: Comprehensive testing of virtualization tools on GitHub Actions, demonstrating Docker feasibility
- **[Docker-in-VM Test Repository](https://github.com/josecelano/test-docker-install-inside-vm-in-runner)**: Specific research on Docker installation within VMs on GitHub Actions runners, documenting the network connectivity issues

### Related Issues

- **[GitHub Actions Runner Images Issue #13003](https://github.com/actions/runner-images/issues/13003)**: Network connectivity issues with LXD VMs on GitHub runners
- **[Original Virtualization Investigation](https://github.com/actions/runner-images/issues/12933)**: Background context on GitHub Actions virtualization support

## Testcontainers Integration Analysis

### Benefits of testcontainers-rs

- **Container Lifecycle Management**: Automatic startup/cleanup
- **Network Management**: Automatic port mapping and network configuration
- **Integration**: Well-integrated with Rust testing ecosystem
- **Parallel Testing**: Multiple containers can run in parallel

### Implementation Approach

- **Generic Image**: Use `testcontainers::GenericImage` for Ubuntu container
- **Custom Configuration**: Configure SSH, systemd, and networking
- **Volume Mounting**: SSH keys and test artifacts
- **Port Mapping**: SSH port (22) mapping for Ansible connectivity

### Alternative: Direct Docker CLI

- **Simpler Setup**: Direct `docker run` commands
- **Less Dependencies**: No additional crates required
- **Manual Management**: Explicit container lifecycle management
- **More Control**: Direct control over Docker operations

## Network Configuration

### Ansible Connectivity Requirements

1. **SSH Access**: Container must accept SSH connections
2. **Port Mapping**: Map container SSH port to host
3. **IP Address**: Deterministic container IP for Ansible inventory
4. **DNS Resolution**: Container must resolve package repositories

### GitHub Actions Networking

- **Docker Networking**: Works reliably on GitHub Actions
- **Port Mapping**: Standard Docker port mapping supported
- **Internet Access**: Containers have internet access for package downloads
- **No Nested Virtualization**: Avoids the LXD VM networking issues

## Implementation Plan Summary

### Phase B.1 Deliverables

1. **Docker Configuration**: Create `docker/test-ubuntu/Dockerfile`
2. **Container Setup**: Ubuntu 24.04 with SSH, systemd, sudo user
3. **Integration Strategy**: Document testcontainers vs direct Docker approach
4. **Network Requirements**: Document Ansible connectivity requirements
5. **Cloud-init Alternative**: Design container initialization approach

### Next Steps (B.2+)

1. **Docker Implementation**: Build and test Docker configuration
2. **Binary Creation**: Implement `e2e-config-tests` binary
3. **Container Management**: Integrate container lifecycle with tests
4. **Local Testing**: Validate complete workflow locally
5. **CI Integration**: Create GitHub Actions workflow

## Technical Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                GitHub Actions Runner                    │
│  ┌─────────────────────────────────────────────────────┐│
│  │              e2e-config-tests binary                 ││
│  │  ┌─────────────────────────────────────────────────┐ ││
│  │  │            Docker Container                     │ ││
│  │  │  ┌─────────────────────────────────────────────┐ │ ││
│  │  │  │         Ubuntu 24.04 LTS                   │ │ ││
│  │  │  │  - SSH Server (port 22)                    │ │ ││
│  │  │  │  - Systemd (service management)            │ │ ││
│  │  │  │  - Sudo user (ansible connectivity)        │ │ ││
│  │  │  │  - Package management (apt)                │ │ ││
│  │  │  └─────────────────────────────────────────────┘ │ ││
│  │  └─────────────────────────────────────────────────┘ ││
│  │                      ▲                                ││
│  │                      │ SSH Connection                 ││
│  │                      ▼                                ││
│  │  ┌─────────────────────────────────────────────────┐ ││
│  │  │              Ansible Client                     │ ││
│  │  │  - install-docker.yml                           │ ││
│  │  │  - install-docker-compose.yml                   │ ││
│  │  │  - inventory generation                         │ ││
│  │  └─────────────────────────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

## Risk Assessment

### Low Risk

- **Docker Support**: Well-established and reliable on GitHub Actions
- **Network Connectivity**: Docker containers have consistent internet access
- **Package Installation**: No nested virtualization issues

### Medium Risk

- **Systemd in Containers**: May require special configuration
- **SSH Setup**: Need to ensure SSH server starts correctly
- **Performance**: Container overhead vs VM performance

### Mitigation Strategies

- **Systemd**: Use proven systemd-in-Docker patterns
- **SSH Testing**: Validate SSH connectivity in local testing phase
- **Documentation**: Comprehensive troubleshooting documentation

## Conclusion

Docker containers provide a viable and reliable alternative to LXD VMs for configuration testing. The approach addresses the core network connectivity issues while maintaining compatibility with the existing Ansible-based configuration workflow. The implementation should start with a custom Ubuntu 24.04 Dockerfile and consider testcontainers-rs integration for better test lifecycle management.
