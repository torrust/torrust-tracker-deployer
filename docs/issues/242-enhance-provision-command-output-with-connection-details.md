# Enhance Provision Command Output with SSH Connection Details

**Issue**: #242
**Parent Epic**: N/A (standalone improvement)
**Related**: User Experience, CLI Output

## Overview

Improve the `provision` command output to display all essential information needed to connect to the newly provisioned instance. Currently, users must manually inspect the environment state file (`data/{env-name}/environment.json`) or remember configuration details to determine how to connect to the provisioned instance.

This enhancement will display the instance IP address, SSH port, and SSH private key path directly in the command output, enabling users to immediately connect to their provisioned infrastructure without additional commands.

## Goals

- [ ] Display instance IP address in provision command output
- [ ] Display SSH service port in provision command output
- [ ] Display absolute path to SSH private key in provision command output
- [ ] Format output to be clear and copy-paste friendly for SSH connection
- [ ] Maintain consistency with existing output formatting patterns

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/cli/commands/provision.rs`
**Pattern**: CLI Command Output Enhancement

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Use `UserOutput` for all user-facing output (see [docs/contributing/output-handling.md](../docs/contributing/output-handling.md))
- [ ] Extract connection details from environment runtime outputs and user inputs
- [ ] Format output according to presentation layer conventions

### Architectural Constraints

- [ ] Use `UserOutput` methods exclusively (never `println!`, `eprintln!`)
- [ ] Extract data from domain model (Environment state) without duplicating logic
- [ ] Maintain separation between command orchestration and output formatting
- [ ] Ensure output is testable through UserOutput inspection

### Anti-Patterns to Avoid

- ❌ Direct use of `println!`, `eprintln!`, or stdout/stderr
- ❌ Embedding formatting logic in domain or application layers
- ❌ Hardcoding SSH port instead of reading from configuration
- ❌ Displaying relative paths instead of absolute paths for SSH keys

## Specifications

### Current Output

When running `provision {environment}`, users currently see:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: provision-output-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
  ✓ Infrastructure provisioned (took 28.8s)
✅ Environment 'provision-output-test' provisioned successfully
```

### Proposed Enhanced Output

After provisioning completes, display connection details:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: provision-output-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
  ✓ Infrastructure provisioned (took 28.8s)
✅ Environment 'provision-output-test' provisioned successfully

Instance Connection Details:
  IP Address:        10.140.190.171
  SSH Port:          22
  SSH Private Key:   /home/user/path/to/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/user/path/to/fixtures/testing_rsa torrust@10.140.190.171 -p 22
```

### Data Sources

The information is available in the environment state after provisioning:

```json
{
  "Provisioned": {
    "context": {
      "user_inputs": {
        "ssh_credentials": {
          "ssh_priv_key_path": "/path/to/private/key",
          "ssh_username": "torrust"
        },
        "ssh_port": 22
      },
      "runtime_outputs": {
        "instance_ip": "10.140.190.171"
      }
    }
  }
}
```

### Implementation Location

The output enhancement should be added in the provision command handler:

- File: `src/presentation/cli/commands/provision.rs`
- Method: After successful provisioning in the command execution
- Access data from: `Environment<Provisioned>` state
- Output via: `UserOutput` methods

### Formatting Guidelines

1. **Section Header**: "Instance Connection Details:"
2. **Field Alignment**: Use consistent spacing for readability
3. **Copy-Paste Format**: Provide complete SSH command ready to copy
4. **Absolute Paths**: Always display absolute paths for SSH keys
5. **Port Display**: Always show port even if default (22)

## Implementation Plan

### Phase 1: Data Extraction (estimated: 30 minutes)

- [ ] Task 1.1: Locate provision command completion handler in `src/presentation/cli/commands/provision.rs`
- [ ] Task 1.2: Extract instance IP from `runtime_outputs.instance_ip`
- [ ] Task 1.3: Extract SSH port from `user_inputs.ssh_port`
- [ ] Task 1.4: Extract SSH private key path from `user_inputs.ssh_credentials.ssh_priv_key_path`
- [ ] Task 1.5: Extract SSH username from `user_inputs.ssh_credentials.ssh_username`

### Phase 2: Output Formatting (estimated: 30 minutes)

- [ ] Task 2.1: Create formatted output message with connection details
- [ ] Task 2.2: Ensure absolute path resolution for SSH private key
- [ ] Task 2.3: Format SSH command with all required parameters
- [ ] Task 2.4: Add output after successful provision confirmation using `UserOutput`
- [ ] Task 2.5: Test output formatting with different path lengths and configurations

### Phase 3: Edge Cases and Testing (estimated: 1 hour)

- [ ] Task 3.1: Test with custom SSH port (non-22)
- [ ] Task 3.2: Test with relative vs absolute SSH key paths
- [ ] Task 3.3: Test with long absolute paths
- [ ] Task 3.4: Verify output in E2E tests captures new information
- [ ] Task 3.5: Update any relevant documentation

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Provision command output includes instance IP address
- [ ] Provision command output includes SSH port (even if default 22)
- [ ] Provision command output includes absolute path to SSH private key
- [ ] Provision command output includes SSH username
- [ ] Output provides ready-to-copy SSH connection command
- [ ] Output uses `UserOutput` methods (no direct `println!`/`eprintln!`)
- [ ] Output formatting is consistent with existing patterns
- [ ] Works correctly with custom SSH ports (non-22)
- [ ] Works correctly with both relative and absolute SSH key paths
- [ ] E2E tests continue to pass with new output format

## Related Documentation

- [docs/contributing/output-handling.md](../contributing/output-handling.md) - Output handling conventions
- [docs/console-commands.md](../console-commands.md) - Command documentation
- [docs/user-guide/commands/provision.md](../user-guide/commands/provision.md) - Provision command guide
- [docs/codebase-architecture.md](../codebase-architecture.md) - Architecture overview

## Notes

### Design Decisions

1. **Always Show Port**: Even though 22 is the default SSH port, we explicitly display it to:

   - Avoid user confusion about what port to use
   - Support custom port configurations transparently
   - Make the SSH command complete and unambiguous

2. **Absolute Paths**: SSH key paths are converted to absolute paths to:

   - Ensure the command works regardless of user's current directory
   - Avoid path resolution ambiguity
   - Match how SSH tools expect paths

3. **Complete SSH Command**: Providing the full SSH command:
   - Reduces friction for users to connect to their instance
   - Serves as documentation for less experienced users
   - Eliminates the need for a separate "how to connect" command

### Future Enhancements

- Consider adding output to other commands (configure, release, run) showing current connection status
- Could add a dedicated `connection-info` or `ssh-info` command for retrieving connection details later
- Might integrate with SSH config file generation for easier connection management

### Alternative Approaches Considered

1. **Create separate `show` command**: Rejected because it adds extra step for users
2. **Output only IP**: Rejected because users still need port and key path
3. **Minimal output**: Rejected because the goal is to provide complete, actionable information
