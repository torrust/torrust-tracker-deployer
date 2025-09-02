# Ansible Playbook Testing Strategy

## Overview

This document outlines the research and decision-making process for implementing automated testing of Ansible playbooks in the Torrust Testing Infrastructure project.

## Requirements

- Test playbooks like `wait-cloud-init.yml` that require cloud-init execution
- Fast testing for future playbooks (Docker installation, firewall setup, etc.)
- Support for testing playbook dependencies and integration scenarios
- Each playbook should validate its own pre-conditions and post-conditions

## Alternatives Evaluated

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

### Hybrid Approach

We have decided to implement a **hybrid testing strategy** that optimizes for both speed and completeness:

#### 1. Molecule with Docker Driver (Primary)

- **Use case**: Most playbooks (Docker installation, firewall setup, package management, etc.)
- **Benefits**: Fast feedback loop, good for development and CI
- **Limitations**: Cannot test systemd services or cloud-init scenarios

#### 2. LXD Containers (Special Cases)

- **Use case**: Cloud-init related tests (like `wait-cloud-init.yml`)
- **Benefits**: Proper cloud-init support, better systemd compatibility
- **Setup**: Use base images similar to `config/tofu/lxd/cloud-init.yml`

#### 3. Integration Test Suite

- **Use case**: Testing playbook dependencies and execution chains
- **Implementation**: Single long-running VM for dependency chain testing
- **Benefits**: Simulates real deployment scenarios

#### 4. Built-in Validation

- **Requirement**: Each playbook includes pre-condition and post-condition checks
- **Implementation**: Use Ansible facts and assertions
- **Benefits**: Self-documenting requirements and outcomes

## Implementation Plan

### Phase 1: Molecule Setup

1. Configure Molecule with Docker driver for basic playbooks
2. Create standardized test scenarios
3. Implement basic idempotency testing

### Phase 2: LXD Integration

1. Set up LXD for cloud-init testing
2. Create base images matching production environment
3. Test `wait-cloud-init.yml` specifically

### Phase 3: Integration Testing

1. Design integration test scenarios
2. Implement playbook dependency chains
3. Create comprehensive test suites

### Phase 4: Validation Framework

1. Add pre/post-condition checks to existing playbooks
2. Standardize assertion patterns
3. Document testing best practices

## Rationale

This hybrid approach addresses our specific needs:

- **Speed**: Docker-based tests provide rapid feedback for most scenarios
- **Completeness**: LXD handles cloud-init and systemd requirements
- **Realism**: Integration tests simulate actual deployment conditions
- **Maintainability**: Built-in validations ensure playbooks are self-documenting

The strategy balances development velocity with test coverage, ensuring we can quickly iterate on playbooks while maintaining confidence in their behavior across different environments.

## Next Steps

1. Set up Molecule framework with Docker driver
2. Configure LXD for cloud-init testing scenarios
3. Begin implementing tests for existing playbooks
4. Establish CI/CD integration for automated testing
