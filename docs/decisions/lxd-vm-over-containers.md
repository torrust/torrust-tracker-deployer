# Decision Record: LXD Virtual Machines over Containers for Production Alignment

**Date**: September 10, 2025  
**Status**: Accepted  
**Decision Makers**: Infrastructure Team  
**Related Documentation**: [LXD Configuration Analysis](../tofu-lxd-configuration.md)

## Context and Problem Statement

After successfully implementing LXD containers for testing and deployment automation, we needed to decide between using LXD **containers** versus **virtual machines** for our primary infrastructure testing and deployment workflows.

Both options are supported by LXD and functionally work with our OpenTofu and Ansible automation. However, we needed to align our testing infrastructure with future production cloud deployments on providers like Hetzner, AWS, and DigitalOcean.

The core question was: **Should we use LXD containers or virtual machines as our primary instance type for deployment testing and automation?**

## Decision Drivers

- **Production Alignment**: Match behavior of target cloud VM environments
- **Future Compatibility**: Ensure deployment scripts work identically in production
- **Security Model**: Provide appropriate isolation for testing workloads
- **Performance Requirements**: Acceptable deployment testing speed
- **Resource Efficiency**: Balance isolation with resource consumption
- **Debugging Experience**: Enable realistic troubleshooting scenarios

## Considered Options

### Option A: LXD Containers

**Approach**: Continue using LXD containers as the primary instance type

**Pros**:

- **Faster boot time**: ~2-5 seconds startup
- **Lower resource usage**: Can run with <1GB RAM
- **GitHub Actions friendly**: Suitable for resource-constrained shared runners
- **Faster CI/CD cycles**: Minimal overhead for integration testing
- **Shared kernel efficiency**: Better resource utilization for parallel testing

**Cons**:

- **Namespace isolation only**: Shared kernel with host system
- **Different from production**: Cloud providers use full VMs, not containers
- **Limited cloud-init**: Some cloud-init features don't work in containers
- **Network interface naming**: Uses predictable names (`eth0`) unlike cloud VMs
- **Security model mismatch**: Different isolation than production environments
- **Debugging disparity**: Issues may not reproduce identically in production VMs

### Option B: LXD Virtual Machines (Selected)

**Approach**: Use LXD virtual machines as the primary instance type

**Pros**:

- **Production parity**: Identical behavior to cloud VMs (Hetzner, AWS, DigitalOcean)
- **Full kernel isolation**: True virtualization matching cloud security models
- **Complete cloud-init support**: All features work as in production
- **Realistic networking**: Hardware-like interface names (`enp5s0`) matching cloud VMs
- **Better debugging**: Issues reproduce identically in production
- **Future-proof architecture**: Direct compatibility with cloud migration
- **Surprising performance**: 37% faster than containers for full deployment workflows (52s vs 85s)

**Cons**:

- **Higher resource usage**: Requires 2GB+ RAM per instance
- **Longer boot time**: ~20-30 seconds for proper kernel initialization
- **GitHub Actions overhead**: May require custom runners for resource-intensive testing

## Decision

We choose **Option B: LXD Virtual Machines**

## Rationale

### Primary Reason: Production Alignment

The most compelling factor is **production alignment**. All target cloud providers (Hetzner, AWS, DigitalOcean) provide virtual machines, not containers. Using VMs ensures:

1. **Identical behavior**: Deployment scripts, networking, and system behavior match production exactly
2. **Bug prevention**: Issues are discovered during testing, not in production
3. **Reduced debugging**: No need to troubleshoot environment-specific differences

### Performance Advantage

Contrary to expectations, virtual machines actually perform **37% better** than containers for complete deployment workflows:

- **Container E2E tests**: ~85 seconds
- **Virtual Machine E2E tests**: ~52 seconds

This improvement comes from:

- More predictable boot sequences
- Better cloud-init integration
- Fewer networking conflicts
- More robust systemd environment

### Strategic Future-Proofing

Virtual machines provide better strategic alignment:

- **Cloud migration**: Scripts work identically when moving to cloud providers
- **Security compliance**: Hardware-level isolation matches enterprise requirements
- **Operational consistency**: Same troubleshooting approaches across all environments

## Implementation Details

### Configuration Changes

Updated `templates/tofu/lxd/main.tf`:

```hcl
resource "lxd_instance" "torrust_vm" {
  name      = var.container_name
  image     = var.image
  type      = "virtual-machine"  # Changed from "container"
  profiles  = [lxd_profile.torrust_profile.name]

  config = {
    "boot.autostart"      = "true"
    "security.secureboot" = "false"  # VM-specific setting
  }

  wait_for_network = true  # Ensure proper network initialization
}
```

### Resource Allocation

Enhanced profile with VM-appropriate resources:

```hcl
resource "lxd_profile" "torrust_profile" {
  config = {
    "limits.memory"  = "2GB"    # VM minimum
    "limits.cpu"     = "2"      # Dedicated cores
    "user.user-data" = file("${path.module}/cloud-init.yml")
  }
}
```

### Code Changes

Fixed LXD JSON parser to handle VM network interfaces:

- **Problem**: Parser was hardcoded for container interface names (`eth0`)
- **Solution**: Dynamic interface discovery supporting VM names (`enp5s0`, etc.)

## Alternative Use Cases for Containers

While virtual machines are the **primary choice**, containers remain valuable for:

- **Fast CI/CD testing**: When startup speed > production fidelity
- **GitHub Actions**: Resource-constrained shared runner environments
- **Integration testing**: Quick validation of deployment logic
- **Development iterations**: Rapid prototyping of infrastructure changes

## Consequences

### Positive

- **Production confidence**: Deployments work identically across all environments
- **Better performance**: 37% faster complete deployment testing
- **Reduced debugging**: Issues caught during testing, not production
- **Future compatibility**: Ready for any cloud provider migration
- **Enhanced security**: Hardware-level isolation for sensitive workloads

### Negative

- **Higher resource usage**: 2GB+ RAM requirement may limit parallel testing
- **Longer boot time**: 20-30s startup may impact quick iteration cycles
- **GitHub Actions constraints**: May require custom runners for resource-intensive workflows

### Mitigation Strategies

- **Hybrid approach**: Use containers for quick unit-style testing, VMs for integration testing
- **Resource management**: Optimize VM resource allocation for specific test scenarios
- **Custom runners**: Consider dedicated GitHub runners for resource-intensive VM testing

## Monitoring and Review

This decision should be reviewed when:

- **Cloud provider changes**: Migration to new cloud infrastructure
- **Resource constraints**: GitHub Actions limitations become prohibitive
- **Performance requirements**: Need for faster testing cycles becomes critical
- **Technology changes**: LXD or cloud provider capabilities evolve significantly

## Related Documents

- [LXD Configuration Analysis](../tofu-lxd-configuration.md) - Complete technical comparison
- [LXD over Multipass Decision](lxd-over-multipass.md) - Platform selection rationale
- [Docker Testing Rejection](docker-testing-rejection.md) - Container testing evaluation
