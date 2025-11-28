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

**Current Phase**: ✅ COMPLETED

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Answer clarifying questions (all 22 questions answered)
4. ✅ Update specification based on answers
5. ✅ Create GitHub issue ([#203](https://github.com/torrust/torrust-tracker-deployer/issues/203))
6. ✅ Implementation complete ([PR #204](https://github.com/torrust/torrust-tracker-deployer/pull/204))

**Implementation Summary**:

- Domain layer: State transition `Environment<Created>::register(instance_ip)` → `Environment<Provisioned>`
- Application layer: `RegisterCommandHandler` with SSH connectivity validation
- Presentation layer: CLI command with `--instance-ip` argument
- E2E tests: Migrated from `run_provision_simulation` hack to proper `register` command
- Documentation: User guide, ADR, and updated console commands

## 🎯 Quick Summary

This feature adds a new `torrust-tracker-deployer register` command that allows users to register pre-existing infrastructure (VMs, physical servers, or containers) directly into the deployer workflow. This solves two key needs:

**Key Points**:

- **Problem**: Users cannot use the deployer with their own already-provisioned infrastructure, and E2E testing requires hacky workarounds for container-based tests
- **Solution**: New `register` command that transitions environments from `Created` to `Provisioned` state, validates SSH connectivity (minimal v1), and enables continuing with `configure`, `release`, etc.
- **Command Name**: Chosen `register` based on industry precedent (GitHub/GitLab runners, Consul services, etc.)
- **Priority**: HIGH - Implemented before Hetzner provider to simplify E2E tests
- **Validation**: Minimal for v1 (SSH connectivity only), advanced validation deferred to v2
- **Safety**: Registered instances marked with metadata, destroy command preserves registered infrastructure
- **Status**: ✅ COMPLETED

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
- Replaced `run_provision_simulation` hack with proper `register` command
- Better test isolation

## 🔗 Related Documentation

- [User Guide: Register Command](../../user-guide/commands/register.md) - How to use the command
- [ADR: Register Existing Instances](../../decisions/register-existing-instances.md) - Design decisions
- [Console Commands](../../console-commands.md) - Command reference
- [Development Principles](../../development-principles.md)
- [Roadmap](../../roadmap.md) - Section 2: Hetzner provider support
- [VM Providers](../../vm-providers.md) - LXD vs container comparison
- [State Machine](../../../src/domain/environment/state/mod.rs) - Environment states

---

**Created**: November 19, 2025  
**Last Updated**: November 28, 2025 (Implementation Complete)
