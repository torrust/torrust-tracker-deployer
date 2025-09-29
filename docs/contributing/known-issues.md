# Known Issues and Expected Behaviors

This document describes known issues, expected errors, and normal behaviors that may appear alarming but are actually under control in the Torrust Tracker Deploy project.

## 🔍 Overview

During E2E tests and normal operation, some console output may appear as errors (displayed in red) even though they represent expected behavior or harmless warnings. This document catalogs these known cases to help developers and users understand what's normal versus what requires attention.

## 🟡 Expected "Errors" in E2E Tests

### SSH Host Key Warnings

**Appearance**: Red error messages in logs during SSH operations

**Example Output**:

```text
2025-09-29T11:32:57.584329Z ERROR torrust_tracker_deploy::shared::executor: Command produced stderr output, operation: "command_execution", command: ssh -i /tmp/.tmpSZhY0Y/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 22 torrust@10.140.190.144 rm -f /tmp/test-docker-compose.yml, stderr: Warning: Permanently added '10.140.190.144' (ED25519) to the list of known hosts.
```

**Why This Happens**:

- SSH writes host key information to stderr (not stdout)
- Our executor currently treats all stderr output as errors
- The SSH options we use (`-o StrictHostKeyChecking=no`) generate these warnings by design

**Is This Actually An Error?**: **NO** - This is expected behavior

**Root Cause**: SSH is informing us that it's adding the host key to the known_hosts file, which is normal security behavior.

**Commands Affected**:

- `ssh -i <key> -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 22 user@host <command>`
- Any SSH-based operations during provisioning and deployment

**Current Status**: **Known and Under Control**

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

**Current Status**: **Known and Under Control**

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

### Short Term

- Document all known warning patterns
- Improve logging documentation for users
- Add troubleshooting guides

### Medium Term

- Implement smarter error detection based on exit codes
- Filter known harmless warnings from error logs
- Use appropriate log levels (WARN vs ERROR)

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
