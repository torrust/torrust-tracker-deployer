# Known Issues and Expected Behaviors

This document describes known issues, expected errors, and normal behaviors that may appear alarming but are actually under control in the Torrust Tracker Deploy project.

## 🔍 Overview

During E2E tests and normal operation, some console output may contain warnings or informational messages that are normal behavior. Since we updated our logging strategy (as of September 2025), the executor module now logs both stdout and stderr at debug level when commands succeed, which should reduce confusion about what constitutes an actual error.

## 🟡 Expected "Errors" in E2E Tests

### SSH Host Key Warnings

**Appearance**: Debug messages in logs during SSH operations (as of September 2025 update)

**Example Output**:

```text
2025-09-30T11:32:57.584329Z DEBUG torrust_tracker_deploy::shared::executor: stderr: Warning: Permanently added '10.140.190.144' (ED25519) to the list of known hosts.
```

**Why This Happens**:

- SSH writes host key information to stderr (not stdout)
- As of September 2025, our executor logs both stdout and stderr at debug level when commands succeed
- The SSH options we use (`-o StrictHostKeyChecking=no`) generate these warnings by design

**Is This Actually An Error?**: **NO** - This is expected behavior and now properly logged as debug information

**Root Cause**: SSH is informing us that it's adding the host key to the known_hosts file, which is normal security behavior.

**Commands Affected**:

- `ssh -i <key> -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 22 user@host <command>`
- Any SSH-based operations during provisioning and deployment

**Current Status**: **Resolved** - As of September 2025, these messages are now logged at debug level instead of error level

**Future Resolution**: We plan to improve error detection by:

1. Checking command exit codes instead of just stderr presence
2. Filtering known harmless warnings from error logs
3. Using appropriate log levels (WARN vs ERROR)

### Docker Compose Operations Through SSH

**Appearance**: Red error messages during Docker Compose validation

**Example Output**:

```text
2025-09-29T11:32:57.376789Z ERROR torrust_tracker_deploy::shared::executor: Command produced stderr output, operation: "command_execution", command: ssh -i /tmp/.tmpSZhY0Y/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 22 torrust@10.140.190.144 cd /tmp && docker compose -f test-docker-compose.yml config, stderr: Warning: Permanently added '10.140.190.144' (ED25519) to the list of known hosts.
```

**Why This Happens**: Same as SSH warnings above - the SSH connection itself produces stderr output even when the Docker Compose command succeeds.

**Is This Actually An Error?**: **NO** - This is expected SSH behavior

**Commands Affected**:

- `docker compose version` (via SSH)
- `docker compose config` (via SSH)
- `docker compose up/down` (via SSH)
- Any Docker operations executed remotely

**Current Status**: **Resolved** - As of September 2025, these messages are now logged at debug level instead of error level

## 🔧 How to Distinguish Real Errors

### For Developers

When investigating logs, look for these patterns to distinguish real errors from expected warnings:

#### Real Errors (Investigate These)

- Exit codes != 0
- Connection timeouts
- Permission denied (when not expected)
- File not found errors
- Service failures

#### Expected Warnings (Safe to Ignore)

- SSH host key warnings: `Warning: Permanently added '...' to the list of known hosts`
- SSH with `StrictHostKeyChecking=no`: Any stderr about host verification
- Commands that succeed with exit code 0 but produce stderr warnings

### For Users

If you see red error messages in the output:

1. **Check if the operation completed successfully** - Did the overall command/test pass?
2. **Look for the specific patterns** mentioned in this document
3. **Check exit codes** - If the process completed with success, stderr warnings are usually harmless
4. **Focus on final results** - E2E tests will show overall PASS/FAIL status

## 🔮 Future Improvements

### Completed (September 2025)

- ✅ **Implemented smarter logging strategy**: Executor now logs both stdout and stderr at debug level when commands succeed
- ✅ **Used appropriate log levels**: No longer logging stderr as ERROR when commands succeed

### Short Term

- Document all known warning patterns
- Improve logging documentation for users
- Add troubleshooting guides

### Medium Term

- **Wrapper-level command handling**: Move command-specific output analysis from the generic executor to specialized wrappers (e.g., SSH connectivity checks in `src/shared/ssh/service_checker.rs`)
- Filter known harmless warnings from error logs at the wrapper level
- Implement command-specific error handling strategies

### Long Term

- Restructure logging to separate warnings from errors
- Implement user-friendly output modes
- Add verbose/debug modes for detailed information

## 🧩 Architecture Strategy: Wrapper-Level Error Handling

As of September 2025, we're moving toward a pattern where:

1. **Generic Executor**: The `src/shared/executor.rs` module remains generic and logs stdout/stderr at debug level when commands succeed
2. **Specialized Wrappers**: Command-specific modules (like `src/shared/ssh/service_checker.rs`) analyze command output and handle known patterns appropriately
3. **Context-Aware Logging**: Wrappers can distinguish between expected warnings and actual errors based on the specific command and context

### Example Implementation Pattern

```rust
// In a specialized wrapper like SSH service checker
pub fn check_connectivity(&self, host: &str) -> Result<(), SshError> {
    let output = self.executor.run_command("ssh", &[host, "echo", "connected"], None)?;

    // Wrapper analyzes the specific command output
    // Can handle SSH-specific warnings appropriately
    // Logs meaningful messages at the right level

    Ok(())
}
```

This approach keeps the executor generic while allowing command-specific intelligence in the appropriate modules.

### Long Term

- Restructure logging to separate warnings from errors
- Implement user-friendly output modes
- Add verbose/debug modes for detailed information

## 🚨 When to Be Concerned

Contact the development team or file an issue if you see:

- **Process failures**: Overall commands or tests failing
- **Connection errors**: Unable to connect to instances
- **Permission errors**: Unexpected permission denied messages
- **Service failures**: Applications not starting or responding
- **Data corruption**: Invalid configurations or lost data
- **Unknown error patterns**: Errors not documented here

## 📋 Reporting New Issues

When reporting new errors or unexpected behavior:

1. **Include full context**: Command being run, environment details
2. **Show exit codes**: Was the overall operation successful?
3. **Provide logs**: Full error messages and stack traces
4. **Describe impact**: What functionality is affected?
5. **Steps to reproduce**: How can others reproduce the issue?

## 🔗 Related Documentation

- [Console stdout/stderr Handling](../research/UX/console-stdout-stderr-handling.md) - Technical background on why these warnings appear
- [Error Handling Guide](./error-handling.md) - How we handle errors in the codebase
- [Development Principles](../development-principles.md) - Our approach to observability and user experience
- [E2E Testing](../e2e-testing.md) - Understanding E2E test output and results

## 📞 Getting Help

- **Documentation**: Check this document and related links first
- **GitHub Issues**: Search existing issues or create a new one
- **Development Team**: Contact maintainers for urgent issues
- **Community**: Engage with the Torrust community for general questions
