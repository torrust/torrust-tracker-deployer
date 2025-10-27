# Document Create Environment Command

**Epic Subissue**: 10 of 10
**Issue**: [#52](https://github.com/torrust/torrust-tracker-deployer/issues/52)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Create Environment Command
**Related**: [User Guide Commands](../user-guide/commands.md), [Create Environment Command Documentation](../user-guide/commands/create-environment.md)

## Overview

Add comprehensive user-facing documentation for the `create environment` command to the user guide. This documentation will help users understand how to create new deployment environments from configuration files.

The create environment command is now fully implemented (Subissues 1-7) and needs proper user-facing documentation in the `docs/user-guide/` directory.

## Goals

- [ ] Create detailed command documentation at `docs/user-guide/commands/create-environment.md`
- [ ] Update `docs/user-guide/commands.md` to mark create command as implemented
- [ ] Include practical examples for common use cases
- [ ] Document all command flags and configuration file formats
- [ ] Provide troubleshooting guidance for common errors

## 🏗️ Architecture Requirements

**Documentation Type**: User Guide
**Module Path**: `docs/user-guide/commands/`
**Pattern**: Command Reference Documentation

### Documentation Structure Requirements

- [ ] Follow existing command documentation pattern (see [docs/user-guide/commands/destroy.md](../user-guide/commands/destroy.md))
- [ ] Include command synopsis, description, usage examples, and troubleshooting
- [ ] Use clear, actionable language for user-facing documentation
- [ ] Provide both JSON and TOML configuration examples

### Anti-Patterns to Avoid

- ❌ Developer-focused implementation details in user guide
- ❌ Missing practical examples
- ❌ Unclear error troubleshooting guidance
- ❌ Inconsistent formatting with other command docs

## Specifications

### Documentation Structure

The command documentation should follow this structure:

1. **Command Overview**

   - Brief description of what the command does
   - When to use it
   - Implementation status badge

2. **Syntax**

   - Command signature with all flags
   - Required vs optional parameters

3. **Configuration File Format**

   - JSON format example
   - TOML format example
   - Field descriptions with validation rules

4. **Examples**

   - Basic usage (minimal configuration)
   - Advanced usage (custom working directory)
   - Template generation workflow

5. **Common Use Cases**

   - Development environment setup
   - Testing environment setup
   - Production environment setup

6. **Troubleshooting**

   - Configuration validation errors
   - File system permission issues
   - Invalid SSH key paths
   - Environment name conflicts

7. **Related Commands**
   - Link to provision command
   - Link to destroy command
   - Link to template generation

### Content to Include

#### Command Overview Section

```markdown
# Create Environment Command

**Status**: ✅ Fully Implemented

Create a new deployment environment from a configuration file. This command initializes the environment with validated configuration, SSH credentials, and prepares it for provisioning.

**Use when**:

- Setting up a new deployment environment
- Initializing environment configuration from templates
- Preparing environments for provisioning

**Command**: `torrust-tracker-deployer create environment`
```

#### Syntax Section

````markdown
## Syntax

```bash
torrust-tracker-deployer create environment --env-file <path> [options]
```
````

### Required Flags

- `--env-file <path>` - Path to environment configuration file (JSON or TOML)

### Optional Flags

- `--working-dir <path>` - Working directory for environment data (default: `.`)
- `--log-output <mode>` - Logging output mode: `stderr`, `file`, `file-and-stderr` (default: `stderr`)

````<!-- markdownlint-disable-line MD040 -->

#### Configuration Format Section

```markdown
## Configuration File Format

### JSON Format

```json
{
  "environment": {
    "name": "my-environment"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "deployer",
    "port": 22
  }
}
````

### TOML Format

```toml
[environment]
name = "my-environment"

[ssh_credentials]
private_key_path = "~/.ssh/id_rsa"
public_key_path = "~/.ssh/id_rsa.pub"
username = "deployer"
port = 22
```

### Field Descriptions

- **environment.name** (required): Unique environment identifier
  - Must be lowercase alphanumeric with hyphens
  - Used for directory names and resource identification
- **ssh_credentials.private_key_path** (required): Path to SSH private key
  - Supports `~` for home directory expansion
  - File must exist and be readable
- **ssh_credentials.public_key_path** (required): Path to SSH public key
  - Supports `~` for home directory expansion
  - File must exist and be readable
- **ssh_credentials.username** (required): SSH username for VM access
  - Used for connecting to provisioned infrastructure
- **ssh_credentials.port** (optional): SSH port (default: 22)
  - Port number for SSH connections

````<!-- markdownlint-disable-line MD040 -->

#### Examples Section

```markdown
## Examples

### Basic Usage (Default Working Directory)

Create an environment in the default location (`./data/`):

```bash
# Generate configuration template
torrust-tracker-deployer create template --output-file config.json

# Edit the configuration file
nano config.json

# Create the environment
torrust-tracker-deployer create environment --env-file config.json
````

Result: Environment created at `./data/my-environment/`

### Custom Working Directory

Create an environment in a custom location:

```bash
torrust-tracker-deployer create environment \
  --env-file config.json \
  --working-dir /opt/deployments
```

Result: Environment created at `/opt/deployments/data/my-environment/`

### With File Logging

Create an environment with detailed file logging:

```bash
torrust-tracker-deployer create environment \
  --env-file config.json \
  --log-output file-and-stderr
```

Result: Logs written to `./data/logs/log.txt` and stderr

````<!-- markdownlint-disable-line MD040 -->

#### Common Use Cases Section

```markdown
## Common Use Cases

### Development Environment

Quick setup for local development and testing:

```bash
# Use test SSH keys from fixtures
cat > dev-config.json << 'EOF'
{
  "environment": {
    "name": "dev-local"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "developer",
    "port": 22
  }
}
EOF

torrust-tracker-deployer create environment --env-file dev-config.json
````

### Testing Environment

Setup for CI/CD testing:

```bash
# Generate unique environment name for test run
TEST_ENV="test-$(date +%s)"

cat > test-config.json << EOF
{
  "environment": {
    "name": "${TEST_ENV}"
  },
  "ssh_credentials": {
    "private_key_path": "${HOME}/.ssh/ci_key",
    "public_key_path": "${HOME}/.ssh/ci_key.pub",
    "username": "ci-runner",
    "port": 22
  }
}
EOF

torrust-tracker-deployer create environment --env-file test-config.json
```

### Production Environment

Production setup with security best practices:

```bash
# Use dedicated production SSH key with passphrase
cat > prod-config.json << 'EOF'
{
  "environment": {
    "name": "production"
  },
  "ssh_credentials": {
    "private_key_path": "/secure/keys/production_id_rsa",
    "public_key_path": "/secure/keys/production_id_rsa.pub",
    "username": "deploy-prod",
    "port": 22
  }
}
EOF

# Use dedicated working directory
sudo mkdir -p /opt/torrust-deployments
sudo chown $(whoami):$(whoami) /opt/torrust-deployments

torrust-tracker-deployer create environment \
  --env-file prod-config.json \
  --working-dir /opt/torrust-deployments \
  --log-output file
```

````<!-- markdownlint-disable-line MD040 -->

#### Troubleshooting Section

```markdown
## Troubleshooting

### Configuration Validation Errors

**Problem**: `Environment name 'My_Env' is invalid`

```text
Error: Configuration validation failed
Environment name 'My_Env' is invalid: must contain only lowercase letters,
numbers, and hyphens
````

**Solution**: Use lowercase alphanumeric characters and hyphens only:

```bash
# ❌ Invalid
"name": "My_Env"
"name": "my.env"
"name": "MY-ENV"

# ✅ Valid
"name": "my-env"
"name": "production-01"
"name": "dev-local"
```

### SSH Key Not Found

**Problem**: `SSH private key not found at path`

```text
Error: Configuration validation failed
SSH private key not found at 'fixtures/missing_key'
```

**Solution**: Verify SSH key paths exist:

```bash
# Check if keys exist
ls -la ~/.ssh/id_rsa*

# Generate new keys if needed
ssh-keygen -t rsa -b 4096 -f ~/.ssh/deployer_key

# Update configuration with correct paths
"private_key_path": "~/.ssh/deployer_key"
"public_key_path": "~/.ssh/deployer_key.pub"
```

### Environment Already Exists

**Problem**: `Environment 'my-env' already exists`

```text
Error: Environment creation failed
Environment 'my-env' already exists at './data/my-env'
```

**Solution**: Choose a different name or remove the existing environment:

```bash
# Option 1: Use different name
# Edit config.json and change environment.name

# Option 2: Remove existing environment
torrust-tracker-deployer destroy my-env

# Then retry create
torrust-tracker-deployer create environment --env-file config.json
```

### Permission Denied

**Problem**: `Permission denied when creating directory`

```text
Error: Failed to create environment directory
Permission denied (os error 13): './data/my-env'
```

**Solution**: Ensure write permissions for the working directory:

```bash
# Check permissions
ls -ld ./data

# Create directory with correct permissions
mkdir -p ./data
chmod 755 ./data

# Or use a directory you own
torrust-tracker-deployer create environment \
  --env-file config.json \
  --working-dir ~/deployments
```

### Invalid JSON/TOML Format

**Problem**: `Failed to parse configuration file`

```text
Error: Configuration parsing failed
expected `,` or `}` at line 5 column 3
```

**Solution**: Validate configuration file syntax:

```bash
# Validate JSON
jq empty config.json

# Validate TOML
toml-cli check config.toml

# Or regenerate from template
torrust-tracker-deployer create template --output-file config.json
```

````<!-- markdownlint-disable-line MD040 -->

#### Related Commands Section

```markdown
## Related Commands

- [`create template`](./create-template.md) - Generate configuration file template
- [`provision`](./provision.md) - Provision infrastructure for the environment
- [`destroy`](./destroy.md) - Remove environment and clean up resources
- [`status`](./status.md) - Check environment status (future)

## Next Steps

After creating an environment:

1. **Verify Creation**: Check that environment directory exists

   ```bash
   ls -la ./data/my-environment/
   cat ./data/my-environment/environment.json
````

1. **Provision Infrastructure**: Deploy the infrastructure

   ```bash
   torrust-tracker-deployer provision my-environment
   ```

1. **Monitor Logs**: Check logs for any issues

   ```bash
   tail -f ./data/logs/log.txt
   ```

````<!-- markdownlint-disable-line MD040 -->

### Update to commands.md

Update the main commands index file to mark the create command as implemented:

```markdown
## Planned Commands (MVP)

```bash
# Available commands
torrust-tracker-deployer create environment   # Create new deployment environment
torrust-tracker-deployer create template      # Generate configuration template
torrust-tracker-deployer destroy <env>        # Destroy infrastructure and clean up resources

# Future commands
torrust-tracker-deployer deploy <env>         # Future - Full deployment (provision → configure → release)
torrust-tracker-deployer run <env>            # Future - Start application services
torrust-tracker-deployer status <env>         # Future - Check environment status
torrust-tracker-deployer test <env>           # Future - Run validation tests
````

**Note**: The `deploy` command will internally orchestrate the complete deployment workflow: provision → configure → release.

## Available Commands

### Environment Management

#### [`create environment`](commands/create-environment.md)

Create a new deployment environment from a configuration file.

**Status**: ✅ Fully Implemented

**Use when**: Initializing new environments for development, testing, or production

```bash
torrust-tracker-deployer create environment --env-file ./config/environment.json
```

Creates:

- Environment data directory
- Initial environment state (Created)
- Persistent configuration

```<!-- markdownlint-disable-line MD040 -->

### Update to commands.md

#### [`destroy`](commands/destroy.md)

Remove environment and clean up all resources.
```

## Implementation Plan

### Subtask 1: Create Command Documentation File (2 hours)

- [ ] Create `docs/user-guide/commands/create-environment.md`
- [ ] Write comprehensive command overview and description
- [ ] Document syntax with all flags (required and optional)
- [ ] Include configuration file format for both JSON and TOML
- [ ] Add field descriptions with validation rules
- [ ] Provide practical examples for common use cases

### Subtask 2: Add Troubleshooting Section (1 hour)

- [ ] Document common configuration validation errors
- [ ] Provide solutions for SSH key issues
- [ ] Include guidance for environment name conflicts
- [ ] Add permission and file system error troubleshooting
- [ ] Document JSON/TOML parsing errors

### Subtask 3: Update Commands Index (30 minutes)

- [ ] Update `docs/user-guide/commands.md` implementation status
- [ ] Mark create command as "✅ Fully Implemented"
- [ ] Update command list to show create environment and create template
- [ ] Add cross-references to new documentation
- [ ] Update "Available Commands" section with create command entry

### Subtask 4: Review and Testing (30 minutes)

- [ ] Verify all markdown links work correctly
- [ ] Test example commands for accuracy
- [ ] Ensure consistency with other command documentation
- [ ] Run markdown linter: `cargo run --bin linter markdown`
- [ ] Verify all pre-commit checks pass

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Markdown linting passes: `cargo run --bin linter markdown`
- [ ] All links are valid and working

**Documentation Completeness**:

- [ ] Command documentation created at `docs/user-guide/commands/create-environment.md`
- [ ] Documentation includes all required sections:
  - [ ] Command overview with status badge
  - [ ] Syntax with required and optional flags
  - [ ] Configuration file formats (JSON and TOML)
  - [ ] Field descriptions with validation rules
  - [ ] Practical examples (basic, custom working dir, file logging)
  - [ ] Common use cases (development, testing, production)
  - [ ] Comprehensive troubleshooting section
  - [ ] Related commands and next steps
- [ ] Commands index (`docs/user-guide/commands.md`) updated
- [ ] Create command marked as "✅ Fully Implemented"
- [ ] Cross-references to create command documentation added

**Example Validation**:

- [ ] All example commands are syntactically correct
- [ ] Configuration file examples are valid JSON/TOML
- [ ] File paths in examples follow project conventions
- [ ] Use cases represent realistic scenarios

**Consistency**:

- [ ] Follows same structure as existing command documentation (destroy.md)
- [ ] Uses consistent terminology across documentation
- [ ] Formatting matches other user guide pages
- [ ] Code blocks properly formatted with syntax highlighting

## Related Documentation

- [User Guide Commands Index](../user-guide/commands.md)
- [Destroy Command Documentation](../user-guide/commands/destroy.md)
- [Epic #34 - Create Environment Command](./34-epic-create-environment-command.md)
- [Contributing Guide](../contributing/README.md)

## Notes

### Documentation Audience

This documentation is for **end users**, not developers:

- Use clear, actionable language
- Focus on "how to use" rather than "how it works"
- Provide practical examples users can copy and adapt
- Include troubleshooting for common user errors
- Avoid implementation details

### Relationship to Other Subissues

This subissue depends on completion of Subissues 1-7 (create command implementation). It documents the fully implemented command for user consumption.

### Template Generation Documentation

While this issue focuses on documenting the `create environment` command, there should also be documentation for `create template`. Consider creating that in a follow-up or including basic coverage here with a note that detailed template documentation may come later.

### Maintenance

After creating this documentation:

- Keep it updated when command behavior changes
- Add new troubleshooting cases as users report issues
- Update examples if configuration format evolves
- Monitor user feedback for documentation improvements

### Time Estimate

**Total time**: 4 hours

- Command documentation file: 2 hours
- Troubleshooting section: 1 hour
- Commands index update: 30 minutes
- Review and testing: 30 minutes

This estimate assumes familiarity with the create command implementation and access to example configurations.
