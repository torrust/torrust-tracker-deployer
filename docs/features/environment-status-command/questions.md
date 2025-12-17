# Clarifying Questions for Environment Status Command

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Command Name Decision**

**Question**: Do you agree with using `show` as the command name?

**Context**: Based on the specification analysis, `show` was chosen because:

- The command displays stored environment information without verification
- Aligns with common CLI patterns (`kubectl get`, `docker inspect`, `git show`)
- Reserves `status` for future service health/runtime status checks
- Clear separation of concerns: `show` (data), `test` (infrastructure), `status` (future services)

**Decision Made**: Use `show` (documented in specification)

**Your Answer**: [Confirm or provide alternative with rationale]

Yes, I agree with using `show` as the command name.

### 2. **Core Functionality - Minimum Viable Output**

**Question**: What is the minimum information that must be displayed for all environments?

**Proposed Minimum**:

- Environment name
- Current state (Created, Provisioned, Configured, Released, Running, Destroyed)

**Your Answer**: [To be filled by product owner]

Confirmed. The minimum information to be displayed for all environments is the environment name and current state.

### 3. **State-Specific Information**

**Question**: What additional information should be displayed for each state?

**Proposed State-Specific Details**:

- **Created**: Basic environment info only
- **Provisioned**: Instance IP address, SSH port (22 or custom)
- **Configured**: Docker version, Docker Compose version
- **Released**: Deployed application version/tag
- **Running**: Service URLs, tracker ports, API endpoints
- **Destroyed**: Destruction timestamp, cleanup status

**Your Answer**: [To be filled by product owner - adjust or confirm]

In the Provisioned state and beyond, include also the path for the SSH private key used for access.

### 4. **Provider-Specific Information**

**Question**: Should the command display provider-specific information (LXD vs Hetzner)?

**Examples**:

- LXD: Container ID, profile name
- Hetzner: Server ID, datacenter location, server type

**Your Answer**: [To be filled by product owner]

Yes, that would be useful information to include for users to identify their environments more easily.

### 5. **Out of Scope**

**Question**: Do you agree with the scope exclusions?

**Confirmed Out of Scope** (from specification):

- Real-time monitoring (CPU, memory, network usage)
- Historical state transition tracking
- Log viewing or tailing
- Multiple environment listing (separate `list` command)
- Infrastructure verification and smoke tests (use `test` command instead)
- Remote service health checks or connectivity validation
- Watch mode (user confirmed not desired)
- Verbosity levels (handled globally, not per-command)

**Your Answer**: [Confirm or add additional exclusions]

Confirmed. The outlined scope exclusions are appropriate for this command.

## 🎯 Technical Approach

### 6. **Output Format - Initial Implementation**

**Question**: Do you agree with the phased output format approach?

**Decision Made** (from specification):

- **Phase 1-5**: Human-friendly output only (pretty-printed with colors and formatting)
- **Future Enhancement**: JSON output with `--format json` flag

**Rationale**: Deliver value quickly with human-friendly output first, add machine-readable format later based on user needs

**Your Answer**: [Confirm or request JSON format in initial implementation]

Yes, I agree with the phased output format approach.

### 7. **Error Handling**

**Question**: What should happen when environment doesn't exist or is in an unexpected state?

**Proposed Behavior**:

- Environment not found: Clear error message with suggestion to use `list` command
- Invalid state: Display available information and note the unexpected state
- Permission errors: Clear message about file access issues

**Your Answer**: [To be filled by product owner - adjust or confirm]

All proposed behaviors are acceptable. However, there should not be invalid states.

### 8. **Integration with Existing Commands**

**Question**: Should this command integrate with or complement any existing commands?

**Context**: This is a new read-only command that displays information from the environment JSON file and possibly queries remote infrastructure.

**Your Answer**: [To be filled by product owner]

For the time being, it should remain a standalone command and it does not query remote infrastructure.

## 📊 Priority and Timeline

### 9. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Context**: This is task 5.1 in the roadmap, listed under "Add extra console app commands"

**Your Answer**: [To be filled by product owner]

Medium priority.

### 10. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**: [To be filled by product owner]

January 2026 would be a reasonable target date for completion.

### 11. **Dependencies**

**Question**: Does this feature depend on other work being completed first?

**Known Dependencies**:

- Requires environment state persistence (already implemented)
- Requires UserOutput system for formatting (already implemented)
- No blocking dependencies identified

**Your Answer**: [To be filled by product owner - confirm or add additional dependencies]

Confirmed. No additional dependencies identified. We might want to include some data that is not yet stored in the environment JSON, but that can be added later as needed.

## ✅ Success Criteria

### 12. **Definition of Done**

**Question**: How do we know this feature is complete and working correctly?

**Proposed Acceptance Criteria**:

- [ ] Command displays environment name and state for all state types
- [ ] Command shows IP address and SSH port for provisioned environments
- [ ] Command handles non-existent environments with clear error message
- [ ] Command follows existing output formatting patterns (UserOutput)
- [ ] Unit tests cover command logic
- [ ] E2E test validates command execution
- [ ] User documentation updated with command usage and examples

**Your Answer**: [To be filled by product owner - adjust or confirm]

Confirmed. The proposed acceptance criteria are appropriate for defining the completion of this feature.

### 13. **Testing Requirements**

**Question**: What level of testing is expected? (Unit | Integration | E2E)

**Proposed Testing**:

- **Unit Tests**: Command logic, output formatting
- **Integration Tests**: Environment loading, state handling
- **E2E Tests**: Full command execution with real environments

**Your Answer**: [To be filled by product owner - adjust or confirm]

All proposed testing levels are appropriate.

### 14. **Documentation Requirements**

**Question**: What documentation needs to be updated or created?

**Proposed Documentation Updates**:

- `docs/console-commands.md` - Add status/show command documentation
- `docs/user-guide/commands.md` - Add command reference
- `docs/user-guide/commands/show.md` - Detailed command guide
- Update roadmap to mark task 5.1 as complete

**Your Answer**: [To be filled by product owner - adjust or confirm]

Confirmed. The proposed documentation updates are appropriate.

## ⚠️ Risk Assessment

### 15. **Known Risks**

**Question**: What are the potential risks or challenges with this feature?

**Identified Risks**:

- State-specific information may not always be available (e.g., provisioned but no IP stored)
- Remote infrastructure queries may fail (network issues, VM down)
- Performance impact if querying remote systems for real-time status

**Your Answer**: [To be filled by product owner - add additional risks or mitigation strategies]

Correct, but as per the specification, we are not querying remote systems for this command.

### 16. **Backward Compatibility**

**Question**: Confirm that backward compatibility is not required?

**Context**: This is a new command with no backward compatibility concerns. Additionally, the project is in early development with no production users, so extending the domain model (adding `created_at` timestamp, `service_endpoints`) doesn't require migration logic.

**Decision Made** (from specification): No backward compatibility or migration logic needed

**Your Answer**: [Confirm or raise concerns about existing deployments]

Confirmed. No backward compatibility concerns exist for this new command.

### 17. **Alternative Approaches**

**Question**: Do you agree with the simple approach (no remote queries)?

**Decision Made** (from specification): **Simple approach** - Only read from environment JSON (no remote queries)

**Rationale**:

- Fast and reliable (no network dependencies)
- Clear separation of concerns: `show` displays data, `test` verifies infrastructure
- Aligns with read-only information display pattern
- Can be extended later if needed

**Alternatives Rejected**:

- **Rich approach** (query remote infrastructure): Slower, can fail on network issues, duplicates `test` command
- **Hybrid approach** (optional remote queries): Adds complexity, unclear benefit

**Your Answer**: [Confirm or request remote verification capability]

Confirmed. The simple approach of only reading from the environment JSON is appropriate for this command.

## 💡 Additional Questions

### 18. **Remote Status Verification**

**Question**: Confirm that remote verification is out of scope for this command?

**Decision Made** (from specification): **Display stored state only** - No remote verification

**Rationale**:

- Infrastructure verification is the responsibility of the existing `test` command
- Keeps `show` command fast and reliable (no network calls)
- Clear command separation: `show` (data), `test` (infrastructure), `status` (future service health)
- Users can run `test` command separately if they need verification

**Your Answer**: [Confirm or explain why remote verification is needed in show command]

Yes, it is confirmed that remote verification is out of scope for this command.

### 19. **Future JSON Output Format**

**Question**: If we add JSON output in the future, what should the structure look like?

**Example**:

```json
{
  "name": "my-environment",
  "state": "Provisioned",
  "provider": "lxd",
  "infrastructure": {
    "ip_address": "10.140.190.14",
    "ssh_port": 22
  },
  "created_at": "2025-12-16T10:30:00Z",
  "updated_at": "2025-12-16T11:45:00Z"
}
```

**Your Answer**: [To be filled by product owner - confirm or propose alternative]

Yes, that's a good structure for future JSON output. We could add more info in the provider section as needed. However, for now, we should focus on the human-friendly output.

### 20. **Verbosity Levels**

**Question**: Confirm that per-command verbosity levels are not needed?

**Decision Made** (from specification): **No per-command verbosity levels**

**Rationale**:

- Verbosity is handled globally in the application (applies to all commands)
- Per-command verbosity would create inconsistent user experience
- The command will display all relevant information for the current state
- Global verbosity flags (if they exist) will control detailed output across all commands

**Your Answer**: [Confirm or explain specific need for per-command verbosity]

It's not that it's not needed, but rather that it is out of scope for this command at this time.
There is a draft feature specification for verbosity levels that can be implemented later in docs/features/progress-reporting-in-application-layer/README.md. The idea is to increase the level of detail shown based on global verbosity settings, from only in the presentation layer to also including more detailed steps in the application layer.

### 21. **E2E Testing Strategy for Different Environment States**

**Question**: Which testing strategy do you prefer for validating the show command with different environment states?

**Options** (to be decided in Phase 5):

1. **Strategy 1 - Existing Workflow Integration**: Call show command in existing E2E workflow tests (`e2e_complete_workflow_tests`, `e2e_deployment_workflow_tests`, `e2e_infrastructure_lifecycle_tests`) after each state transition
   - **Pros**: Most realistic, tests actual state transitions, validates end-to-end flow
   - **Cons**: Requires running through full workflows (slower, more complex)

2. **Strategy 2 - Mocked State**: Mock internal state in dedicated E2E test before running show command
   - **Pros**: More isolated, faster, easier to test edge cases
   - **Cons**: Requires mocking infrastructure (may not catch integration issues)

3. **Strategy 3 - Unit Tests Only**: Test different states only via unit tests for message building logic (decoupled from internal state)
   - **Pros**: Fastest, best for testing message formatting logic
   - **Cons**: Doesn't validate end-to-end state reading

**Recommended Approach** (from specification): Combination - Strategy 3 for message formatting unit tests + Strategy 1 for realistic E2E validation

**Your Answer**: [Select preferred strategy or confirm recommended approach]

The recommended approach of combining Strategy 3 for unit tests and Strategy 1 for E2E validation is appropriate. Anyway, we will see during implementation if we need to adjust the approach based on complexity.

## 📝 Notes

Add any additional context, constraints, or considerations here.
