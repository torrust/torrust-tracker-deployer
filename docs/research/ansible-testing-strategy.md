# Ansible Playbook Testing Strategy

## Overview

This document outlines the testing strategy for Ansible playbooks in the Torrust Tracker Deploy project. Based on comprehensive research comparing Docker containers vs LXD containers, we have established an LXD-exclusive approach for all Ansible testing.

## Updated Strategy (September 2025)

After extensive research and testing (documented in [docker-vs-lxd-ansible-testing.md](docker-vs-lxd-ansible-testing.md)), we have adopted a **single-platform LXD strategy** that prioritizes:

- **Completeness**: Full infrastructure and application testing capabilities
- **Realism**: Production-equivalent environment behavior
- **Efficiency**: VM reuse for multiple playbook executions
- **Sequential Testing**: Playbooks executed in deployment order

## Requirements

- Test playbooks like `wait-cloud-init.yml` that require cloud-init execution
- Support for infrastructure playbooks (Docker installation, firewall setup, etc.)
- Support for application deployment playbooks (Docker Compose stacks)
- Test playbook dependencies and integration scenarios in deployment order
- Each playbook should validate its own pre-conditions and post-conditions
- Reuse provisioned VMs for efficiency across multiple playbook tests

## Current Strategy: LXD-Only Approach

### Core Decision

**Use LXD containers exclusively** for all Ansible playbook testing based on research findings that demonstrate:

1. ✅ **Complete functionality**: Supports all required features (systemd, Docker daemon, networking)
2. ✅ **Real testing**: Can validate actual service deployment and functionality
3. ✅ **Production equivalence**: Behaves like actual cloud VMs
4. ✅ **Reasonable performance**: ~17s setup + ~5s per playbook is acceptable
5. ✅ **Consistent workflow**: Single testing approach reduces complexity
6. ✅ **CI/CD ready**: Proven to work in GitHub Actions

### Testing Workflow

#### 1. VM Provisioning and Reuse

```bash
# Provision LXD container once
cd build/tofu/lxd
tofu apply -auto-approve  # ~17.6s initial setup

# Reuse the same VM for multiple playbook tests
cd ../../ansible
```

#### 2. Sequential Playbook Execution

Execute playbooks in the order they would run during actual deployment:

```bash
# Infrastructure Setup (in dependency order)
time ansible-playbook wait-cloud-init.yml           # ~2.5s
time ansible-playbook install-docker.yml            # ~27.7s
time ansible-playbook install-docker-compose.yml    # ~5.6s
time ansible-playbook setup-firewall.yml            # ~3.5s

# Application Deployment
time ansible-playbook deploy-docker-stack.yml       # ~22.6s
```

#### 3. VM Cleanup (When Needed)

```bash
cd build/tofu/lxd
tofu destroy -auto-approve  # Clean slate for next test cycle
```

### Benefits of This Approach

- **Realistic Testing**: Full systemd, cloud-init, and Docker daemon support
- **Dependency Validation**: Tests actual playbook execution chains
- **Performance Efficiency**: VM reuse eliminates repeated provisioning overhead
- **Integration Testing**: Validates that playbooks work together as intended
- **Production Parity**: Matches actual cloud deployment environment

## Previously Evaluated Alternatives

> **Note**: The following alternatives were extensively researched but determined to be insufficient for our comprehensive testing needs. See [Decision Record: Rejecting Docker for Ansible Testing](../decisions/docker-testing-rejection.md) for detailed analysis.

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

### LXD-Only Testing Strategy

Based on comprehensive research and performance testing, we have implemented a **single-platform LXD strategy** that provides:

#### Core Components

1. **Single VM Reuse**: Provision one LXD container and reuse for multiple playbook tests
2. **Sequential Execution**: Run playbooks in deployment order to test real integration scenarios
3. **Complete Environment**: Full systemd, cloud-init, Docker daemon, and networking support
4. **Production Parity**: LXD containers behave identically to cloud VMs

#### Implementation Details

**Base Infrastructure**:

- Ubuntu 24.04 LXD containers with cloud-init support
- SSH access using fixtures/testing_rsa keys
- OpenTofu for consistent VM provisioning

**Test Execution Pattern**:

```bash
# One-time setup
cd build/tofu/lxd && tofu apply -auto-approve

# Sequential playbook testing (reusing same VM)
cd ../../ansible
ansible-playbook wait-cloud-init.yml
ansible-playbook install-docker.yml
ansible-playbook install-docker-compose.yml
ansible-playbook setup-firewall.yml
ansible-playbook deploy-docker-stack.yml

# Cleanup when needed
cd ../tofu/lxd && tofu destroy -auto-approve
```

**Performance Metrics**:

- Initial VM provisioning: ~17.6 seconds
- Playbook execution: ~3-28 seconds per playbook
- Total integration test cycle: ~60-80 seconds (all playbooks)

#### Built-in Validation Requirements

Each playbook must include:

- **Pre-condition checks**: Verify environment requirements before execution
- **Post-condition validation**: Confirm expected state after execution
- **Idempotency testing**: Ensure safe re-execution
- **Error handling**: Graceful failure with clear messaging

## Rationale for LXD-Only Approach

## Rationale for LXD-Only Approach

This strategy addresses our specific needs while avoiding the limitations discovered in alternative approaches:

- **Complete Testing Coverage**: Unlike Docker containers, LXD supports full systemd services, Docker daemon operations, and complex networking scenarios
- **Production Equivalence**: LXD containers behave identically to cloud VMs, ensuring test results accurately predict production behavior
- **Efficient Resource Usage**: VM reuse eliminates the overhead of repeated provisioning while maintaining test isolation through playbook idempotency
- **Simplified Workflow**: Single testing platform reduces complexity and maintenance overhead
- **Real Integration Testing**: Sequential playbook execution validates actual deployment scenarios and dependencies

## Implementation Status

### Completed

- ✅ LXD infrastructure setup with OpenTofu
- ✅ SSH key management using fixtures/ directory
- ✅ Performance benchmarking and optimization
- ✅ Basic playbook testing (install-docker.yml, setup-firewall.yml)
- ✅ Application deployment testing (deploy-docker-stack.yml)

### In Progress

- 🔄 Standardizing playbook validation patterns
- 🔄 Creating comprehensive test scenarios
- 🔄 Documentation of best practices

### Next Steps

1. **Enhance Existing Playbooks**: Add pre/post-condition validation to all playbooks
2. **Create Test Scenarios**: Develop standard test cases for different playbook types
3. **CI/CD Integration**: Implement automated testing in GitHub Actions
4. **Monitoring and Metrics**: Track test performance and reliability over time
