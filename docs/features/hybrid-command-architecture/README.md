# Hybrid Command Architecture

## Overview

This feature implements a hybrid command architecture that provides both low-level (plumbing) and high-level (porcelain) commands for deployment operations.

## Status

**Current**: Planning and Specification  
**Implementation**: Plumbing commands first, porcelain commands later

## Problem Statement

Users need different levels of control over the deployment process:

- **Beginners** want simple, guided deployment with minimal commands
- **Advanced users** need precise control over individual deployment steps
- **CI/CD systems** require reliable, granular commands for automation
- **Development workflows** benefit from quick, intelligent deployment commands

A single command interface cannot optimally serve all these use cases.

## Solution

Implement a **hybrid architecture** with two command levels:

### Plumbing Commands (Low-Level)

Individual commands for precise control:

- `torrust-tracker-deployer create <env>`
- `torrust-tracker-deployer provision <env>`
- `torrust-tracker-deployer configure <env>`
- `torrust-tracker-deployer release <env>`
- `torrust-tracker-deployer run <env>`

### Porcelain Commands (High-Level)

Orchestration commands built on plumbing commands:

- `torrust-tracker-deployer deploy <env>` - Smart deployment from current state

## Architecture Decisions

### Plumbing First Implementation

**Rationale**: Plumbing commands provide the foundation. Porcelain commands are built on top of stable plumbing operations.

**Implementation Order**:

1. **Phase 1**: Implement all plumbing commands
2. **Phase 2**: Build porcelain commands using plumbing commands
3. **Phase 3**: Optimize and enhance user experience

### Limited Porcelain Scope

**Scope**: Porcelain commands only automate the core deployment workflow (`provision` → `configure` → `release` → `run`).

**Exclusions**: Management commands (`create`, `list`, `check`, `status`, `destroy`) remain individual operations because:

- They serve different purposes than deployment
- They require different user interaction patterns
- They don't benefit from orchestration

### State-Aware Orchestration

The `deploy` command intelligently determines next steps:

- **From `created`**: `provision` → `configure` → `release` → `run`
- **From `provisioned`**: `configure` → `release` → `run`
- **From `configured`**: `release` → `run`
- **From `released`**: `run`
- **From `running`**: No-op (already complete)

## User Experience Benefits

### For Different User Types

**Beginners**:

```bash
torrust-tracker-deployer create myenv
torrust-tracker-deployer deploy myenv    # One command to completion
```

**Advanced Users**:

```bash
torrust-tracker-deployer create myenv
torrust-tracker-deployer provision myenv
# Custom configuration changes
torrust-tracker-deployer configure myenv
torrust-tracker-deployer release myenv
torrust-tracker-deployer run myenv
```

**CI/CD Pipelines**:

```bash
# Precise control, clear failure points
torrust-tracker-deployer provision $ENV_NAME
torrust-tracker-deployer configure $ENV_NAME
torrust-tracker-deployer release $ENV_NAME
```

**Development Workflows**:

```bash
# Quick iteration
torrust-tracker-deployer deploy dev-env     # Smart deployment
```

## Technical Implementation

### Command Composition

Porcelain commands internally call plumbing commands:

```rust
// Simplified example
impl DeployCommand {
    pub fn execute(&self, env: Environment) -> Result<()> {
        let state = env.current_state()?;

        match state {
            EnvironmentState::Created => {
                self.provision_cmd.execute(&env)?;
                self.configure_cmd.execute(&env)?;
                self.release_cmd.execute(&env)?;
                self.run_cmd.execute(&env)?;
            },
            EnvironmentState::Provisioned => {
                self.configure_cmd.execute(&env)?;
                self.release_cmd.execute(&env)?;
                self.run_cmd.execute(&env)?;
            },
            // ... other states
        }
        Ok(())
    }
}
```

### Error Handling Strategy

- **Plumbing commands**: Fail fast with specific error codes
- **Porcelain commands**: Provide context about which step failed and how to continue

### Progress Reporting

Porcelain commands show unified progress across multiple steps:

```text
[1/4] Provisioning infrastructure...
✓ LXD container created

[2/4] Configuring system...
✓ Docker installed
✓ Network configured

[3/4] Releasing application...
✓ Application deployed

[4/4] Starting services...
✓ Torrust Tracker running at http://10.140.190.14:7070
```

## Future Enhancements

### Additional Porcelain Commands

Potential future additions:

- `torrust-tracker-deployer redeploy <env>` - Release and restart without reprovisioning
- `torrust-tracker-deployer reset <env>` - Return to configured state
- `torrust-tracker-deployer upgrade <env>` - Update application while preserving data

### Configuration Integration

Porcelain commands could support configuration profiles:

```bash
torrust-tracker-deployer deploy prod-env --config production.toml
```

## References

- [Console Commands Documentation](../../console-commands.md) - Complete command reference
- [UX Design Research](../../research/UX/ux-design-discussion.md) - Original research that led to this design
- Git's porcelain/plumbing architecture - Inspiration for this approach

## Implementation Status

- [ ] **Phase 1**: Plumbing Commands Implementation

  - [ ] `create` command
  - [x] `provision` command (partially implemented in E2E tests)
  - [x] `configure` command (partially implemented in E2E tests)
  - [ ] `release` command
  - [ ] `run` command
  - [x] `destroy` command (being implemented)

- [ ] **Phase 2**: Porcelain Commands Implementation

  - [ ] `deploy` command orchestration logic
  - [ ] State detection and step selection
  - [ ] Unified progress reporting
  - [ ] Enhanced error messages with continuation guidance

- [ ] **Phase 3**: Integration and Polish
  - [ ] Command composition optimization
  - [ ] User experience testing
  - [ ] Documentation updates
  - [ ] CI/CD integration examples
