# Manual E2E Test: Render Command

This guide provides step-by-step instructions for manually testing the `render` command and verifying that it generates identical artifacts to the `provision` command.

## 📋 Overview

The `render` command generates deployment artifacts **without provisioning infrastructure**. This allows you to:

- Preview what will be deployed before committing to infrastructure provisioning
- Generate artifacts for manual deployment
- Inspect and validate configuration before provisioning
- Compare artifacts between different configurations

**Key Principle**: The render command should generate **identical** artifacts to those created during provisioning, except for IP addresses (which come from real infrastructure during provision).

## 🎯 Test Objectives

This manual test verifies:

1. **Artifact Generation**: render command successfully creates all deployment artifacts
2. **Artifact Completeness**: All 8 service templates are rendered (OpenTofu, Ansible, Docker Compose, Tracker, Prometheus, Grafana, Caddy, Backup)
3. **Artifact Equivalence**: Artifacts match those created by provision command (except IP addresses)
4. **Dual Input Modes**: Both `--env-name` and `--env-file` modes work correctly
5. **Idempotency**: Multiple render calls produce consistent results

## 📝 Test Prerequisites

- **LXD configured**: Required for provision comparison (see [LXD Setup](../README.md))
- **Test SSH keys**: Use `fixtures/testing_rsa` and `fixtures/testing_rsa.pub`
- **Clean workspace**: No existing test environments

```bash
# Verify no existing test environment
cargo run -- list

# Check dependencies
cargo run --bin dependency-installer check
```

## 🔄 Test Workflow

### Test 1: Render with Environment Name (Full Comparison)

This test compares render command output with actual provision command output.

#### Step 1: Create Environment

```bash
# Generate configuration template
cargo run -- create template --provider lxd envs/render-test.json

# Customize the template
nano envs/render-test.json
```

**Required customizations**:

```json
{
  "environment": {
    "name": "render-test"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-render-test"
  }
}
```

```bash
# Create environment (state: Created)
cargo run -- create environment --env-file envs/render-test.json
```

**Expected output**:

```text
✅ Environment 'render-test' created successfully!

State: Created
```

**Verify environment state**:

```bash
cargo run -- show render-test
```

Should show state: `Created`

#### Step 2: Render Artifacts

```bash
# Render artifacts with a test IP address to a preview directory
cargo run -- render --env-name render-test --instance-ip 192.168.1.100 --output-dir ./render-test-preview
```

**Expected output**:

```text
⏳ [1/3] Validating input parameters...
⏳   ✓ Done (took Xms)
⏳ [2/3] Loading configuration...
⏳   ✓ Done (took Xms)
⏳ [3/3] Generating deployment artifacts...
⏳   ✓ Done (took Xms)
✅
Deployment artifacts generated successfully!

  Source: Environment: render-test
  Target IP: 192.168.1.100
  Output: ./render-test-preview

Next steps:
  - Review artifacts in the output directory
  - Use 'provision' command to deploy infrastructure
  - Or use artifacts manually with your deployment tools
```

**Save the rendered artifacts location**:

```bash
RENDER_PREVIEW_DIR="render-test-preview"
```

#### Step 3: Verify Rendered Artifacts

```bash
# List generated artifacts
ls -la $RENDER_PREVIEW_DIR/

# Should contain:
# - tofu/           - OpenTofu infrastructure code
# - ansible/        - Ansible playbooks and inventory
# - docker-compose/ - Docker Compose configuration
# - tracker/        - Tracker configuration
# - prometheus/     - Prometheus configuration
# - grafana/        - Grafana provisioning
# - caddy/          - Caddy reverse proxy config (if HTTPS enabled)
# - backup/         - Backup scripts (if enabled)
```

**Check OpenTofu configuration**:

```bash
cat $RENDER_PREVIEW_DIR/tofu/main.tf
# Should contain LXD VM configuration
```

**Check Ansible inventory**:

```bash
cat $RENDER_PREVIEW_DIR/ansible/inventory.yml
# Should contain: ansible_host: 192.168.1.100
```

**Check Docker Compose**:

```bash
cat $RENDER_PREVIEW_DIR/docker-compose/docker-compose.yml
# Should contain service definitions
```

#### Step 4: Provision Environment (For Comparison)

```bash
# Provision infrastructure - this will create a real VM
cargo run -- provision render-test
```

**Expected output**:

```text
✅ Environment 'render-test' provisioned successfully!

  Instance IP: <ACTUAL_IP>
  SSH Port: 22
  Instance Name: render-test-<ID>

State: Provisioned
```

**Save the actual IP for comparison**:

```bash
# Get actual IP from show command
ACTUAL_IP=$(cargo run -- show render-test | grep "IP Address" | awk '{print $3}')
echo "Actual IP: $ACTUAL_IP"
```

**Note**: Provisioning will also generate artifacts in `build/render-test/` with the actual IP.

#### Step 5: Compare Render vs Provision Artifacts

```bash
# After provision completes, compare render preview with provision output
# Preview directory: render-test-preview (from render command)
# Provision directory: build/render-test/ (from provision command)
diff -r $RENDER_PREVIEW_DIR build/render-test/
```

**Expected differences**:

1. **Ansible inventory** - IP addresses:

   ```bash
   # Rendered version
   ansible_host: 192.168.1.100  # Test IP

   # Provisioned version
   ansible_host: 10.x.x.x       # Actual VM IP
   ```

2. **No other differences** - All other files should be identical

**Detailed comparison**:

```bash
# Compare OpenTofu configurations (should be identical)
diff $RENDER_PREVIEW_DIR/tofu/main.tf build/render-test/tofu/main.tf

# Compare Docker Compose (should be identical)
diff $RENDER_PREVIEW_DIR/docker-compose/docker-compose.yml \
     build/render-test/docker-compose/docker-compose.yml

# Compare Ansible inventory (only IP should differ)
diff $RENDER_PREVIEW_DIR/ansible/inventory.yml \
     build/render-test/ansible/inventory.yml
```

#### Step 6: Test Idempotency

```bash
# Try to render again to the same output directory (should fail without --force)
cargo run -- render --env-name render-test --instance-ip 192.168.1.100 --output-dir ./render-test-preview

# Should fail with: "Output directory already exists"

# With --force flag, should succeed
cargo run -- render --env-name render-test --instance-ip 192.168.1.100 --output-dir ./render-test-preview --force

# Should succeed without errors
# Artifacts should remain unchanged
```

#### Step 7: Cleanup

```bash
# Destroy the infrastructure
cargo run -- destroy render-test

# Purge the environment data
cargo run -- purge render-test --force

# Remove preview artifacts
rm -rf $RENDER_PREVIEW_DIR
```

---

### Test 2: Render with Configuration File (No Environment Creation)

This test verifies the `--env-file` mode which generates artifacts directly from a config file.

#### Step 1: Generate Configuration File

```bash
# Create config file (don't create environment)
cargo run -- create template --provider lxd envs/render-direct.json

# Customize configuration
nano envs/render-direct.json
```

**Set environment name**:

```json
{
  "environment": {
    "name": "render-direct"
  }
}
```

#### Step 2: Render from Config File

```bash
# Render directly from config file (no environment creation)
cargo run -- render --env-file envs/render-direct.json --instance-ip 192.168.1.200 --output-dir ./render-direct-artifacts
```

**Expected output**:

```text
✅
Deployment artifacts generated successfully!

  Source: Config file: envs/render-direct.json
  Target IP: 192.168.1.200
  Output: ./render-direct-artifacts
```

#### Step 3: Verify Artifacts

```bash
# Check artifacts were generated
ls -la render-direct-artifacts/

# Verify IP in Ansible inventory
grep "ansible_host" render-direct-artifacts/ansible/inventory.yml
# Should show: ansible_host: 192.168.1.200
```

#### Step 4: Verify No Environment Created

```bash
# List environments - render-direct should NOT exist
cargo run -- list

# Should NOT show 'render-direct' environment
```

#### Step 5: Cleanup

```bash
# Remove artifacts
rm -rf render-direct-artifacts
```

---

## ✅ Success Criteria

**Test 1 (Environment Name Mode)**:

- [x] ✅ Environment created successfully
- [x] ✅ Render command succeeds with valid IP
- [x] ✅ All 8 service artifacts generated in `build/render-test/`
- [x] ✅ Provision command succeeds
- [x] ✅ Rendered artifacts match provisioned artifacts (except IP addresses)
- [x] ✅ Only Ansible inventory shows IP difference (192.168.1.100 vs actual IP)
- [x] ✅ Idempotent - multiple renders succeed without errors

**Test 2 (Config File Mode)**:

- [x] ✅ Render from config file succeeds without creating environment
- [x] ✅ All artifacts generated in `./render-direct-artifacts/`
- [x] ✅ IP address correctly set in Ansible inventory
- [x] ✅ Environment NOT created in data directory

---

## 🐛 Common Issues and Solutions

### Issue: Command fails with "Environment not found"

**Cause**: Environment doesn't exist or is in wrong state

**Solution**:

```bash
# Check environment exists and is in Created state
cargo run -- show <env-name>

# If not in Created state, cannot render
```

### Issue: Output directory already exists

**Cause**: Target output directory from previous render

**Solution**:

```bash
# Remove old directory
rm -rf ./my-output-dir

# Or use --force to overwrite
cargo run -- render --env-name test --instance-ip 192.168.1.100 --output-dir ./my-output-dir --force
```

### Issue: IP validation error

**Cause**: Invalid IP format provided

**Solution**:

```bash
# Use valid IPv4 or IPv6 format
cargo run -- render --env-name test --instance-ip 192.168.1.100 --output-dir ./test-preview  # Valid
cargo run -- render --env-name test --instance-ip invalid-ip --output-dir ./test-preview     # Invalid
```

### Issue: Render shows provisioned environment message

**Cause**: Environment already provisioned (not in Created state)

**Expected behavior**: Render command should inform you where existing artifacts are located

**Solution**: This is informational, not an error. The artifacts are already in `build/<env-name>/` from the provision command.

---

## 📊 Verification Checklist

After completing both tests, verify:

### Artifact Completeness

- [ ] `tofu/` directory contains infrastructure code
- [ ] `ansible/` directory contains playbooks and inventory
- [ ] `docker-compose/` directory contains service definitions
- [ ] `tracker/` directory contains tracker.toml
- [ ] `prometheus/` directory contains prometheus.yml
- [ ] `grafana/` directory contains provisioning configs
- [ ] `caddy/` directory exists (if HTTPS enabled)
- [ ] `backup/` directory exists (if backup enabled)

### Artifact Equivalence

- [ ] OpenTofu configs identical between render and provision
- [ ] Docker Compose files identical between render and provision
- [ ] Ansible playbooks identical between render and provision
- [ ] Only Ansible inventory IP differs (test IP vs actual IP)
- [ ] Tracker configuration identical
- [ ] Prometheus configuration identical
- [ ] Grafana provisioning identical

### Command Behavior

- [ ] Render succeeds with `--env-name` for Created environments
- [ ] Render succeeds with `--env-file` without creating environment
- [ ] Render is idempotent (can be called multiple times)
- [ ] Render provides clear success messages
- [ ] Render validates IP address format
- [ ] Render fails gracefully for non-existent environments
- [ ] Render fails gracefully for missing config files

---

## 🔗 Related Documentation

- [Complete Manual Testing Guide](README.md) - Full deployment workflow testing
- [Render Command Reference](../../user-guide/commands/render.md) - Command documentation
- [E2E Testing Overview](../README.md) - Automated E2E test documentation
- [Troubleshooting Guide](../troubleshooting.md) - Common issues and solutions

---

## 📝 Notes

- **IP Addresses**: The render command requires an IP address because:
  - In Created state, infrastructure doesn't exist yet
  - With config file, no infrastructure is ever created
  - IP is needed for Ansible inventory generation

- **Output Directory**: Artifacts are generated in the user-specified output directory (via `--output-dir` flag). This separates preview artifacts from deployment artifacts in `build/<env-name>/`.

- **State Independence**: Render is a **read-only** operation - it never changes environment state

- **Artifact Parity**: This test is critical for ensuring the render command is a true "preview" of what provision will generate
