# Create Environment Command

**Status**: ✅ Fully Implemented

Create a new deployment environment from a configuration file. This command initializes the environment with validated configuration, SSH credentials, and prepares it for provisioning.

**Use when**:

- Setting up a new deployment environment
- Initializing environment configuration from templates
- Preparing environments for provisioning

**Command**: `torrust-tracker-deployer create environment`

## Syntax

```bash
torrust-tracker-deployer create environment --env-file <FILE> [OPTIONS]
```

**Required Arguments**:

- `--env-file <FILE>` (or `-f <FILE>`) - Path to environment configuration file (JSON format)

**Optional Flags**:

- `--working-dir <DIR>` - Working directory for environment data (default: `.`)
- `--log-output <OUTPUT>` - Logging output mode (default: `file-only`)
  - `file-only`: Write logs to file only (production mode)
  - `file-and-stderr`: Write logs to both file and stderr (development/testing mode)
- `--log-file-format <FORMAT>` - Format for file logging (default: `compact`)
  - `pretty`: Pretty-printed output for development (no ANSI codes in files)
  - `json`: JSON output for production environments
  - `compact`: Compact output for minimal verbosity
- `--log-stderr-format <FORMAT>` - Format for stderr logging (default: `pretty`)
  - `pretty`: Pretty-printed output with colors for development
  - `json`: JSON output for machine processing
  - `compact`: Compact output with colors for minimal verbosity
- `--log-dir <DIR>` - Log directory (default: `./data/logs`)

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
    "username": "torrust",
    "port": 22
  }
}
```

### Field Descriptions

**environment.name** (required):

- Unique environment identifier
- Must be lowercase alphanumeric with hyphens only
- Used for directory names and resource identification
- Example values: `dev-local`, `staging`, `production-01`

**ssh_credentials.private_key_path** (required):

- Path to SSH private key file
- Supports `~` for home directory expansion
- File must exist and be readable
- Used for connecting to provisioned infrastructure

**ssh_credentials.public_key_path** (required):

- Path to SSH public key file
- Supports `~` for home directory expansion
- File must exist and be readable
- Used for configuring VM access

**ssh_credentials.username** (required):

- SSH username for VM access
- Default value: `torrust`
- Used for connecting to provisioned infrastructure

**ssh_credentials.port** (optional):

- SSH port number
- Default value: `22` (standard SSH port)
- Port number for SSH connections

## Basic Usage

### Generate and Use Template

The recommended workflow is to generate a template first:

```bash
# Step 1: Generate configuration template
torrust-tracker-deployer create template config.json

# Step 2: Edit the configuration file
nano config.json
# Replace placeholder values:
# - REPLACE_WITH_ENVIRONMENT_NAME
# - REPLACE_WITH_SSH_PRIVATE_KEY_PATH
# - REPLACE_WITH_SSH_PUBLIC_KEY_PATH

# Step 3: Create the environment
torrust-tracker-deployer create environment --env-file config.json
```

Result: Environment created at `./data/my-environment/`

### Direct Creation

Create an environment with a prepared configuration file:

```bash
torrust-tracker-deployer create environment --env-file my-config.json
```

## Examples

### Basic Usage (Default Working Directory)

Create an environment in the default location (`./data/`):

```bash
# Prepare configuration file
cat > dev-config.json << 'EOF'
{
  "environment": {
    "name": "dev-local"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  }
}
EOF

# Create the environment
torrust-tracker-deployer create environment --env-file dev-config.json
```

Result: Environment created at `./data/dev-local/`

### Custom Working Directory

Create an environment in a custom location:

```bash
torrust-tracker-deployer create environment \
  --env-file config.json \
  --working-dir /opt/deployments
```

Result: Environment created at `/opt/deployments/data/dev-local/`

### With File and Console Logging

Create an environment with logging to both file and stderr for debugging:

```bash
torrust-tracker-deployer create environment \
  --env-file config.json \
  --log-output file-and-stderr
```

Result: Logs written to both `./data/logs/log.txt` and stderr

### Using Test SSH Keys

For development and testing, use the provided test SSH keys:

```bash
cat > test-config.json << 'EOF'
{
  "environment": {
    "name": "test-env"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "developer",
    "port": 22
  }
}
EOF

torrust-tracker-deployer create environment --env-file test-config.json
```

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
```

### Testing Environment

Setup for CI/CD testing with unique environment names:

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
  --log-output file-only
```

## What Gets Created

The create environment command initializes:

1. **Environment Directory Structure**
   - Creates `data/<environment-name>/` directory
   - Stores environment configuration
   - Prepares space for state files

2. **Environment State**
   - Initializes environment state to `Created`
   - Records environment metadata
   - Prepares for provisioning workflow

3. **Validated Configuration**
   - Validates all configuration fields
   - Verifies SSH key files exist and are readable
   - Ensures environment name follows naming conventions

## Troubleshooting

### Configuration Validation Errors

**Problem**: `Environment name 'My_Env' is invalid`

```text
Error: Configuration validation failed
Environment name 'My_Env' is invalid: must contain only lowercase letters,
numbers, and hyphens
```

**Solution**: Use lowercase alphanumeric characters and hyphens only:

```bash
# ❌ Invalid names
"name": "My_Env"       # Contains underscore and uppercase
"name": "my.env"       # Contains period
"name": "MY-ENV"       # Contains uppercase

# ✅ Valid names
"name": "my-env"
"name": "production-01"
"name": "dev-local"
```

### SSH Key Not Found

**Problem**: `SSH private key not found at path`

```text
Error: Configuration validation failed
SSH private key not found at '~/.ssh/missing_key'
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

# Option 2: Remove existing environment first
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

### Invalid JSON Format

**Problem**: `Failed to parse configuration file`

```text
Error: Configuration parsing failed
expected `,` or `}` at line 5 column 3
```

**Solution**: Validate configuration file syntax:

```bash
# Validate JSON syntax
jq empty config.json

# Or regenerate from template
torrust-tracker-deployer create template config.json

# Edit with valid JSON syntax
nano config.json
```

### SSH Key File Permissions

**Problem**: SSH key files exist but cannot be read

```text
Error: Configuration validation failed
SSH private key exists but cannot be read: permission denied
```

**Solution**: Fix SSH key file permissions:

```bash
# SSH private keys should have restricted permissions
chmod 600 ~/.ssh/id_rsa

# SSH public keys can be more permissive
chmod 644 ~/.ssh/id_rsa.pub

# Verify permissions
ls -l ~/.ssh/id_rsa*
```

### Working Directory Not Found

**Problem**: Custom working directory doesn't exist

```text
Error: Working directory '/opt/deployments' does not exist
```

**Solution**: Create the working directory first:

```bash
# Create directory with appropriate permissions
sudo mkdir -p /opt/deployments
sudo chown $(whoami):$(whoami) /opt/deployments

# Verify permissions
ls -ld /opt/deployments

# Then create environment
torrust-tracker-deployer create environment \
  --env-file config.json \
  --working-dir /opt/deployments
```

## Inspecting Logs

If environment creation fails, check the logs for detailed information:

```bash
# View logs with default file-only logging
cat data/logs/log.txt

# Or use file-and-stderr for real-time debugging
torrust-tracker-deployer create environment \
  --env-file config.json \
  --log-output file-and-stderr \
  --log-stderr-format pretty
```

The logs will show:

- Configuration validation details
- File system operations
- Environment creation progress
- Detailed error messages with context

## Idempotent Operation

The create environment command is **NOT idempotent**:

- If an environment already exists, creation will fail
- You must destroy the existing environment before recreating
- This prevents accidental data loss

```bash
# First creation succeeds
torrust-tracker-deployer create environment --env-file config.json

# Second creation fails (environment exists)
torrust-tracker-deployer create environment --env-file config.json
# Error: Environment 'my-env' already exists

# Must destroy first to recreate
torrust-tracker-deployer destroy my-env
torrust-tracker-deployer create environment --env-file config.json
```

## Exit Codes

- `0` - Success (environment created successfully)
- `1` - Error (creation failed due to validation or file system errors)

## Verification

After creating an environment, verify it was created successfully:

### Check Environment Directory

```bash
# Verify environment directory exists
ls -la ./data/my-environment/

# Check environment configuration
cat ./data/my-environment/environment.json
```

### Check Logs

```bash
# View creation logs
cat ./data/logs/log.txt

# Look for success message
grep "Environment created successfully" ./data/logs/log.txt
```

## Related Commands

- [`create template`](#generate-template) - Generate configuration file template
- [`destroy`](./destroy.md) - Remove environment and clean up resources
- [Command Index](../commands.md) - Overview of all commands

## Next Steps

After creating an environment, the typical workflow is:

1. **Verify Creation**: Check that environment directory and configuration exist

   ```bash
   ls -la ./data/my-environment/
   cat ./data/my-environment/environment.json
   ```

2. **Provision Infrastructure**: Deploy the infrastructure (future command)

   ```bash
   # Future command - not yet implemented
   torrust-tracker-deployer provision my-environment
   ```

3. **Monitor Logs**: Check logs for any issues

   ```bash
   tail -f ./data/logs/log.txt
   ```

## Generate Template

For convenience, use the `create template` command to generate a configuration template:

```bash
# Generate template with default name
torrust-tracker-deployer create template

# Generate template with custom name
torrust-tracker-deployer create template my-config.json
```

The template will contain placeholder values that you need to replace:

- `REPLACE_WITH_ENVIRONMENT_NAME` - Choose a unique environment name
- `REPLACE_WITH_SSH_PRIVATE_KEY_PATH` - Path to your SSH private key
- `REPLACE_WITH_SSH_PUBLIC_KEY_PATH` - Path to your SSH public key

Edit the generated template and then use it to create your environment.

## See Also

- [Logging Guide](../logging.md) - Configure logging output and formats
- [Template Customization](../template-customization.md) - Advanced template configuration
