# Clarifying Questions for Config Validation Command

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Core Functionality**

**Question**: What is the minimum viable functionality for this feature?

**Your Answer**:

The `validate` command checks whether a configuration is **intrinsically valid** - meaning it validates the config in isolation, as if it were the first environment being created. It does NOT check conditions that depend on the current application state.

**Three Levels of Validation**:

| Level | Name                           | Description                                                                    | Included? |
| ----- | ------------------------------ | ------------------------------------------------------------------------------ | --------- |
| 1     | **Syntactic**                  | JSON is valid, required fields exist, types are correct                        | ✅ Yes    |
| 2     | **Config-intrinsic semantics** | Cross-field rules within the config itself (e.g., Grafana requires Prometheus) | ✅ Yes    |
| 3     | **State-dependent semantics**  | Rules that depend on current app state (e.g., environment name already exists) | ❌ No     |

**Why exclude state-dependent validation?**

- The same config may be valid in one data directory but invalid in another (due to name conflicts)
- The user is asking "Is this configuration valid?" not "Will `create` succeed right now?"
- State-dependent checks belong to a potential future `create --dry-run` feature

**Examples of config-intrinsic semantic rules** (included):

- If Grafana is enabled, Prometheus must be enabled
- If database type is MySQL, MySQL config section must be present
- Port numbers must be in valid range
- Provider-specific required fields (e.g., Hetzner requires API token)

**Examples of state-dependent rules** (excluded):

- Environment name already exists in data directory
- SSH key file exists on filesystem
- Provider API is reachable

**Context**: Should we validate:

- Only JSON syntax?
- JSON schema compliance?
- Semantic validation (e.g., valid provider types, port ranges)?
- Cross-field validation (e.g., MySQL config only when database type is MySQL)?
- External resource validation (e.g., SSH key file exists)?

### 2. **Command Design**

**Question**: Should this be a new standalone command (e.g., `validate`) or a flag on the existing `create` command (e.g., `create --dry-run`)?

**Your Answer**: **Option A - Standalone `validate` command**

**Rationale**:

A `validate` command and a `--dry-run` flag make different promises to users:

- **`validate`** = "Is this configuration sane?" (static correctness, no side effects, fast, deterministic)
- **`--dry-run`** = "Will this work here?" (exercises real code path, may touch environment read-only)

Since our validation:

- Checks schemas, invariants, and cross-field rules
- Does NOT require touching the outside world
- Should be usable in CI, editors, and automation
- Should NOT overpromise executability

A standalone `validate` command is the right choice. It aligns with DDD boundaries (validation at the domain layer) and keeps the CLI honest.

We may add `create --dry-run` in the future if we want to show a deployment plan that exercises the full execution path, but that's a separate feature.

**Options**:

- **Option A**: New `validate` command - clearer intent, discoverable, follows Unix philosophy ✅ **CHOSEN**
- **Option B**: `create --dry-run` flag - familiar pattern, shows what would happen
- **Option C**: Both - `validate` as primary, `--dry-run` as alias

---

### 3. **Out of Scope**

**Question**: What is explicitly NOT included in this feature?

**Your Answer**:

The `validate` command only checks **config-intrinsic** validity. Anything that depends on external state is out of scope.

| Consideration                                         | Included? | Reason                                        |
| ----------------------------------------------------- | --------- | --------------------------------------------- |
| External resource existence (SSH keys, Docker images) | No        | Depends on filesystem state                   |
| Provider connectivity (LXD socket, Hetzner API)       | No        | Depends on network/external services          |
| Environment name uniqueness                           | No        | Depends on current app state (data directory) |
| JSON Schema compliance                                | No        | Not currently used by `create` command        |

**Key insight**: The same configuration file might be valid for one data directory but cause a name conflict in another. The `validate` command treats the config as if it were the first environment being created - checking only intrinsic validity.

**Why not check environment name conflicts?**

The user is asking "Is this a valid configuration?" not "Will `create` succeed in my current data directory?" A config with an existing environment name is still a _valid_ config - it would work perfectly in a fresh data directory.

**Considerations**:

- Should we validate that external resources exist (SSH keys, Docker images)?
- Should we check provider connectivity (e.g., LXD socket, Hetzner API)?
- Should we validate that the environment name doesn't already exist?

### 4. **User Experience**

**Question**: How should users interact with this feature? What's the expected workflow?

**Your Answer**:

Users invoke the standalone `validate` command with the same `--env-file` argument used by `create`:

```bash
torrust-tracker-deployer validate --env-file envs/my-config.json
```

This provides a familiar, consistent interface across commands.

**Example workflows**:

```bash
# Option A: Standalone command ✅ CHOSEN
torrust-tracker-deployer validate --env-file envs/my-config.json

# Option B: Dry-run flag
torrust-tracker-deployer create --env-file envs/my-config.json --dry-run

# Option C: Both available
```

---

## 🎯 Technical Approach

### 5. **Output Format**

**Question**: What should the output look like for valid vs invalid configurations?

**Your Answer**:

The output should match the `create` command's behavior:

- **Valid config**: Success message indicating the configuration is valid
- **Invalid config**: Error message with details about what failed validation

No special output format (JSON, verbose mode) is required for the initial implementation.

**Options**:

- Simple success/error message ✅
- Detailed validation report with all checks performed
- JSON output for programmatic use
- Verbose mode with all validation steps shown

### 6. **Error Reporting**

**Question**: How detailed should error messages be? Should we report all errors at once or stop at the first error?

**Your Answer**:

Follow the same error reporting behavior as the `create` command. This ensures consistency and avoids surprising users who are familiar with `create`.

**Options**:

- **Fail-fast**: Stop at first error (simpler, faster) - matches current `create` behavior
- **Collect all**: Report all validation errors at once (better UX for fixing multiple issues)

### 7. **Integration Points**

**Question**: Should this share validation logic with the existing `create` command, or be independent?

**Your Answer**:

Shared logic is required to ensure consistency. If validation diverges between `validate` and `create`, users would encounter a confusing experience where `validate` succeeds but `create` fails (or vice versa).

The implementation should extract the validation portion of `CreateCommandHandler` into a reusable function that both commands can call.

**Consideration**: Sharing logic ensures consistency but couples the implementations. Independent logic allows for more thorough validation but risks divergence.

## 📊 Priority and Timeline

### 8. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Your Answer**: **Medium priority**

This feature is valuable but not blocking other work. It improves developer experience and enables safer experimentation with configurations.

**Context**: This is particularly valuable for:

- AI agents that need safe exploration of configurations
- Users learning the system who want to experiment
- CI/CD pipelines that validate configs before deployment

### 9. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**:

No specific deadline, but ideally completed before the first beta release to ensure users have safe configuration exploration from the start.

### 10. **Dependencies**

**Question**: Does this feature depend on other work being completed first?

**Your Answer**:

No direct dependencies. The feature can be implemented using existing validation logic in the `create` command.

**Known considerations**:

- Depends on current validation logic in `create` command
- May benefit from JSON Schema Generation feature if we want schema-based validation

## ✅ Success Criteria

### 11. **Definition of Done**

**Question**: How do we know this feature is complete and working correctly?

**Your Answer**:

The suggested criteria are correct. All items must be completed:

**Suggested criteria**:

- [ ] Command exists and is documented
- [ ] Valid configs return success (exit code 0)
- [ ] Invalid configs return clear error messages (exit code non-zero)
- [ ] No state changes occur during validation
- [ ] Help text explains the command
- [ ] User guide updated

### 12. **Testing Requirements**

**Question**: What level of testing is expected? (Unit | Integration | E2E)

**Your Answer**:

E2E tests following the pattern established in `tests/e2e/commands/create/`. The tests should verify:

- Valid configurations are accepted
- Invalid configurations produce appropriate errors
- No state changes occur (the `data/` directory remains unchanged)

**Suggested**:

- Unit tests for validation logic
- Integration tests for command execution
- E2E test verifying no state changes after validation

## 🤔 Additional Questions

### 13. **AI Agent Considerations**

**Question**: Are there specific requirements for AI agent usage of this feature?

**Your Answer**:

No special requirements beyond normal usage. The standard command interface is sufficient for AI agents.

Future enhancements (JSON output, structured errors) could improve AI agent integration but are not required for the initial implementation.

**Context**: AI agents may:

- Generate configs programmatically and need quick validation
- Want structured (JSON) output for parsing
- Need to validate many config variations quickly

### 14. **Existing Environment Handling**

**Question**: If validating a config for an environment name that already exists, should we warn the user?

**Your Answer**: [To be filled by product owner]

**Options**:

- Ignore existing environments (pure validation only)
- Warn that create would fail due to name conflict
- Error out since create would fail anyway

Warn that create would fail due to name conflict.
