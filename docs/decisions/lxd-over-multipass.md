# Decision Record: Choosing LXD over Multipass for VM-like Testing

**Date**: September 3, 2025  
**Status**: Accepted  
**Decision Makers**: Infrastructure Team  
**Related Documentation**: [VM Providers Comparison](../vm-providers.md)

## Context and Problem Statement

We needed to find an alternative to KVM/Libvirt virtualization for our testing environments. The primary constraint was that KVM/Libvirt cannot be executed in shared GitHub runners, which is critical for our CI/CD pipeline.

We wanted to avoid the complexity and cost of setting up custom bare-metal runners, so we evaluated lightweight alternatives that could run in GitHub's shared infrastructure while still providing VM-like capabilities for testing our Ansible playbooks and infrastructure automation.

The core question was: **Should we use LXD containers or Multipass VMs as our primary virtualization solution for testing in CI/CD environments?**

## Decision Drivers

- **GitHub Actions Compatibility**: Must run reliably in shared GitHub runners
- **CI/CD Performance**: Fast startup times for efficient testing cycles
- **Official Support**: Need documented and supported solutions
- **Resource Efficiency**: Lower resource consumption for parallel testing
- **Long-term Stability**: Solution should be maintained and supported
- **Cost Effectiveness**: Avoid expensive custom runner infrastructure

## Considered Options

### Option A: Multipass VMs

**Approach**: Use Multipass to create lightweight Ubuntu VMs for testing

**Pros**:

- Full VM isolation with separate kernels
- True virtualization environment
- Simple installation and setup
- Good local development experience
- Complete hardware-level isolation

**Cons**:

- **No official GitHub Actions support**: While it appears to work, there's no documentation in the official GitHub docs
- Higher resource usage due to full virtualization
- Slower boot times (30-60 seconds)
- Requires nested virtualization support
- Uncertain long-term compatibility with GitHub runners

**Testing Results**:
We implemented and tested Multipass with the workflow `.github/workflows/test-multipass-provision.yml` and found it worked in practice, but the lack of official documentation raised concerns about sustainability.

### Option B: LXD Containers

**Approach**: Use LXD system containers as VM-like testing environments

**Pros**:

- **Official GitHub Actions support**: Documented and guaranteed compatibility
- Faster boot times (5-10 seconds)
- Lower resource usage (shared kernel)
- Better CI/CD performance characteristics
- Efficient storage management with snapshots
- Strong container ecosystem integration

**Cons**:

- Process-level isolation (not hardware-level)
- Shared kernel limitations
- More complex initial setup (LXD daemon)
- Container paradigm may be unfamiliar to some team members

## Decision

**We choose LXD containers over Multipass VMs** for the following reasons:

### 1. **Official Support Guarantee**

LXD has official support from the GitHub Actions team with documented compatibility, while Multipass support is undocumented and potentially unreliable. We opened issue [#12933](https://github.com/actions/runner-images/issues/12933) on the GitHub Actions runner-images repository to get official clarification about Multipass support, but cannot wait for an uncertain response.

### 2. **Performance Benefits**

LXD provides superior performance characteristics for CI/CD:

- 5-10 second boot times vs 30-60 seconds for Multipass
- Lower memory and CPU overhead
- Faster test execution cycles

### 3. **Risk Mitigation**

Using an officially supported solution reduces the risk of:

- Breaking changes in future GitHub runner updates
- Lack of support channels when issues arise
- Uncertainty about long-term compatibility

### 4. **Sufficient Capabilities**

LXD containers meet all our testing requirements:

- Cloud-init support for initialization testing
- Full systemd support for service testing
- Docker support for containerized applications
- Network configuration testing capabilities

## Consequences

### Positive

- **Reliable CI/CD**: Official GitHub Actions support ensures stable testing pipeline
- **Faster Feedback**: Quicker boot times enable faster development cycles
- **Cost Effective**: No need for custom runners or expensive infrastructure
- **Resource Efficient**: Lower resource usage allows more parallel testing
- **Future-Proof**: Official support provides long-term stability

### Negative

- **Learning Curve**: Team needs to learn LXD container concepts
- **Setup Complexity**: Initial LXD daemon configuration is more involved
- **Isolation Limitations**: Process-level isolation instead of hardware-level

### Neutral

- **Documentation**: Need to maintain LXD-specific documentation and examples
- **Tooling**: Development scripts and automation focused on LXD

## Validation

This decision is validated by:

1. **Working Implementation**: LXD-based testing environment is operational
2. **Performance Metrics**: Measured boot time and resource usage improvements
3. **Team Consensus**: All team members agree on the technical merits
4. **Comparison Analysis**: Detailed feature comparison documented in [VM Providers Comparison](../vm-providers.md)

## Related Decisions

- [Rejecting Docker Containers for Ansible Testing](docker-testing-rejection.md) - Documents why full VM-like environments are needed
- Future decision may revisit Multipass if official GitHub Actions support is confirmed

## References

- [VM Providers Comparison Documentation](../vm-providers.md)
- [GitHub Actions Runner Images Issue #12933](https://github.com/actions/runner-images/issues/12933)
- LXD Official Documentation
- GitHub Actions Official Documentation
