# Validate Command

Validates environment configuration files without creating deployments. Use this command to check configuration correctness before running the actual deployment workflow.

## Command Syntax

```bash
torrust-tracker-deployer validate --env-file <CONFIG_FILE>
```

### Options

- `--env-file, -f <FILE>` - Path to the environment configuration file to validate (required)

## Usage Examples

### Basic Usage

```bash
torrust-tracker-deployer validate --env-file envs/my-environment.json
```

### Validate with Short Flag

```bash
torrust-tracker-deployer validate -f config/production.json
```

## What This Command Does

The validate command performs comprehensive validation of environment configuration files:

1. **File Validation** - Verifies the configuration file exists and is readable
2. **JSON Schema Validation** - Checks JSON syntax and structure
3. **Domain Validation** - Validates field constraints and business rules:
   - SSH key files must exist at specified paths
   - Environment names must follow naming rules (lowercase with dashes)
   - Port numbers must be valid
   - IP addresses must be well-formed
   - Domain names must follow DNS conventions
   - All required fields must be present

## When to Use

- **Before Creating Environments** - Catch configuration errors early before starting infrastructure provisioning
- **CI/CD Pipelines** - Validate configurations as part of automated testing
- **Configuration Review** - Verify configuration correctness during code review
- **Troubleshooting** - Diagnose why environment creation failed
- **Learning** - Understand configuration structure and requirements

## When NOT to Use

- **After Environment Creation** - Use `show` command to view existing environment details instead
- **Creating Templates** - Use `create template` to generate valid configuration files
- **Deployment** - Validate does not deploy - use full workflow commands for actual deployment

## Output Details

### Success Output

```text
⏳ [1/3] Loading configuration file...
⏳   ✓ Configuration file loaded (took 0ms)
⏳ [2/3] Validating JSON schema...
⏳   ✓ Schema validation passed (took 0ms)
⏳ [3/3] Validating configuration fields...
⏳   ✓ Field validation passed (took 0ms)

✅ Configuration file 'envs/full-stack-docs.json' is valid

Environment Details:
• Name: full-stack-docs
• Provider: lxd
• Prometheus: Enabled
• Grafana: Enabled
• HTTPS: Enabled
• Backups: Enabled
```

### Error Output Examples

**File Not Found**:

```text
❌ Validate command failed: Configuration file not found: /tmp/nonexistent.json

For detailed troubleshooting:
Verify the file path is correct: /tmp/nonexistent.json
Use 'create template' to generate a valid configuration file.
```

**Invalid JSON**:

```text
❌ Validate command failed: Validation failed for configuration file: config.json

For detailed troubleshooting:
JSON parsing failed for file 'config.json'.

Error details:
key must be a string at line 1 column 3

Common issues:
- Missing or extra commas
- Unmatched braces or brackets
- Invalid escape sequences
- Comments (not allowed in JSON)

Tips:
- Use a JSON validator or editor with syntax highlighting
- Compare with a template: 'create template --provider lxd'
- Check the JSON schema in schemas/environment-config.json
```

**Missing SSH Keys**:

```text
❌ Validate command failed: Validation failed for configuration file: config.json

For detailed troubleshooting:
Configuration validation failed.

Error: SSH private key file not found: /tmp/nonexistent-key

This means the configuration file has valid JSON syntax but violates
domain constraints or business rules.

Common issues:
- SSH key files don't exist at specified paths
- Invalid environment name (must be lowercase with dashes)
- Invalid port numbers or IP addresses
- Missing required fields
- HTTPS configured but no services have TLS enabled
```

## Common Scenarios

### Scenario 1: Pre-Deployment Validation

Before starting a deployment, validate your configuration:

```bash
# Create configuration from template
torrust-tracker-deployer create template -p lxd > envs/my-env.json

# Edit configuration...
# vim envs/my-env.json

# Validate before deploying
torrust-tracker-deployer validate -f envs/my-env.json

# If valid, proceed with deployment
torrust-tracker-deployer create environment -f envs/my-env.json
```

### Scenario 2: CI/CD Pipeline Integration

Add validation as a CI test:

```bash
#!/bin/bash
# .github/workflows/validate-configs.sh

for config in envs/*.json; do
    echo "Validating $config..."
    torrust-tracker-deployer validate -f "$config"

    if [ $? -ne 0 ]; then
        echo "❌ Validation failed for $config"
        exit 1
    fi
done

echo "✅ All configurations valid"
```

### Scenario 3: Troubleshooting Configuration Issues

When environment creation fails, use validate to diagnose:

```bash
# Creation fails
torrust-tracker-deployer create environment -f envs/broken.json
# Error: SSH key not found...

# Use validate for detailed error messages
torrust-tracker-deployer validate -f envs/broken.json
# Shows: SSH private key file not found: /path/to/key
#        With troubleshooting tips...

# Fix issue and re-validate
vim envs/broken.json
torrust-tracker-deployer validate -f envs/broken.json
# ✅ Configuration valid

# Retry creation
torrust-tracker-deployer create environment -f envs/broken.json
```

## Troubleshooting

### Error: Configuration file not found

**Cause**: The file path is incorrect or the file doesn't exist.

**Solution**: Verify the file path and ensure the file exists:

```bash
# Check if file exists
ls -l envs/my-config.json

# Use absolute path if relative path fails
torrust-tracker-deployer validate -f /full/path/to/envs/my-config.json

# Generate a template if starting fresh
torrust-tracker-deployer create template -p lxd > envs/new-config.json
```

### Error: JSON parsing failed

**Cause**: The JSON syntax is invalid (missing commas, unmatched braces, etc.).

**Solution**: Use a JSON validator or IDE with JSON support:

```bash
# Use jq to validate and format
jq . envs/my-config.json

# Use Python's JSON tool
python -m json.tool < envs/my-config.json

# Compare with template
torrust-tracker-deployer create template -p lxd | jq . > valid-template.json
diff envs/my-config.json valid-template.json
```

### Error: SSH key file not found

**Cause**: The SSH key paths in the configuration don't exist.

**Solution**: Generate SSH keys or update paths to existing keys:

```bash
# Option 1: Generate new SSH keys
ssh-keygen -t rsa -b 4096 -f ~/.ssh/torrust_deployer -N ""

# Option 2: Update configuration with correct paths
vim envs/my-config.json
# Update "private_key_path" and "public_key_path"

# Validate
torrust-tracker-deployer validate -f envs/my-config.json
```

### Error: Invalid environment name

**Cause**: Environment name doesn't follow naming rules.

**Solution**: Use valid naming format (lowercase letters, numbers, dashes):

```bash
# Invalid names:
# - "MyEnv" (uppercase)
# - "my_env" (underscores)
# - "123-env" (starts with number)
# - "my-env-" (ends with dash)

# Valid names:
# - "my-env"
# - "production"
# - "dev-01"
# - "staging-2025"

# Fix in configuration
vim envs/my-config.json
# Change "name": "MyEnv" to "name": "my-env"

torrust-tracker-deployer validate -f envs/my-config.json
```

## Related Commands

- [`create template`](./create-template.md) - Generate valid configuration templates
- [`create environment`](./create-environment.md) - Create deployment environments (runs validation internally)
- [`show`](./show.md) - Display existing environment details

## See Also

- [Environment Creation Configuration](../../user-guide/environment-creation-config.md)
- [JSON Schema](../../../schemas/environment-config.json)
- [SSH Key Setup](../../user-guide/ssh-key-setup.md)
- [Troubleshooting Guide](../../user-guide/troubleshooting.md)
