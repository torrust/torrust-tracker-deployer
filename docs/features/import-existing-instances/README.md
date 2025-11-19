# Register Existing Instances

Allow users to register already-provisioned instances with the Torrust Tracker Deployer, bypassing the provision phase and enabling deployment to infrastructure managed outside the deployer.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and problem statement
- Feature goals
- Proposed solution (new `register` command)
- Implementation details
- Command naming analysis and alternatives considered
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- Scope and requirements
- Technical approach
- Priority and timeline
- Success criteria
- Risk assessment

## 📋 Status

**Current Phase**: ✅ Questions Answered - Ready for Implementation

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Answer clarifying questions (all 22 questions answered)
4. ✅ Update specification based on answers
5. ⏳ Create GitHub issue
6. ⏳ Begin implementation

**Next Steps**:

1. Create GitHub issue linking to this feature
2. Add to roadmap as child issue under appropriate EPIC
3. Begin implementation after issue creation

## 🎯 Quick Summary

This feature adds a new `torrust-tracker-deployer register` command that allows users to register pre-existing infrastructure (VMs, physical servers, or containers) directly into the deployer workflow. This solves two key needs:

**Key Points**:

- **Problem**: Users cannot use the deployer with their own already-provisioned infrastructure, and E2E testing requires hacky workarounds for container-based tests
- **Solution**: New `register` command that creates environments directly in the `Provisioned` state, validates SSH connectivity (minimal v1), and enables continuing with `configure`, `release`, etc.
- **Command Name**: Chosen `register` based on industry precedent (GitHub/GitLab runners, Consul services, etc.)
- **Priority**: HIGH - Must be implemented before Hetzner provider to simplify E2E tests
- **Validation**: Minimal for v1 (SSH connectivity only), defer advanced validation to v2
- **Safety**: Registered instances marked with metadata, future destroy command will require confirmation
- **Status**: ✅ Questions Answered - Ready for GitHub issue creation and implementation

## 🔄 Dual Purpose

This feature serves both **production users** and **development/testing** needs:

### For End Users

- Register spare/existing servers instead of provisioning new ones
- Deploy with unsupported cloud providers
- Custom infrastructure configurations
- Cost optimization through infrastructure reuse

### For Development/Testing

- Faster E2E tests with Docker containers (5-10s vs 30-60s)
- GitHub Actions compatibility (workaround for nested VM issues)
- Replace `run_provision_simulation` hack with proper `register` command
- Better test isolation

## 🔗 Related Documentation

- [Development Principles](../../development-principles.md)
- [Roadmap](../../roadmap.md) - Section 2: Hetzner provider support
- [VM Providers](../../vm-providers.md) - LXD vs container comparison
- [State Machine](../../../src/domain/environment/state/mod.rs) - Environment states
- Current workaround: [run_provision_simulation.rs](../../../src/testing/e2e/tasks/container/run_provision_simulation.rs)

---

**Created**: November 19, 2025  
**Last Updated**: November 19, 2025 (Questions Answered)
