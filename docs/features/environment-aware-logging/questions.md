# Clarifying Questions for Environment-Aware Logging Feature

## 🤔 Questions for Implementation

Please answer these questions by editing this file directly. Add your answers after each question.

---

### 1. **Current State Recognition - Option 3 Already Exists**

Commands already implement Option 3 using `#[instrument]` with environment fields:

```rust
#[instrument(
    name = "provision_command",
    skip_all,
    fields(
        command_type = "provision",
        environment = %environment.name()
    )
)]
```

This means all logs within command execution are already in spans with environment context!

**Questions**:

- Is the current span-based approach working well enough?
- Is the issue primarily about **visibility** (how the environment is displayed)?
- Or is it about **organization** (file-based separation for future UI)?
- Should we focus on improving the formatter to show environment more prominently?
- Or should we move to Option 2 (separate files) for the UI migration?

**Your Answer**:

- Is the current span-based approach working well enough?

I guess it is working well enough, but it could be improved for visibility. IN some logs is clear which environment they belong to, but in others it is not that obvious.

Clear:

````text
```text
  2025-10-08T09:35:40.731158Z  INFO torrust_tracker_deploy::application::steps::software::docker: Installing Docker via Ansible, step: "install_docker", action: "install_docker", note: "We skip the update-apt-cache playbook in E2E tests to avoid CI network issues"
    at src/application/steps/software/docker.rs:62
    in torrust_tracker_deploy::application::steps::software::docker::install_docker with step_type: "software", component: "docker", method: "ansible"
    in torrust_tracker_deploy::application::commands::configure::configure_command with command_type: "configure", environment: e2e-full
````

Not clear:

```text
  2025-10-08T09:36:02.850501Z  WARN torrust_tracker_deploy::shared::ssh::client: SSH warning detected, operation: "ssh_warning", host_ip: 10.140.190.62, Warning: Permanently added '10.140.190.62' (ED25519) to the list of known hosts.
    at src/shared/ssh/client.rs:156
    in torrust_tracker_deploy::infrastructure::remote_actions::docker_compose::docker_compose_validation with action_type: "validation", component: "docker_compose", server_ip: 10.140.190.62
    in torrust_tracker_deploy::application::steps::validation::docker_compose::validate_docker_compose with step_type: "validation", component: "docker_compose"
    in torrust_tracker_deploy::application::commands::test::test_command with command_type: "test"
```

I've notice that the TestCommand does not have the environment in the span instrumentation, so maybe we should add it there too.

Some logs are not included in a command span, for example:

```text
  2025-10-08T09:36:06.021164Z  INFO torrust_tracker_deploy::e2e::tasks::virtual_machine::cleanup_infrastructure: Test environment cleaned up successfully, operation: "cleanup", status: "success"
    at src/e2e/tasks/virtual_machine/cleanup_infrastructure.rs:66
```

- Is the issue primarily about **visibility** (how the environment is displayed)?

Yes, I think it is primarily about visibility. Maybe we should review all the logs and see if we can add the environment field in some of them.

- Or is it about **organization** (file-based separation for future UI)?

No, that is not a priority for me right now. It's not decided yet. We might end up needing it, but for now, visibility is more important. It's not clear yet if it's convenient to have separate logs files per environment.

- Should we focus on improving the formatter to show environment more prominently?

Maybe, not changing the formatter but analyzing why some logs do not have the environment field in the span context. Maybe just adding it to the ones that makes sense it will be enough.

- Or should we move to Option 2 (separate files) for the UI migration?

No, not yet.

### 2. **Severity of the Problem**

You mentioned "users are not likely to run deployment for two environments at the same time."

**Questions**:

- Is this primarily a "nice-to-have" for better debugging, or is it causing actual problems?
- Have you experienced specific issues that would be solved by this feature?
- Should we prioritize this over other features in the backlog?

**Your Answer**:

- Is this primarily a "nice-to-have" for better debugging, or is it causing actual problems?

TThe primary goal was to make it easier to debug when multiple environments are being deployed. It's not a common scenario, but it can happen, especially in CI environments.

- Have you experienced specific issues that would be solved by this feature?

No, I haven't encountered any specific issues that would be directly addressed by this feature.

- Should we prioritize this over other features in the backlog?

No, if it's a big effort, we can prioritize other features first. I would implement if it's only adding some fields to the spans that are missing it and to other logs that make sense to have it.

### 3. **stdout/stderr Migration Timeline**

You noted that once a UI is implemented, logs will need to move to files anyway.

**Questions**:

- Is UI development planned in the near term (next 3-6 months)?
- Should we implement Option 2 (file-based logs) now to avoid doing the work twice?
- Or is maintaining stdout/stderr important for current development workflow?

**Your Answer**:

- Is UI development planned in the near term (next 3-6 months)?

Yes.

- Should we implement Option 2 (file-based logs) now to avoid doing the work twice?

Only, if we decide that having separate log files per environment is the best approach. But that is something I do not want to decide now. So maybe just improve the visibility of the environment in the current logs is the best option now.

- Or is maintaining stdout/stderr important for current development workflow?

It's easier for development because you do not need to open the logs file. However, we can implement other strategies to make it easier to access the logs files, like a command that tails the logs of a specific environment.

### 4. **Cross-Environment Debugging**

**Questions**:

- Do you foresee needing to correlate logs across multiple environments?
- For example, debugging a shared infrastructure issue affecting multiple environments?
- If yes, would separate log files (Option 2) make this harder?

**Your Answer**:

- Do you foresee needing to correlate logs across multiple environments?

Not actually, but it could be useful in some scenarios.

- For example, debugging a shared infrastructure issue affecting multiple environments?

I do not see an example now.

- If yes, would separate log files (Option 2) make this harder?

Yes, because we would not have a single place to look for all the logs. But we could implement a command that aggregates the logs from multiple environments if needed.

### 5. **Span Implementation Review (Already Done!)**

**Current Implementation**: Commands already create spans in their `execute()` methods using `#[instrument]`.

**Questions**:

- Is the current span location (in command `execute()` methods) appropriate?
- Should we extend spans to other areas (e.g., wrap earlier in the command dispatch)?
- Are there any commands missing environment spans?
- Is the span hierarchy working as expected for all use cases?

**Your Answer**:

- Is the current span location (in command `execute()` methods) appropriate?

Yes, it is appropriate.

- Should we extend spans to other areas (e.g., wrap earlier in the command dispatch)?

It could be an option to wrap earlier in the command dispatch, but we do not have production code yet,
so the code above the command is only for testing. I would wait until we have production code to decide if we need to change it.

- Are there any commands missing environment spans?

No, all commands have environment spans. However, as mentioned before, the TestCommand does not have the environment in the span instrumentation, so maybe we should add it there too.

- Is the span hierarchy working as expected for all use cases?

Yes, it is working as expected.

### 6. **Environment Identification Strategy**

**Questions**:

- Should the environment name come from:
  - **A**: CLI arguments (explicit `--environment` flag)
  - **B**: Environment object (from loaded state)
  - **C**: Both (CLI during creation, state afterward)
- How should we handle commands that don't have an environment yet (e.g., `create`)?
  - **A**: Use a default/placeholder like "none" or "pending"
  - **B**: Don't include environment field at all
  - **C**: Use the environment name being created

**Your Answer**:

- Should the environment name come from:
  - **A**: CLI arguments (explicit `--environment` flag)
  - **B**: Environment object (from loaded state)
  - **C**: Both (CLI during creation, state afterward)

Option C: Flag during creation and also while running commands that need it.

- How should we handle commands that don't have an environment yet (e.g., `create`)?
  - **A**: Use a default/placeholder like "none" or "pending"
  - **B**: Don't include environment field at all
  - **C**: Use the environment name being created

I guess option C: Use the environment name being created.

### 7. **Log File Organization (Option 2)**

If we choose Option 2 (separate log files):

**Questions**:

- Preferred log file structure:
  - **A**: `logs/{environment}/app.log` (simple)
  - **B**: `logs/{environment}/{date}/app.log` (daily rotation)
  - **C**: `logs/{environment}/app-{timestamp}.log` (session-based)
- Should logs be rotated automatically?
- Should there be a size limit per log file?
- Should old logs be automatically archived/compressed?

**Your Answer**:

- Preferred log file structure:
  - **A**: `logs/{environment}/app.log` (simple)
  - **B**: `logs/{environment}/{date}/app.log` (daily rotation)
  - **C**: `logs/{environment}/app-{timestamp}.log` (session-based)

Option A: for now, simple is better. We can rotate logs later if they grow too big.

- Should logs be rotated automatically?

Not now. I expect the logs to be small for now.

- Should there be a size limit per log file?

Yes, but if we implement log rotation, it will be handled by the rotation mechanism.

- Should old logs be automatically archived/compressed?

Yes, but if we implement log rotation, it will be handled by the rotation mechanism.

### 8. **Backward Compatibility**

**Questions**:

- Are there any existing tools or scripts that parse the current log output?
- Would changing log format or location break anything?
- Do we need to maintain backward compatibility with the current format?

**Your Answer**:

- Are there any existing tools or scripts that parse the current log output?

No, we do not have any tools or scripts that parse the current log output. But the logs might be parsed by AI assistant tools in the future that are using the deployer to deploy the tracker.

- Would changing log format or location break anything?

No.

- Do we need to maintain backward compatibility with the current format?

No. WE are still in early development, so we can change the format if needed.

### 9. **Testing Requirements**

**Questions**:

- Should we add automated tests to verify environment context appears in logs?
- How thorough should the testing be (unit tests, integration tests, E2E tests)?
- Should we add tests to ensure infrastructure layers remain environment-agnostic?

**Your Answer**:

- Should we add automated tests to verify environment context appears in logs?

Yes, we should add automated tests to verify that the environment context appears in the logs where it is expected.

- How thorough should the testing be (unit tests, integration tests, E2E tests)?

Unit tests are ideal. However it might not be easy considering all the infrastructure layers that are involved in the logging. So maybe integration tests are more appropriate.

- Should we add tests to ensure infrastructure layers remain environment-agnostic?

No, I think we should focus in this case in "what to expect" instead of "what not to do". So we should test that the application layers include the environment context in the logs where it is expected.

### 10. **Performance Implications**

**Questions**:

- Are there any performance concerns with any of the three options?
- Should we benchmark log performance with the new environment context?
- Is log volume expected to increase significantly?

**Your Answer**:

- Are there any performance concerns with any of the three options?

No at all. Performance is not a concern in this case and in general in this application.

- Should we benchmark log performance with the new environment context?

No.

- Is log volume expected to increase significantly?

No.

### 11. **Abstraction Layer Boundaries**

You mentioned that infrastructure adapters shouldn't know about environments.

**Questions**:

- Are there any edge cases where infrastructure layers might need environment context?
- Should we enforce this through linting or code review?
- How do we prevent environment context from leaking into lower layers?

**Your Answer**:

- Are there any edge cases where infrastructure layers might need environment context?

Yes, there might be cases. But those layers should not be aware of the environment. It could be possible to include the environment if we define a generic mechanism to include metadata in the operations, but I would not implement that for now.

- Should we enforce this through linting or code review?

No, I think code review is enough.

- How do we prevent environment context from leaking into lower layers?

With code review for now.

### 12. **E2E Testing Impact**

**Questions**:

- How would each option affect E2E test output and debugging?
- Do E2E tests need to verify environment identification?
- Should E2E tests produce combined logs or separate logs per environment?

**Your Answer**:

- How would each option affect E2E test output and debugging?

Each option would affect the E2E test output and debugging by potentially altering the log format and the information included in the logs. This could make it easier to identify issues based on the environment context.

- Do E2E tests need to verify environment identification?

No, I think E2E tests are high-level tests that do not need to verify the environment identification in the logs. They should focus on verifying the overall functionality of the application.

- Should E2E tests produce combined logs or separate logs per environment?

For now, combined logs are fine. If we decide to implement separate logs per environment in the future, we can update the E2E tests accordingly.

### 13. **Formatter Configuration**

Given that span context exists but might not be visible enough:

**Questions**:

- Should we develop a custom tracing formatter that shows `[environment]` prefix on each line?
- Would a formatter that extracts environment from span context solve the visibility issue?
- Should we provide multiple formatter options (compact vs verbose)?
- Do you prefer environment shown as prefix (`[e2e-full] log message`) or suffix (`log message [e2e-full]`)?

**Your Answer**:

- Should we develop a custom tracing formatter that shows `[environment]` prefix on each line? Yes, this could improve visibility.

No, not for now. I think just adding the environment field to the spans that are missing it and to other logs that make sense to have it will be enough.

- Would a formatter that extracts environment from span context solve the visibility issue?

The current formatter already extracts the environment from the span context, so it is not a problem of the formatter. The problem is that some logs do not have the environment in the span context.

- Should we provide multiple formatter options (compact vs verbose)?

They already exist.

- Do you prefer environment shown as prefix (`[e2e-full] log message`) or suffix (`log message [e2e-full]`)?

I prefer the prefix, but I do not want to implement that for now.

### 14. **Additional Considerations**

**Questions**:

- Is there anything else we should consider that wasn't covered in the specification?
- Are there other systems or tools we should look at for inspiration?
- Any other concerns or requirements?

**Your Answer**:

- Is there anything else we should consider that wasn't covered in the specification?

No. I think we have covered everything.

- Are there other systems or tools we should look at for inspiration?

Yes, we should look at other logging frameworks and tools that support environment-aware logging for inspiration.

- Any other concerns or requirements?

No. I think we have covered everything.

## 📋 Summary

### Recommended Approach

**Selected Option**: Hybrid - Improve Visibility (Combination of existing Option 3 + targeted improvements)

**Reasoning**:

- Span-based infrastructure (Option 3) already works well
- Some commands (like `TestCommand`) are missing environment in their span instrumentation
- Solution: Fill the gaps rather than redesign the system
- Focus on visibility improvements where they matter most

### Implementation Priority

**Priority Level**: Medium

**Timeline**: Incremental - can be done as part of regular development

**Blocking Issues**: None

### Key Requirements

1. Add environment field to command spans that are missing it (e.g., `TestCommand`)
2. Add environment field to 10-20 strategic logs at application/domain layers
3. Keep infrastructure layers environment-agnostic (no changes to adapters)
4. Document when to include environment field in logging guide

### Out of Scope

1. Custom formatter development (current formatter works fine)
2. Separate log files per environment (deferred until UI implementation)
3. E2E test utility logs (not production code)
4. Changing command dispatch or entry points (wait for production code)
