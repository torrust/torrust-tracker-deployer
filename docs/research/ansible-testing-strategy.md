# Ansible Playbook Testing Strategy

## Overview

This document outlines the testing strategy for Ansible playbooks in the Torrust Tracker Deployer project. Based on comprehensive research and the implementation of **E2E Test Split Architecture**, we have established a **phase-specific testing approach** that uses both LXD containers and Docker containers optimally for different deployment phases.

## Updated Strategy (September 2025)

After extensive research, testing, and architecture evolution, we have adopted a **phase-specific testing strategy** that prioritizes:

- **Phase Separation**: Different deployment phases tested with optimal technologies
- **Completeness**: Full coverage from infrastructure provisioning to application deployment
- **Efficiency**: Fast feedback loops where possible, comprehensive testing where necessary
- **Production Parity**: Test environments that accurately reflect production behavior

## E2E Test Split Architecture

### Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Ansible Testing Strategy                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐              ┌─────────────────────────────┐ │
│  │ Provision Phase │              │ Configuration Phase         │ │
│  │                 │              │                             │ │
│  │ 🖥️  LXD VMs     │              │ 🐳 Docker Containers       │ │
│  │ 🏗️  OpenTofu    │              │ 📋 Ansible Playbooks       │ │
│  │ ☁️  Cloud-init   │              │ 🔧 provisioned-instance    │ │
│  │                 │              │                             │ │
│  │ Tests:          │              │ Tests:                      │ │
│  │ • VM creation   │              │ • install-docker.yml        │ │
│  │ • SSH access    │              │ • install-docker-compose    │ │
│  │ • User setup    │              │ • update-apt-cache.yml      │ │
│  │ • Cloud-init ✓  │              │ • Service configuration     │ │
│  └─────────────────┘              └─────────────────────────────┘ │
│         ~20-30s                           ~3-5s per playbook      │
└─────────────────────────────────────────────────────────────────┘
```

## Requirements by Deployment Phase

### Provision Phase Testing (LXD VMs)

- ✅ Test VM provisioning and infrastructure creation
- ✅ Validate cloud-init execution and completion
- ✅ Verify SSH access and user setup
- ✅ Confirm network connectivity and basic system state
- ✅ Test OpenTofu infrastructure management

### Configuration Phase Testing (Docker Containers)

- ✅ Test Ansible playbooks like `install-docker.yml`, `install-docker-compose.yml`
- ✅ Validate software installation and system configuration
- ✅ Support Docker-in-Docker scenarios for application deployment
- ✅ Test service management and configuration
- ✅ Verify package downloads and dependency installation

## Current Strategy: Phase-Specific Testing Approach

### Core Decision

**Use optimal technology per deployment phase** based on research findings and E2E test split architecture:

1. **Provision Phase**: **LXD VMs** for complete infrastructure testing

   - ✅ Full VM provisioning and cloud-init testing
   - ✅ Real networking and system-level capabilities
   - ✅ Production-equivalent environment behavior
   - ⏱️ ~20-30s setup time (acceptable for comprehensive testing)

2. **Configuration Phase**: **Docker Containers** for fast Ansible testing
   - ✅ Fast feedback loops (~3-5s setup time)
   - ✅ Docker-in-Docker support for application deployment
   - ✅ Sufficient system capabilities for software installation
   - ✅ Reliable GitHub Actions compatibility

### Testing Workflows

#### 1. Provision Phase Testing (e2e-provision-tests)

```bash
# Test infrastructure provisioning with LXD VMs
cargo run --bin e2e-provision-tests

# What this tests:
# - VM creation via OpenTofu + LXD
# - Cloud-init completion and user setup
# - SSH access and connectivity
# - Basic system state validation
# Time: ~20-30 seconds
```

#### 2. Configuration Phase Testing (e2e-config-tests)

```bash
# Test software installation with Docker containers
cargo run --bin e2e-config-tests

# What this tests:
# - Container creation (provisioned-instance state)
# - Ansible playbook execution:
#   - install-docker.yml
#   - install-docker-compose.yml
#   - update-apt-cache.yml
# - Service configuration and validation
# Time: ~10-15 seconds total
```

#### 3. Parallel Execution

Both test suites can run independently and in parallel:

```bash
# Run both test phases simultaneously
cargo run --bin e2e-provision-tests &
cargo run --bin e2e-config-tests &
wait
```

### Benefits of Phase-Specific Approach

- **Optimal Performance**: Fast feedback for configuration changes (~3-5s), comprehensive testing for infrastructure (~20-30s)
- **Better Isolation**: Infrastructure issues don't block configuration testing and vice versa
- **Parallel Execution**: Both phases can run simultaneously, reducing total test time
- **Technology Fit**: Each phase uses the most appropriate technology for its requirements
- **Easier Debugging**: Clear separation makes it easier to identify whether issues are infrastructure or configuration related
- **CI/CD Optimized**: Docker containers avoid GitHub Actions networking issues while LXD provides complete VM testing

## Previously Evaluated Alternatives

> **Note**: The following alternatives were extensively researched and informed our final **phase-specific approach**. See decision records:
>
> - [Docker Testing Evolution](../decisions/docker-testing-evolution.md) - Complete evolution from rejection to acceptance
> - [Docker Testing Revision](../decisions/docker-testing-revision.md) - Updated decision for configuration phase

### 1. Molecule with Docker Driver

**Description**: Official Ansible testing framework using Docker containers as test instances.

**Pros**:

- Fast execution (containers vs VMs)
- Built specifically for Ansible testing
- Supports multiple scenarios and test phases
- Good integration with pytest for advanced testing
- Can test idempotency automatically

**Cons**:

- Limited systemd support in containers (affects service-related playbooks)
- Cannot test kernel modules or low-level system features
- Docker-specific networking might not reflect real VM behavior

### 2. Vagrant with VirtualBox/LibVirt

**Description**: Use Vagrant to provision VMs for testing playbooks in isolated environments.

**Pros**:

- Full VM isolation between tests
- Complete OS environment (systemd, kernel modules, etc.)
- Can test cloud-init integration accurately
- Supports multiple OS distributions

**Cons**:

- Slower than containers (VM startup overhead)
- Higher resource consumption
- More complex setup and maintenance

### 3. LXD/LXC Containers

**Description**: Use LXD containers which provide VM-like features with container speed.

**Pros**:

- Faster than VMs, slower than Docker but more complete
- Better systemd support than Docker
- Can run cloud-init properly
- Good isolation between tests

**Cons**:

- Requires LXD setup and configuration
- Less portable than Docker solutions
- Still some limitations compared to full VMs

### 4. GitHub Actions with Matrix Strategy

**Description**: Run tests in CI using different OS versions in GitHub Actions runners.

**Pros**:

- Real Ubuntu/Debian environments
- Parallel execution across OS versions
- No local resource consumption
- Easy integration with existing CI

**Cons**:

- Slower feedback loop
- Limited to GitHub-supported OS versions
- Cannot test certain networking scenarios
- Shared runner limitations

### 5. Testinfra with Docker Compose

**Description**: Use Docker Compose to orchestrate test environments and Testinfra to verify playbook results.

**Pros**:

- Fast container-based testing
- Can test multi-host scenarios easily
- Python-based assertions (familiar for many)
- Good for testing final states rather than Ansible execution

**Cons**:

- Tests the result, not the Ansible execution itself
- Requires maintaining both playbooks and test code
- Limited system-level testing capabilities

### 6. Kitchen-Ansible with Various Drivers

**Description**: Use Test Kitchen (from Chef ecosystem) with Ansible provisioner and multiple drivers.

**Pros**:

- Supports multiple virtualization backends
- Good lifecycle management (create, converge, verify, destroy)
- Can switch drivers based on test requirements

**Cons**:

- Less Ansible-native than Molecule
- Additional learning curve
- Primarily designed for Chef, adapted for Ansible

## Selected Strategy

### Phase-Specific Testing Architecture

Based on comprehensive research, performance testing, and E2E test split implementation, we have adopted a **phase-specific testing strategy** that provides:

#### Core Components

1. **Provision Phase (LXD VMs)**:

   - Complete infrastructure provisioning testing
   - Cloud-init and VM lifecycle validation
   - OpenTofu infrastructure management testing
   - Production-equivalent environment behavior

2. **Configuration Phase (Docker Containers)**:

   - Fast Ansible playbook testing
   - Software installation and configuration validation
   - Docker-in-Docker capability for application deployment
   - Efficient CI/CD pipeline integration

3. **Container Architecture**: `docker/provisioned-instance/`
   - Ubuntu 24.04 LTS base (matches production VMs)
   - SSH server via supervisor (not systemd)
   - Password + SSH key authentication support
   - Ready state for configuration phase testing

#### Implementation Details

**Provision Phase Testing**:

- Ubuntu 24.04 LXD containers with cloud-init support
- OpenTofu infrastructure management
- SSH access using fixtures/testing_rsa keys
- VM creation and basic system validation

**Configuration Phase Testing**:

- `docker/provisioned-instance/` container simulation
- Ansible connectivity via SSH (password + key auth)
- Docker container lifecycle management
- Software installation and service configuration

**Test Execution Patterns**:

```bash
# Provision phase (runs independently)
cargo run --bin e2e-provision-tests
# - VM creation ~20-30s
# - Cloud-init validation
# - SSH connectivity verification
# - Cleanup and resource management

# Configuration phase (runs independently)
cargo run --bin e2e-config-tests
# - Container creation ~2-3s
# - SSH key setup via password auth
# - Ansible playbook execution
# - Software installation validation
# - Container cleanup
```

**Performance Metrics**:

- Provision phase: ~20-30 seconds (comprehensive infrastructure testing)
- Configuration phase: ~10-15 seconds (fast configuration validation)
- **Total parallel execution**: ~20-30 seconds (both phases simultaneously)
- **Significant improvement**: 50-60% faster than sequential LXD-only approach

#### Built-in Validation Requirements

Each playbook must include:

- **Pre-condition checks**: Verify environment requirements before execution
- **Post-condition validation**: Confirm expected state after execution
- **Idempotency testing**: Ensure safe re-execution
- **Error handling**: Graceful failure with clear messaging

## Rationale for LXD-Only Approach

## Rationale for Phase-Specific Approach

This strategy addresses our specific needs by leveraging the best aspects of both technologies while avoiding their limitations:

### Why LXD for Provision Phase

- **Complete VM Testing**: Full cloud-init, networking, and system-level capability testing
- **Production Equivalence**: LXD containers behave identically to cloud VMs
- **Infrastructure Focus**: Tests actual VM provisioning and infrastructure management
- **OpenTofu Integration**: Natural fit for infrastructure-as-code testing

### Why Docker for Configuration Phase

- **Fast Feedback**: ~3-5s setup enables rapid configuration development cycles
- **Docker-in-Docker**: Proven capability for application deployment testing (research validated)
- **CI/CD Optimized**: Avoids GitHub Actions networking issues encountered with LXD
- **Sufficient Capabilities**: Meets all requirements for software installation and configuration testing

### Combined Benefits

- **Optimal Performance**: Fast where possible, comprehensive where necessary
- **Better Test Isolation**: Infrastructure and configuration issues don't interfere with each other
- **Parallel Execution**: Significant time savings through concurrent test execution
- **Technology Appropriateness**: Each phase uses the most suitable technology
- **Easier Maintenance**: Clear separation of concerns reduces complexity

## Implementation Status

### Completed ✅

- ✅ **Provision Phase Testing**: `e2e-provision-tests` binary with LXD VM testing
- ✅ **Docker Container Infrastructure**: `docker/provisioned-instance/` configuration
- ✅ **SSH Authentication**: Password + SSH key support for container connectivity
- ✅ **Supervisor Integration**: Container-friendly process management
- ✅ **Performance Benchmarking**: Validated phase-specific approach benefits
- ✅ **Decision Documentation**: Comprehensive decision records and architecture docs

### In Progress 🔄

- 🔄 **Configuration Phase Testing**: `e2e-config-tests` binary implementation (Phase B.3)
- 🔄 **Container Lifecycle Management**: Integration with test binary
- 🔄 **Ansible Playbook Integration**: Testing install-docker.yml, install-docker-compose.yml
- 🔄 **GitHub Actions Workflow**: CI/CD pipeline for configuration testing

### Next Steps 📋

1. **Complete Phase B.3**: Implement `e2e-config-tests` binary with Docker container integration
2. **Playbook Validation**: Add comprehensive pre/post-condition checking to all playbooks
3. **CI/CD Integration**: Implement parallel test execution in GitHub Actions
4. **Future Container Phases**: Design `configured-instance`, `released-instance` containers
5. **Monitoring and Metrics**: Track test performance and reliability over time

## Related Documentation

- [E2E Tests Split Plan](../refactors/split-e2e-tests-provision-vs-configuration.md)
- [Docker Testing Revision Decision](../decisions/docker-testing-revision.md)
- [Docker Phase Architecture](../decisions/docker-phase-architecture.md)
- [Docker Configuration Testing Research](./e2e-docker-config-testing.md)
- [Docker Testing Evolution](../decisions/docker-testing-evolution.md) - Complete strategy evolution
