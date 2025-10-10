# Known Issues and Expected Behaviors

This document describes known issues, expected errors, and normal behaviors that may appear alarming but are actually under control in the Torrust Tracker Deployer project.

## 🟡 Expected Warnings During Operation

### SSH Host Key Warnings

**What You'll See**: During SSH operations, you may see warnings like this at WARN level:

```text
2025-09-30T14:34:48.175396Z  WARN torrust_tracker_deploy::shared::ssh::client: SSH warning detected, operation: "ssh_warning", host_ip: 10.140.190.14, Warning: Permanently added '10.140.190.14' (ED25519) to the list of known hosts.
    at src/shared/ssh/client.rs:157
    in torrust_tracker_deploy::application::steps::connectivity::wait_ssh_connectivity::wait_ssh_connectivity with step_type: "connectivity", protocol: "ssh"
    in torrust_tracker_deploy::application::commands::provision::provision_command with command_type: "provision"
```

**Why This Happens**:

- SSH writes host key information to stderr when connecting to new hosts
- Our SSH configuration uses `-o StrictHostKeyChecking=no` for automation
- SSH informs us that it's adding the host key to the known_hosts file, which is normal security behavior
- The application detects these warnings and logs them at WARN level for visibility

**Is This An Error?**: **NO** - This is expected and normal SSH behavior

**What It Means**: SSH is successfully connecting and securely recording the host's identity for future connections

## 🔧 Current Error Handling Strategy

### Logging Levels

- **ERROR**: Actual failures that require attention
- **WARN**: Expected warnings that users should be aware of (e.g., SSH host key additions)
- **DEBUG**: Detailed execution information for troubleshooting

### SSH Warning Detection

SSH warnings are automatically detected and logged with structured context including:

- Operation type (`ssh_warning`)
- Target host IP address
- Complete warning message
- Source location in code
- Execution context (which step/command triggered it)

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

- [Error Handling Guide](./error-handling.md) - How we handle errors in the codebase
- [Development Principles](../development-principles.md) - Our approach to observability and user experience
- [E2E Testing](../e2e-testing.md) - Understanding E2E test output and results

## 📞 Getting Help

- **Documentation**: Check this document and related links first
- **GitHub Issues**: Search existing issues or create a new one
- **Development Team**: Contact maintainers for urgent issues
- **Community**: Engage with the Torrust community for general questions
