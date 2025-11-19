# Clarifying Questions for Register Existing Instances

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Core Functionality**

**Question**: Should the `register` command support only basic connectivity validation, or should it also validate instance requirements (OS version, disk space, memory, etc.)?

**Your Answer**:

It should only validate basic connectivity validation.

### 2. **Out of Scope**

**Question**: What is explicitly NOT included in this feature? For example:

- Registering multiple instances at once (cluster registration)?
- Auto-detecting instance configuration?
- Migrating existing Docker Compose deployments?
- Registering by cloud provider instance ID?

**Your Answer**:

Everything you mentioned is out of scope.

### 3. **User Experience**

**Question**: Should the register command be interactive (prompting for missing information) or require all parameters via command-line flags? Or support both modes?

**Your Answer**:

No, It should not. It should require all parameters via command-line flags or with a configuration file, like the
create environment command, if they are too many.

### 4. **SSH Key Management**

**Question**: Should the register command:

- Require users to provide existing SSH keys?
- Support generating new SSH keys and installing them on the instance?
- Support both approaches?

**Your Answer**:

- It should require users to provide the same things we require to create an environment, like SSH keys, username, in addition to other parameters that only make sense for existing instances, like IP address. When we provision an instance using the normal create command we get the IP address after provisioning, but in this case, the user must provide it.

## 🎯 Technical Approach

### 5. **State Transition Strategy**

**Question**: The current state machine requires environments to go through `Created` → `Provisioning` → `Provisioned`. Should we:

- Add a direct constructor to create `Environment<Provisioned>` (bypassing earlier states)?
- Add a special `Imported` state that's equivalent to `Provisioned`?
- Use the existing state machine but mark the environment differently (metadata)?

**Your Answer**:

It will only be possible to register an instance that is already provisioned, if there are no other environments with the same name. It should fail f we try to register an environment with the same name as an existing one.

After that, the environment will be in the `Provisioned` state.

We could optionally add metadata to mark it as "registered", but that's not strictly necessary.

### 6. **Validation Depth**

**Question**: How thorough should instance validation be?

**Option A - Minimal**: Only validate SSH connectivity  
**Option B - Basic**: SSH + OS detection  
**Option C - Comprehensive**: SSH + OS + architecture + disk + memory + network  
**Option D - Configurable**: Allow `--validate` flag with different levels

**Your Answer**:

For this first iteration, minimal validation should be enough. Other commands will fail later if the instance does not meet requirements.

### 7. **Error Handling**

**Question**: If validation fails (e.g., wrong OS version, insufficient disk space), should the register command:

- Fail completely and refuse to create the environment?
- Create the environment with warnings and let the user proceed?
- Support `--force` flag to bypass validation?

**Your Answer**:

We should create the environment but inform the user of the validation failures.

Since we only check connectivity the state can be `Provisioned`, because we can have a temporal network issue.
If we add more validation in the future and the instance does not met requirements the environment will be created in the `ProvisionedFailed` state.

### 8. **Environment Metadata**

**Question**: Should registered environments be marked differently from provisioned ones? This could help with:

- Future destroy operations (don't destroy registered infrastructure)
- Status reporting (show "registered" vs "provisioned")
- Troubleshooting and support

**Your Answer**:

Yes, definitely. They should be marked differently. And we should prevent destroying registered instances unless explicitly confirmed by the user. This will help avoid accidental data loss. And it's likely that the destroy command will need different logic for registered instances.

## 📊 Priority and Timeline

### 9. **Priority Level**

**Question**: What is the priority of this feature relative to other roadmap items?

**Options**:

- **High**: Block other work to implement this first
- **Medium**: Implement after current sprint/milestone
- **Low**: Defer until Hetzner provider is complete

**Your Answer**:

High, this will allow to simplify E2E tests before adding the new Hetzner provider. And that would make changing the E2E tests easier.

### 10. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**:

No, there is no target date for completion.

### 11. **Dependencies**

**Question**: Should this feature be implemented:

- Before the Hetzner provider (to establish the import pattern)?
- After the Hetzner provider (to see if patterns emerge)?
- In parallel with Hetzner provider development?

**Your Answer**:

Before the Hetzner provider.

## ✅ Success Criteria

### 12. **Definition of Done**

**Question**: How do we know this feature is complete? Should it include:

- ✅ Command successfully registers instances and creates environment in `Provisioned` state?
- ✅ Registered environments work identically to provisioned ones for all subsequent commands?
- ✅ E2E tests use register command instead of `run_provision_simulation` hack?
- ✅ Documentation covers all registration scenarios?
- ✅ User guide includes examples and troubleshooting?

**Your Answer**:

All of them. And we should do a manual test registering two types of existing instances as well: one VM created with LXD or and one Docker container.

### 13. **Testing Requirements**

**Question**: What level of testing is expected?

**Requirements**:

- Unit tests for `ImportCommandHandler`?
- Integration tests with real Docker containers?
- E2E tests using import with VMs?
- All of the above?

**Your Answer**:

Same level as other commands, but we do not need E2E tests since the feature will be indirectly tested when we replace the `run_provision_simulation.rs` with the register command in existing E2E tests.

### 14. **Documentation Requirements**

**Question**: What documentation needs to be updated or created?

- User guide section on importing existing instances?
- ADR documenting the architectural decision?
- Examples in README?
- API documentation?
- Troubleshooting guide?

**Your Answer**:

- New command docs in docs/user-guide/commands.
- Update other parts of the documentation where there is a list of commands.

## ⚠️ Risk Assessment

### 15. **Known Risks**

**Question**: Key risks include:

- Users importing incompatible instances (wrong OS, missing dependencies)
- SSH connectivity issues difficult to debug
- State management complexity (bypassing normal flow)
- Destroy command accidentally destroying user infrastructure

How should we mitigate these risks?

**Your Answer**:

- Users importing incompatible instances (wrong OS, missing dependencies)

Subsequent commands will fail and inform the user of the issues.

- SSH connectivity issues difficult to debug

We do not need to do anything special here, just inform the user of the connectivity issues.

- State management complexity (bypassing normal flow)

TThe flow should be the same as for normal environments after the initial registration.

- Destroy command accidentally destroying user infrastructure

We will not allow destroy registered instances. In the future we could add a reregister command to destroy the metadata without destroying the instance. But we do not need it for now. The user can simply delete the deployer data.

### 16. **Backward Compatibility**

**Question**: Does this feature need to maintain backward compatibility with anything? Will it affect existing environments or commands?

**Your Answer**:

No, it will not affect existing environments or commands.

### 17. **Alternative Approaches**

**Question**: We considered several alternatives:

**Option 1**: Extend `create` command with `--host` flag  
**Option 2**: New `register` command (recommended)  
**Option 3**: OpenTofu container provider

Are there other approaches we should consider?

**Your Answer**:

Not at this time.

## 💡 E2E Testing Questions

### 18. **Testing Infrastructure Replacement**

**Question**: Should we completely replace `run_provision_simulation.rs` with the register command, or keep both approaches?

**Context**: The simulation hack is currently used only for container-based E2E tests. The register command would be more "realistic" but might be more complex.

**Your Answer**:

The current "run_provision_simulation" function does two things:

1. It creates a Docker container to act as the instance.
2. It creates the internal state to simulate the instance was provisioned (but only the data and build contents).

The environment state is directly created in the main E2E test bootstrap code, like this:

```rust
// Step 2: Create a simulated provisioned environment for type-safe configuration
// In config-only tests, we simulate the provisioned state since we use Docker containers
// instead of actual VM provisioning
let created_env = test_context
    .environment
    .clone()
    .try_into_created()
    .context("Environment must be in Created state for config tests")?;
let provisioned_env = created_env.start_provisioning().provisioned();
```

We will only need to do the first point: create the container. The second point and the setup for the correct environment state will be replaced by the register command.

### 19. **GitHub Actions Compatibility**

**Question**: The main reason for container-based tests is GitHub Actions networking issues with nested LXD VMs. Should we:

- Use register command with containers on GitHub Actions?
- Keep using full LXD VMs locally and containers on CI?
- Try to fix the LXD networking issues instead?

**Your Answer**:

We do not need to do anything special here. We only need to refactor the E2E test binaries. They will be called in the GitHub Actions workflows.

## 🔄 Future Enhancements

### 20. **Command Name** ✅ **RESOLVED**

**Question**: What is the best name for this command?

**Decision**: `register` - chosen based on industry precedent (GitHub/GitLab runners, Consul services, Vault backends)

**Alternatives Considered**:

1. **`import`** - Too generic, overloaded in programming contexts (import modules, import data)
2. **`adopt`** - Unique and semantic, but less familiar in DevOps/infrastructure contexts
3. **`add`** - Too vague, doesn't convey that infrastructure already exists
4. **`attach`** - Implies temporary connection rather than permanent management
5. **`connect`** - Sounds more like testing connectivity than registering for management
6. **`onboard`** - Too corporate/HR-like for CLI tool
7. **`claim`** - Too aggressive, uncommon in infrastructure tools
8. **`enroll`** - Too formal/academic, longer to type

**Rationale for `register`**:

- ✅ Strong industry precedent (GitHub Actions runners, GitLab runners)
- ✅ Bidirectional relationship semantics (server registers WITH deployer)
- ✅ Professional, business-appropriate tone
- ✅ Clear ownership and ongoing management implication
- ✅ Discoverable for users familiar with CI/CD workflows

---

### 21. **Future Provider Support**

**Question**: When we add the Hetzner provider, should users be able to:

- Register existing Hetzner instances by instance ID?
- Register from any cloud provider via IP address?
- Both?

**Your Answer**:

The users will be able to register any existing virtual machine that meets the requirements:

- It uses Ubuntu 24.04
- It has SSH connectivity
- It has the public SSH key installed for access
- It has a public IP address reachable from the deployer
- It has a username with sudo access
- It has a mark for cloud-init completion
- In general, it has all the things that the cloud-init template provides when we create a new instance (see templates/tofu/lxd/cloud-init.yml.tera).

And it should not have:

- Dependencies that are incompatible with the deployer (old Docker versions, old systemd versions, etc.)
- Custom configurations that prevent the deployer from working correctly.
- Etc.

### 22. **Cluster Registration**

**Question**: Should future versions support registering multiple instances as a cluster in a single command?

**Your Answer**:

No, not needed at all.

## 📝 Notes

Use this section for any additional context, clarifications, or decisions made during the Q&A process.

**Last Updated**: November 19, 2025
