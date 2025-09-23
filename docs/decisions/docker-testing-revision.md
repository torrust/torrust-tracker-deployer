# Decision Record: Revisiting Docker Container Testing Strategy

**Date**: September 23, 2025  
**Status**: Superseded  
**Decision Makers**: Infrastructure Team  
**Supersedes**: [Docker Testing Rejection](./docker-testing-rejection.md)  
**Related**: [E2E Tests Split](../refactors/split-e2e-tests-provision-vs-configuration.md), [Docker Phase Architecture](./docker-phase-architecture.md)

## Context Change

The original decision to reject Docker containers for Ansible testing (September 2, 2025) has been **superseded** by the implementation of the **E2E Test Split Architecture** (September 2025).

### What Changed

The **fundamental assumptions** in the original decision no longer apply:

1. **Testing Scope Separation**: We no longer test provisioning + configuration in a single test
2. **Cloud-Init Irrelevance**: Configuration tests don't need cloud-init (that's tested in provision phase)
3. **Docker-in-Docker Feasibility**: Research proved Docker installation inside Docker containers works
4. **Phase-Specific Requirements**: Different deployment phases have different testing needs

## New Architecture Overview

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

## Decision Revision

### Original Decision: ❌ **Rejected Docker for All Testing**

**Reason**: Could not test complete infrastructure + configuration together

### New Decision: ✅ **Docker for Configuration Testing**

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

## Implementation Results

### Configuration Testing with Docker Containers

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

### Performance Comparison

| Phase         | Technology       | Setup Time     | Use Case                                    |
| ------------- | ---------------- | -------------- | ------------------------------------------- |
| **Provision** | LXD VMs          | ~17-30 seconds | Infrastructure creation, cloud-init testing |
| **Configure** | Docker Container | ~2-3 seconds   | Software installation, system configuration |
| **Release**   | Docker Container | ~2-3 seconds   | Application deployment testing              |
| **Run**       | Docker Container | ~2-3 seconds   | Service validation, monitoring setup        |

## Benefits of Revised Approach

### ✅ **Best of Both Worlds**

- **LXD VMs**: Complete infrastructure testing where needed
- **Docker Containers**: Fast configuration testing where sufficient

### ✅ **Faster Feedback Loops**

- **Provision Testing**: Still comprehensive but isolated to infrastructure concerns
- **Configuration Testing**: 10x faster feedback for application deployment issues

### ✅ **Better CI/CD Performance**

- **Parallel Execution**: Provision and configuration tests can run in parallel
- **Resource Efficiency**: Configuration tests use minimal resources
- **GitHub Actions Reliability**: Docker containers avoid LXD networking issues

### ✅ **Clearer Test Failures**

- **Infrastructure Issues**: Isolated to provision tests
- **Configuration Issues**: Isolated to configuration tests
- **Easier Debugging**: Clear separation of concerns

## What This Supersedes

The original [Docker Testing Rejection](./docker-testing-rejection.md) decision is now **partially superseded**:

### Still Valid ❌

- **Complete E2E Testing**: Docker cannot replace LXD for full infrastructure testing
- **Cloud-Init Testing**: Docker cannot test cloud-init scenarios

### No Longer Valid ✅

- **Configuration Testing**: Docker is now suitable for configuration phase testing
- **Docker-in-Docker**: Proven to work for configuration scenarios
- **Performance Trade-offs**: Split approach provides better overall performance

## Future Architecture

This revision enables a **phase-specific container architecture**:

```text
docker/
├── provisioned-instance/     # ✅ Current - Ready for configuration
├── configured-instance/      # 🔄 Future - Post-Docker installation
├── released-instance/        # 🔄 Future - Post-app deployment
└── running-instance/         # 🔄 Future - Services validated
```

Each phase can be built FROM the previous phase, creating an efficient testing pipeline.

## References

- [Original Docker Testing Rejection](./docker-testing-rejection.md)
- [E2E Tests Split Plan](../refactors/split-e2e-tests-provision-vs-configuration.md)
- [Docker Phase Architecture](./docker-phase-architecture.md)
- [Docker Configuration Testing Research](../research/e2e-docker-config-testing.md)
- [Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support)
