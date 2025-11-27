# Register Existing Instances - Key Decisions Summary

**Created**: November 19, 2025  
**Last Updated**: November 27, 2025  
**Status**: ✅ All Questions Answered

This document summarizes the key decisions made during the Q&A process for the "Register Existing Instances" feature.

## 🎯 Scope Decisions

### ✅ In Scope for v1

- **Register as alternative to provision** - Transitions existing `Created` environment to `Provisioned`
- **Basic SSH connectivity validation** - Minimal validation only, using credentials from environment
- **Simple CLI** - Only requires `--instance-ip` parameter (SSH credentials already in environment)
- **Metadata tracking** - Mark registered instances differently
- **Graceful validation handling** - Transition environment even if validation fails, warn user
- **Destroy protection** - Prevent accidental destruction of registered instances (requires confirmation)

### ❌ Out of Scope

- **Cluster registration** - Register multiple instances at once
- **Auto-discovery** - Automatically detect instance configuration
- **Docker Compose migration** - Converting existing Docker deployments
- **Cloud provider ID registration** - Hetzner instance ID support (future)
- **Advanced validation** - OS version, architecture, disk space, memory checks (deferred to v2)
- **Automated SSH setup** - Key generation or installation
- **Dependency detection** - Automatic detection and installation of missing dependencies
- **Duplicating create environment** - SSH credentials provided via `create environment` first

## 🔧 Technical Decisions

### Register as Alternative to Provision (NEW)

**Key Insight**: The `create environment` command creates the environment _concept_ (SSH credentials, name, configuration) - not the actual infrastructure. The `register` command is simply an alternative to `provision` for getting to the `Provisioned` state.

**Rationale**:

- No duplication of SSH credential handling
- Only needs instance IP (the only runtime output from provisioning)
- Consistent with domain model (`RuntimeOutputs` only stores `instance_ip`)
- Clean separation: "creating the concept" vs "materializing infrastructure"

### Command Name: `register`

**Rationale**: Strong industry precedent (GitHub/GitLab runners, Consul, Vault)

**Alternatives Rejected**:

- `import` - Too generic, overloaded in programming contexts
- `adopt` - Less familiar in DevOps contexts
- `add`, `attach`, `connect`, `onboard`, `claim`, `enroll` - Various issues (see specification)

### State Management

- **State transition**: `Created` → `Provisioned` (same as `provision` command)
- **Sets runtime output**: `runtime_outputs.instance_ip` populated with provided IP
- **Metadata flag** - Add `"registered": "true"` metadata to environment
- **Future state** - Use `ProvisionFailed` for validation failures in v2

### Validation Strategy

- **v1**: SSH connectivity only (using credentials already in environment)
- **v2**: OS version, architecture, disk space, memory, dependencies
- **Error Handling**: Transition environment even if validation fails, inform user

### Destroy Command Behavior

- **Current**: Registered instances marked with metadata
- **Future**: Destroy requires explicit confirmation for registered instances
- **Re-register**: Future command to remove metadata without destroying instance

## 📊 Priority and Timeline

- **Priority**: **HIGH**
- **Rationale**: Simplifies E2E tests before adding Hetzner provider
- **Timeline**: No fixed deadline
- **Dependencies**: Must be implemented BEFORE Hetzner provider

## ✅ Success Criteria

### Functional Requirements

1. Command requires environment to exist in `Created` state
2. Command only requires `--instance-ip` parameter
3. Environment transitions to `Provisioned` state with `runtime_outputs.instance_ip` set
4. Registered environments marked with `"registered": "true"` metadata
5. SSH connectivity validated using environment's SSH credentials
6. Manual tests: Successfully register LXD VM and Docker container

### Testing Requirements

- Unit tests for `RegisterCommandHandler`
- Unit tests for state transition `Created` → `Provisioned`
- Integration tests for SSH validation and repository integration
- NO dedicated E2E tests (indirectly tested through existing E2E tests)
- Replace `run_provision_simulation.rs` with register command

### Documentation Requirements

- `docs/user-guide/commands/register.md` - User-facing documentation
- `docs/console-commands.md` - Update with register command
- Update all command lists throughout documentation
- Prerequisites documented (create environment first)
- Instance requirements documented (Ubuntu 24.04, SSH, cloud-init, etc.)

## 🔄 E2E Testing Architecture

### Current Approach (`run_provision_simulation.rs`)

1. Creates Docker container to act as instance
2. Creates internal state to simulate provisioned instance
3. Environment state created directly in E2E bootstrap code

### Future Approach (with register command)

1. Create environment first (provides SSH credentials)
2. Create container (same as before)
3. Use register command to transition environment to `Provisioned` state
4. Eliminates `run_provision_simulation.rs` entirely

## 📋 Instance Requirements

### REQUIRED

- Ubuntu 24.04 LTS (exact version)
- SSH connectivity with credentials from environment
- Public SSH key installed for access
- Public IP address reachable from deployer
- Username with sudo access
- Cloud-init completion mark
- All dependencies from cloud-init template (`templates/tofu/lxd/cloud-init.yml.tera`)

### MUST NOT HAVE

- Incompatible dependencies (old Docker, old systemd, etc.)
- Custom configurations preventing deployer operation
- Security restrictions blocking required operations

## ⚠️ Risk Mitigation

| Risk                                       | Mitigation Strategy                                      |
| ------------------------------------------ | -------------------------------------------------------- |
| Users importing incompatible instances     | Subsequent commands fail with clear error messages       |
| SSH connectivity issues                    | Inform user of connectivity issues, create env anyway    |
| State management complexity                | Flow same as normal environments after registration      |
| Destroy command destroying user infra      | Prevent destroying registered instances without confirm  |
| Missing dependencies                       | Users responsible, subsequent commands fail with details |
| Custom configurations blocking deployer    | Users responsible, clear errors guide troubleshooting    |
| Validation failures creating unusable envs | Use ProvisionFailed state in future (v2)                 |

## 🔮 Future Enhancements

1. **Advanced Validation (v2)**: OS version, architecture, disk space, memory, dependencies
2. **Cloud Provider Registration**: Register Hetzner instances by instance ID
3. **Re-register Command**: Remove metadata without destroying instance
4. **ProvisionFailed State**: Better state representation for validation issues
5. **Cluster Support**: Not needed per product owner decision

## 📝 Key Q&A Insights

### On Validation Philosophy

> "Only basic connectivity validation (minimal)" - **Q1**
>
> "Subsequent commands will fail and inform the user of the issues" - **Q15**

### On Error Handling

> "Create environment even with validation failures, inform user" - **Q7**
>
> "We do not need to do anything special here, just inform the user of the connectivity issues" - **Q15**

### On Destroy Safety

> "They should be marked differently. And we should prevent destroying registered instances unless explicitly confirmed by the user." - **Q8**

### On Testing Strategy

> "we do not need E2E tests since the feature will be indirectly tested when we replace the `run_provision_simulation.rs` with the register command in existing E2E tests" - **Q13**

### On Priority

> "High, this will allow to simplify E2E tests before adding the new Hetzner provider. And that would make changing the E2E tests easier." - **Q9**

### On Future Provider Support

> "The users will be able to register any existing virtual machine that meets the requirements" - **Q21**

## 📎 References

- Full specification: [`specification.md`](./specification.md)
- All questions and answers: [`questions.md`](./questions.md)
- Feature overview: [`README.md`](./README.md)
