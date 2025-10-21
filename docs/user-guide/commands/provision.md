# Provision Command

> **Status**: Documentation in progress

The `provision` command creates new infrastructure with virtual machines and networking.

## Command Syntax

```bash
torrust-tracker-deployer provision <ENVIRONMENT_NAME>
```

## Overview

The provision command:

- Creates LXD virtual machines or containers
- Configures network interfaces and bridges
- Sets up SSH access
- Initializes system state with cloud-init

## Documentation Status

This command is implemented but full user documentation is pending.

For developer documentation, see:

- [Command Architecture Developer Guide](../../contributing/commands.md#provisioncommand)
- [Console Commands Overview](../../console-commands.md)

## See Also

- [`configure`](configure.md) - Configure provisioned infrastructure
- [`destroy`](destroy.md) - Remove infrastructure and cleanup
- [Logging Guide](../logging.md) - Configure logging output and formats
