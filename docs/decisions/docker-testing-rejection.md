# Decision Record: Rejecting Docker Containers for Ansible Testing

**Date**: September 2, 2025  
**Status**: ⚠️ **Partially Superseded** (September 23, 2025)  
**Decision Makers**: Infrastructure Team  
**Related Research**: [Docker vs LXD Ansible Testing Research](../research/docker-vs-lxd-ansible-testing.md)  
**Superseded By**: [Docker Testing Revision](./docker-testing-revision.md)

> **⚠️ Status Update**: This decision has been **partially superseded** by the E2E test split architecture.  
> Docker containers are now used for **configuration phase testing** while LXD VMs remain for **provisioning phase testing**.  
> See: [Docker Testing Revision](./docker-testing-revision.md)

## Context and Problem Statement

During the development of the Torrust Tracker Deploy project, we needed to establish a testing strategy for Ansible playbooks. The initial hypothesis was that Docker containers could provide faster testing cycles compared to full VMs or LXD containers, potentially offering significant development velocity improvements.

The core question was: **Should we use lightweight Docker containers for Ansible playbook testing to achieve faster feedback loops?**

## Decision Drivers

- **Development Speed**: Faster test cycles enable quicker iteration
- **Resource Efficiency**: Lower resource consumption for CI/CD pipelines
- **Comprehensive Testing**: Need to test both infrastructure and application deployment playbooks
- **Production Parity**: Test environment should behave like production cloud VMs
- **Maintenance Overhead**: Simpler deployment infrastructure reduces long-term costs

## Considered Options

### Option A: Docker Containers for Testing

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

### Option B: LXD Containers for Testing

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

### Option C: Hybrid Approach

**Approach**: Use Docker for basic playbooks, LXD for complex scenarios

**Pros**:

- Optimized speed for simple tests
- Complete coverage for complex scenarios

**Cons**:

- Increased maintenance complexity
- Dual deployment infrastructure
- Potential inconsistencies between environments

## Decision Outcome

**Chosen Option**: Option B - LXD Containers Exclusively

**Rationale**: After comprehensive research and testing, we reject Docker containers for Ansible testing in favor of LXD containers for the following critical reasons:

## Key Findings That Led to Rejection

### 1. Docker-in-Docker Impossibility

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

### 2. Systemd Service Management Failures

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

### 3. Limited Network Configuration Testing

**Problem**: Firewall and networking configuration requires kernel-level capabilities.

**Evidence**:

- UFW firewall cannot be enabled in Docker containers
- iptables manipulation is restricted
- Network interface management is limited

**Impact**: Cannot validate network security configurations that are critical for production deployment.

### 4. Cloud-Init Integration Gap

**Problem**: Cloud-init testing cannot be properly simulated in Docker containers.

**Evidence**:

- No real cloud-init process execution
- Missing cloud metadata simulation
- Cannot test cloud-init dependent initialization sequences

**Impact**: Cannot validate the complete VM initialization process that occurs in production cloud environments.

## Performance Analysis

Despite Docker's speed advantage, the performance difference is not significant enough to justify the functional limitations:

| Metric               | Docker Container | LXD Container  | Difference     |
| -------------------- | ---------------- | -------------- | -------------- |
| Initial Setup        | ~3-5 seconds     | ~17.6 seconds  | +12-14 seconds |
| Playbook Execution   | ~4-5 seconds     | ~3-28 seconds  | Variable       |
| **Total Test Cycle** | ~7-10 seconds    | ~20-45 seconds | +13-35 seconds |

**Analysis**: The 13-35 second overhead is acceptable when weighed against the comprehensive testing capabilities that LXD provides.

## Alternative Approaches Considered and Rejected

### 1. Simulation-Based Testing

**Approach**: Mock Docker daemon and systemd services in Docker containers
**Rejection Reason**: Testing mocks instead of real services provides false confidence

### 2. Sequential Testing Pipeline

**Approach**: Basic tests in Docker, comprehensive tests in LXD
**Rejection Reason**: Dual infrastructure complexity outweighs benefits; inconsistent results between environments

### 3. Enhanced Docker Images

**Approach**: Pre-install Docker daemon and systemd in Docker containers
**Rejection Reason**: Cannot overcome fundamental Docker-in-Docker and kernel-level limitations

## Implementation Consequences

### Positive Consequences

- **Complete Test Coverage**: All infrastructure and application playbooks can be tested
- **Production Parity**: Test results accurately predict production behavior
- **Single Testing Platform**: Reduced complexity and maintenance overhead
- **Real Integration Testing**: Can validate complete deployment workflows

### Negative Consequences

- **Slower Initial Feedback**: ~17 seconds setup vs ~3 seconds for Docker
- **Higher Resource Usage**: LXD containers consume more memory and CPU
- **Setup Complexity**: LXD requires more initial configuration than Docker

### Mitigation Strategies

- **VM Reuse**: Reuse LXD containers across multiple playbook tests to amortize setup costs
- **Sequential Testing**: Execute playbooks in deployment order to test real integration scenarios
- **Parallel CI**: Run multiple LXD containers in parallel for different test scenarios

## Monitoring and Review

This decision will be reviewed if:

- Docker container technology evolves to support Docker-in-Docker reliably
- Alternative lightweight virtualization technologies emerge
- Performance requirements change significantly
- Testing requirements become less comprehensive

## References

- [Docker vs LXD Ansible Testing Research](../research/docker-vs-lxd-ansible-testing.md)
- [Ansible Testing Strategy](../research/ansible-testing-strategy.md)
- [Docker-in-Docker Limitations Documentation](https://docs.docker.com/engine/security/rootless/#known-limitations)
- [LXD vs Docker Comparison](https://ubuntu.com/blog/lxd-vs-docker)
