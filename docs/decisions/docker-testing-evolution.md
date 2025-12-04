# Decision Record: Docker Container Testing Strategy Evolution

**Date**: September 2, 2025 (Original) / September 23, 2025 (Revision)  
**Status**: Resolved  
**Decision Makers**: Infrastructure Team  
**Related Research**: [Docker vs LXD Ansible Testing Research](../research/docker-vs-lxd-ansible-testing.md)  
**Related**: [E2E Tests Split](../refactors/split-e2e-tests-provision-vs-configuration.md)

## Summary

This document chronicles the complete evolution of our Docker container testing strategy, from initial rejection to final adoption for configuration testing. The journey involved discovering fundamental limitations, architectural changes, and ultimately finding the right scope for Docker containers in our testing strategy.

## Context and Problem Statement

## Phase 1: Initial Docker Rejection (September 2, 2025)

### Original Problem

During the development of the Torrust Tracker Deployer project, we needed to establish a testing strategy for Ansible playbooks. The initial hypothesis was that Docker containers could provide faster testing cycles compared to full VMs or LXD containers, potentially offering significant development velocity improvements.

The core question was: **Should we use lightweight Docker containers for Ansible playbook testing to achieve faster feedback loops?**

### Decision Drivers

- **Development Speed**: Faster test cycles enable quicker iteration
- **Resource Efficiency**: Lower resource consumption for CI/CD pipelines
- **Comprehensive Testing**: Need to test both infrastructure and application deployment playbooks
- **Production Parity**: Test environment should behave like production cloud VMs
- **Maintenance Overhead**: Simpler deployment infrastructure reduces long-term costs

### Considered Options

#### Option A: Docker Containers for Testing

**Approach**: Use Docker containers as Ansible testing targets

**Pros**:

- Fast container startup (~2-5 seconds)
- Lightweight resource usage
- Easy to create multiple test scenarios
- Good integration with CI/CD systems
- Familiar technology stack

**Cons**:

- Limited systemd support in containers
- Cannot test Docker-in-Docker scenarios reliably
- Restricted networking capabilities
- Missing kernel-level features
- Cannot validate cloud-init integration

#### Option B: LXD Containers for Testing

**Approach**: Use LXD containers as Ansible testing targets

**Pros**:

- Full systemd support
- Real Docker daemon capabilities
- Complete networking stack
- Cloud-init compatibility
- Production-equivalent behavior

**Cons**:

- Slower than Docker containers (~17 seconds setup)
- Higher resource usage
- More complex initial setup

#### Option C: Hybrid Approach

**Approach**: Use Docker for basic playbooks, LXD for complex scenarios

**Pros**:

- Optimized speed for simple tests
- Complete coverage for complex scenarios

**Cons**:

- Increased maintenance complexity
- Dual deployment infrastructure
- Potential inconsistencies between environments

### Original Decision: LXD Containers Exclusively

**Chosen Option**: Option B - LXD Containers Exclusively

**Rationale**: After comprehensive research and testing, we reject Docker containers for Ansible testing in favor of LXD containers for the following critical reasons:

### Key Findings That Led to Initial Rejection

#### 1. Docker-in-Docker Impossibility

**Problem**: Core Torrust infrastructure requires Docker daemon functionality for application deployment.

**Evidence**:

```bash
# Docker container test result
TASK [Start Docker service] *****
fatal: [torrust-docker]: FAILED! => {
    "msg": "Could not start docker: Cannot connect to the Docker daemon at unix:///var/run/docker.sock"
}
```

**Impact**: Cannot test real application deployment scenarios that require Docker Compose stack management.

#### 2. Systemd Service Management Failures

**Problem**: Many infrastructure playbooks require systemd service management.

**Evidence**:

```bash
# Docker container limitation
TASK [Enable UFW firewall service] *****
fatal: [torrust-docker]: FAILED! => {
    "msg": "Could not enable service ufw: System has not been booted with systemd"
}
```

**Impact**: Cannot test essential infrastructure services like firewalls, networking, or service management.

#### 3. Limited Network Configuration Testing

**Problem**: Firewall and networking configuration requires kernel-level capabilities.

**Evidence**:

- UFW firewall cannot be enabled in Docker containers
- iptables manipulation is restricted
- Network interface management is limited

**Impact**: Cannot validate network security configurations that are critical for production deployment.

#### 4. Cloud-Init Integration Gap

**Problem**: Cloud-init testing cannot be properly simulated in Docker containers.

**Evidence**:

- No real cloud-init process execution
- Missing cloud metadata simulation
- Cannot test cloud-init dependent initialization sequences

**Impact**: Cannot validate the complete VM initialization process that occurs in production cloud environments.

### Performance Analysis (Original)

Despite Docker's speed advantage, the performance difference is not significant enough to justify the functional limitations:

| Metric               | Docker Container | LXD Container  | Difference     |
| -------------------- | ---------------- | -------------- | -------------- |
| Initial Setup        | ~3-5 seconds     | ~17.6 seconds  | +12-14 seconds |
| Playbook Execution   | ~4-5 seconds     | ~3-28 seconds  | Variable       |
| **Total Test Cycle** | ~7-10 seconds    | ~20-45 seconds | +13-35 seconds |

**Analysis**: The 13-35 second overhead is acceptable when weighed against the comprehensive testing capabilities that LXD provides.

### Alternative Approaches Considered and Rejected (Original)

#### 1. Simulation-Based Testing

**Approach**: Mock Docker daemon and systemd services in Docker containers
**Rejection Reason**: Testing mocks instead of real services provides false confidence

#### 2. Sequential Testing Pipeline

**Approach**: Basic tests in Docker, comprehensive tests in LXD
**Rejection Reason**: Dual infrastructure complexity outweighs benefits; inconsistent results between environments

#### 3. Enhanced Docker Images

**Approach**: Pre-install Docker daemon and systemd in Docker containers
**Rejection Reason**: Cannot overcome fundamental Docker-in-Docker and kernel-level limitations

### Implementation Consequences (Original Decision)

#### Positive Consequences

- **Complete Test Coverage**: All infrastructure and application playbooks can be tested
- **Production Parity**: Test results accurately predict production behavior
- **Single Testing Platform**: Reduced complexity and maintenance overhead
- **Real Integration Testing**: Can validate complete deployment workflows

#### Negative Consequences

- **Slower Initial Feedback**: ~17 seconds setup vs ~3 seconds for Docker
- **Higher Resource Usage**: LXD containers consume more memory and CPU
- **Setup Complexity**: LXD requires more initial configuration than Docker

#### Mitigation Strategies

- **VM Reuse**: Reuse LXD containers across multiple playbook tests to amortize setup costs
- **Sequential Testing**: Execute playbooks in deployment order to test real integration scenarios
- **Parallel CI**: Run multiple LXD containers in parallel for different test scenarios

## Phase 2: Context Change and Strategy Revision (September 23, 2025)

### What Changed

The original decision to reject Docker containers for Ansible testing was **superseded** by the implementation of the **E2E Test Split Architecture** due to GitHub Actions connectivity issues with LXD VMs.

The **fundamental assumptions** in the original decision no longer applied:

1. **Testing Scope Separation**: We no longer test provisioning + configuration in a single test
2. **Cloud-Init Irrelevance**: Configuration tests don't need cloud-init (that's tested in provision phase)
3. **Docker-in-Docker Feasibility**: Research proved Docker installation inside Docker containers works
4. **Phase-Specific Requirements**: Different deployment phases have different testing needs
5. **GitHub Actions Constraints**: Network connectivity issues inside LXD VMs forced architectural changes

### New Architecture Overview

```text
┌─────────────────────────────────────────────────────────┐
│               E2E Test Split Architecture               │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐              ┌─────────────────────┐ │
│  │ e2e-provision   │              │ e2e-config          │ │
│  │                 │              │                     │ │
│  │ ✅ LXD VMs      │              │ ✅ Docker Container │ │
│  │ ✅ OpenTofu     │              │ ✅ Ansible Testing  │ │
│  │ ✅ Cloud-init   │              │ ✅ Docker-in-Docker │ │
│  │ ✅ VM Creation  │              │ ✅ Service Mgmt     │ │
│  └─────────────────┘              └─────────────────────┘ │
│         │                                    │             │
│         ▼                                    ▼             │
│  Tests Infrastructure           Tests Configuration        │
│  Provisioning                  & Application Deployment    │
└─────────────────────────────────────────────────────────┘
```

### Revised Decision: Hybrid Approach

#### ❌ **Original Decision**: Rejected Docker for All Testing

**Reason**: Could not test complete infrastructure + configuration together

#### ✅ **New Decision**: Docker for Configuration Testing

**Reason**: Configuration testing is now **isolated** and Docker containers meet all requirements:

#### ✅ **Docker-in-Docker Works**

- **Research Evidence**: [Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support)
- **Proven**: Docker daemon can be installed and run inside Docker containers
- **Tested**: Ansible playbooks successfully install Docker and Docker Compose

#### ✅ **Cloud-Init Not Needed**

- **Scope**: Configuration tests assume VM is already provisioned
- **Container State**: `provisioned-instance` represents post-provision, pre-configuration state
- **Focus**: Test software installation, not VM initialization

#### ✅ **Systemd Alternative**

- **Solution**: Use supervisor for process management in containers
- **Benefit**: Container-native approach, more reliable than systemd-in-container
- **Result**: SSH service management works perfectly

#### ✅ **Network Capabilities Sufficient**

- **Requirement**: Package downloads during configuration phase
- **Docker**: Provides full internet access for apt/yum package installation
- **Result**: All network requirements met for configuration testing

### Implementation Results

#### Configuration Testing with Docker Containers

**Container**: `docker/provisioned-instance/`

- ✅ Ubuntu 24.04 base (matches production VMs)
- ✅ SSH server via supervisor (Ansible connectivity)
- ✅ Password + SSH key authentication
- ✅ Internet access for package downloads
- ✅ Sudo user matching LXD VM configuration

**Test Results**:

```bash
✅ Container builds successfully
✅ SSH authentication works (password & key)
✅ Ansible can connect and execute playbooks
✅ Docker installation inside container works
✅ No privileged mode required
✅ Fast startup (~2-3 seconds vs ~17 seconds for LXD)
```

#### Performance Comparison

| Phase         | Technology       | Setup Time     | Use Case                                    |
| ------------- | ---------------- | -------------- | ------------------------------------------- |
| **Provision** | LXD VMs          | ~17-30 seconds | Infrastructure creation, cloud-init testing |
| **Configure** | Docker Container | ~2-3 seconds   | Software installation, system configuration |
| **Release**   | Docker Container | ~2-3 seconds   | Application deployment testing              |
| **Run**       | Docker Container | ~2-3 seconds   | Service validation, monitoring setup        |

## Final Decision: Phase-Specific Testing Strategy

### ✅ **Best of Both Worlds**

- **LXD VMs**: Complete infrastructure testing where needed (provision phase)
- **Docker Containers**: Fast configuration testing where sufficient (configure/release/run phases)

### Benefits of Final Approach

#### ✅ **Faster Feedback Loops**

- **Provision Testing**: Still comprehensive but isolated to infrastructure concerns
- **Configuration Testing**: 10x faster feedback for application deployment issues

#### ✅ **Better CI/CD Performance**

- **Parallel Execution**: Provision and configuration tests can run in parallel
- **Resource Efficiency**: Configuration tests use minimal resources
- **GitHub Actions Reliability**: Docker containers avoid LXD networking issues

#### ✅ **Clearer Test Failures**

- **Infrastructure Issues**: Isolated to provision tests
- **Configuration Issues**: Isolated to configuration tests
- **Easier Debugging**: Clear separation of concerns

### What This Final Decision Encompasses

#### Still Using LXD ✅ (Provision Phase)

- **Complete E2E Testing**: LXD VMs for full infrastructure testing
- **Cloud-Init Testing**: LXD VMs can test cloud-init scenarios
- **Production Parity**: LXD provides VM-like environment for provisioning validation

#### Now Using Docker ✅ (Configuration Phase)

- **Configuration Testing**: Docker is suitable for configuration phase testing
- **Docker-in-Docker**: Proven to work for configuration scenarios
- **Performance Benefits**: Faster feedback loops for configuration issues

### Current Implementation Status

This hybrid approach has been **successfully implemented**:

- ✅ `e2e-provision-and-destroy-tests` binary - Tests infrastructure provisioning and destruction lifecycle with LXD VMs
- ✅ `e2e-config-and-release-tests` binary - Tests configuration and release with Docker containers
- ✅ `e2e-tests-full` binary - Complete local testing (LXD + Docker) for development

## Monitoring and Review

## Monitoring and Review

This evolutionary decision will be reviewed if:

- GitHub Actions improves LXD VM network connectivity
- Docker container technology limitations are discovered in configuration testing
- Alternative lightweight virtualization technologies emerge
- Testing requirements change significantly
- The split testing approach proves insufficient for comprehensive validation

## References

- [Docker vs LXD Ansible Testing Research](../research/docker-vs-lxd-ansible-testing.md)
- [Ansible Testing Strategy](../research/ansible-testing-strategy.md)
- [E2E Tests Split Plan](../refactors/split-e2e-tests-provision-vs-configuration.md)
- [Docker Configuration Testing Research](../research/e2e-docker-config-testing.md)
- [Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support)
- [GitHub Actions LXD Connectivity Issues](https://github.com/actions/runner-images/issues/13003)
- [Docker-in-Docker Limitations Documentation](https://docs.docker.com/engine/security/rootless/#known-limitations)
- [LXD vs Docker Comparison](https://ubuntu.com/blog/lxd-vs-docker)
