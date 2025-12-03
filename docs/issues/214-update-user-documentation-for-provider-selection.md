# Update User Documentation for Provider Selection

**Issue**: [#214](https://github.com/torrust/torrust-tracker-deployer/issues/214)
**Parent Epic**: #205 - Epic: Add Hetzner Provider Support
**Related**: [#206](https://github.com/torrust/torrust-tracker-deployer/issues/206), [#207](https://github.com/torrust/torrust-tracker-deployer/issues/207), [#208](https://github.com/torrust/torrust-tracker-deployer/issues/208), [#212](https://github.com/torrust/torrust-tracker-deployer/issues/212), Task 5 (E2E Tests updates)

## Overview

This task updates all user-facing documentation to reflect the new provider selection feature and fixes several issues in the existing documentation. Users need clear guidance on how to configure their environment for different infrastructure providers (LXD for local development, Hetzner for production deployments).

This is Task 6 in Phase 1 ("Make LXD Explicit") of the Hetzner Provider Support epic. It is the final task before Phase 2 (Add Hetzner Provider) begins.

## Goals

- [ ] Update all documentation files that reference provider configuration
- [ ] Fix existing issues in README.md (emoji, E2E commands, structure, Next Steps)
- [ ] Create new provider documentation in user guide
- [ ] Ensure documentation follows project conventions and is user-friendly
- [ ] Add troubleshooting guidance for common provider-related issues

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (Documentation only)
**Module Path**: `docs/`, `README.md`
**Pattern**: User Documentation

### Documentation Standards

- [ ] Follow Markdown linting rules (`.markdownlint.json`)
- [ ] Use proper heading hierarchy
- [ ] Include code examples that can be copy-pasted
- [ ] Avoid GitHub Markdown pitfalls (see [docs/contributing/github-markdown-pitfalls.md](../contributing/github-markdown-pitfalls.md))

### Anti-Patterns to Avoid

- ❌ Vague instructions without concrete examples
- ❌ Missing prerequisites or assumptions
- ❌ Outdated code snippets that don't match current implementation
- ❌ Hash-number patterns for enumeration (creates unintended GitHub links)

## Specifications

### Documentation Files to Update

#### Main Documentation Updates

| File Path                               | Action | Description                                    |
| --------------------------------------- | ------ | ---------------------------------------------- |
| `README.md`                             | Update | Fix emoji, E2E commands, structure, Next Steps |
| `docs/user-guide/quick-start.md`        | Update | Add provider selection to quick start workflow |
| `docs/user-guide/commands.md`           | Update | Document provider requirements for commands    |
| `docs/user-guide/commands/create.md`    | Update | Add provider config to create command docs     |
| `docs/user-guide/commands/provision.md` | Update | Document provider-specific provisioning        |

#### New Provider Documentation (Create)

| File Path                              | Action | Description                              |
| -------------------------------------- | ------ | ---------------------------------------- |
| `docs/user-guide/providers/README.md`  | Create | Provider overview and selection guide    |
| `docs/user-guide/providers/lxd.md`     | Create | LXD provider setup and configuration     |
| `docs/user-guide/providers/hetzner.md` | Create | Hetzner provider setup and configuration |

### README.md Fixes Required

#### A) Fix E2E Test Commands

The README.md contains incorrect E2E test command references. Update to match actual binary names:

**Current (incorrect)**:

```bash
cargo run --bin e2e-provision-tests
```

**Correct**:

```bash
cargo run --bin e2e-provision-and-destroy-tests
```

#### B) Fix Roadmap Emoji

Line 337 shows a broken emoji character: `�` instead of the roadmap emoji.

**Fix**: Replace with proper emoji `🗺️` or `📋`

#### C) Update Repository Structure

The repository structure section needs to reflect current directory layout, especially:

- New `docs/user-guide/providers/` directory
- Any changes to template directories for multi-provider support
- Updated `templates/tofu/` structure (if changed)

#### D) Update Next Steps Section

The "Next Steps" section mentions Hetzner as upcoming. Update to reflect:

- Hetzner provider is now implemented (Phase 1 complete)
- What remains in the roadmap
- Current production-ready status

### Provider Documentation Content Requirements

#### `docs/user-guide/providers/README.md`

Overview document covering:

- What are providers and why they matter
- Comparison table: LXD vs Hetzner
- How to choose the right provider
- Links to provider-specific documentation

#### `docs/user-guide/providers/lxd.md`

LXD provider documentation:

- Prerequisites (LXD installation, profile setup)
- Configuration format with examples
- Common issues and troubleshooting
- Best practices for local development

#### `docs/user-guide/providers/hetzner.md`

Hetzner provider documentation:

- Prerequisites (Hetzner account, API token)
- Configuration format with examples
- Server types and locations reference
- Cost considerations
- Common issues and troubleshooting

### Configuration Examples

**LXD Provider Example**:

```json
{
  "name": "my-local-env",
  "instance_name": "torrust-tracker-vm",
  "provider_config": {
    "provider": "lxd",
    "profile_name": "torrust-profile"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub"
  },
  "ssh_port": 22
}
```

**Hetzner Provider Example**:

```json
{
  "name": "my-production-env",
  "instance_name": "torrust-tracker-prod",
  "provider_config": {
    "provider": "hetzner",
    "api_token": "your-hetzner-api-token-here",
    "server_type": "cx22",
    "location": "nbg1"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub"
  },
  "ssh_port": 22
}
```

## Implementation Plan

### Phase 1: Fix README.md Issues (estimated 1 hour)

- [ ] Task 1.1: Fix broken Roadmap emoji (line 337)
- [ ] Task 1.2: Fix E2E test command references (`e2e-provision-tests` → `e2e-provision-and-destroy-tests`)
- [ ] Task 1.3: Update Repository Structure section to reflect current layout
- [ ] Task 1.4: Update Next Steps section to reflect Hetzner implementation status
- [ ] Task 1.5: Add provider selection mention in Quick Start section

### Phase 2: Create Provider Documentation (estimated 1-2 hours)

- [ ] Task 2.1: Create `docs/user-guide/providers/` directory
- [ ] Task 2.2: Create `docs/user-guide/providers/README.md` with provider overview
- [ ] Task 2.3: Create `docs/user-guide/providers/lxd.md` with LXD documentation
- [ ] Task 2.4: Create `docs/user-guide/providers/hetzner.md` with Hetzner documentation

### Phase 3: Update Existing User Guide Documents (estimated 1 hour)

- [ ] Task 3.1: Update `docs/user-guide/quick-start.md` with provider selection
- [ ] Task 3.2: Update `docs/user-guide/commands.md` with provider requirements
- [ ] Task 3.3: Update `docs/user-guide/commands/create.md` with provider config
- [ ] Task 3.4: Update `docs/user-guide/commands/provision.md` with provider details

### Phase 4: Add Troubleshooting Content (estimated 30 minutes)

- [ ] Task 4.1: Add provider-specific troubleshooting to each provider doc
- [ ] Task 4.2: Document common errors and their solutions
- [ ] Task 4.3: Add links between related documentation

### Phase 5: Review and Validate (estimated 30 minutes)

- [ ] Task 5.1: Run markdown linter on all updated/created files
- [ ] Task 5.2: Verify all code examples are valid JSON
- [ ] Task 5.3: Check all internal links work correctly
- [ ] Task 5.4: Review for clarity and completeness from a new user's perspective

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All markdown files pass linting

**README.md Fixes**:

- [ ] Roadmap emoji displays correctly (not `�`)
- [ ] E2E test commands are correct (`e2e-provision-and-destroy-tests`)
- [ ] Repository structure is accurate
- [ ] Next Steps reflects current Hetzner implementation status

**Provider Documentation**:

- [ ] `docs/user-guide/providers/README.md` exists with provider overview
- [ ] `docs/user-guide/providers/lxd.md` exists with complete LXD documentation
- [ ] `docs/user-guide/providers/hetzner.md` exists with complete Hetzner documentation
- [ ] All provider docs include configuration examples
- [ ] All provider docs include troubleshooting sections

**User Guide Updates**:

- [ ] Quick start references provider selection
- [ ] Commands documentation mentions provider requirements
- [ ] Create command docs include provider_config examples
- [ ] Provision command docs explain provider-specific behavior

**Documentation Quality**:

- [ ] Documentation is clear and understandable for new users
- [ ] Prerequisites are explicitly stated
- [ ] Code examples can be copy-pasted and work
- [ ] Links to related documentation work correctly
- [ ] Terminology is consistent across all documentation

## Related Documentation

- [Hetzner Provider Feature Specification](../features/hetzner-provider-support/specification.md)
- [Hetzner Provider Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [Epic: Add Hetzner Provider Support](./205-epic-hetzner-provider-support.md)
- [Development Principles](../development-principles.md) - Actionability and User Friendliness
- [GitHub Markdown Pitfalls](../contributing/github-markdown-pitfalls.md)

## Notes

- **Dependency on Tasks 1-5**: This documentation task should be completed after the code changes are finalized to ensure documentation matches implementation.
- **Placeholder tokens**: Example Hetzner configurations should use obvious placeholder text like `"your-hetzner-api-token-here"` to prevent accidental token exposure.
- **README.md visibility**: The README.md is the first thing users see - fixing issues here has high impact.
- **Documentation structure**: Creating the `providers/` directory establishes a pattern for adding future providers (AWS, GCP, etc.).

---

**Created**: December 3, 2025
**Status**: Ready for Implementation
