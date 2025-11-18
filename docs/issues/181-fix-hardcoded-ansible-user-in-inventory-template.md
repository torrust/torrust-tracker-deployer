# Fix Hardcoded ansible_user in Inventory Template

**Issue**: #181
**Type**: Bug
**Priority**: High
**Affects**: Provision workflow, SSH connectivity

## Problem Description

The Ansible inventory template (`templates/ansible/inventory.yml.tera`) has a **hardcoded username** "torrust" for the `ansible_user` field, instead of using a template variable. This causes SSH authentication failures when users configure a different username in their environment configuration.

### Current Behavior

**Template** (`templates/ansible/inventory.yml.tera` line 73):

```yaml
ansible_user: torrust
```

**User Configuration** (`test-config.json`):

```json
{
  "ssh_credentials": {
    "username": "ubuntu" // User specifies "ubuntu"
  }
}
```

**Generated Inventory** (`build/test-env/ansible/inventory.yml`):

```yaml
ansible_user: torrust  // Still uses hardcoded "torrust"
```

**Result**: SSH authentication fails because Ansible tries to connect as "torrust" instead of "ubuntu".

### Error Symptoms

When provisioning with a non-"torrust" username:

```text
fatal: [torrust-tracker-vm]: UNREACHABLE! => {
    "changed": false,
    "unreachable": true
}

MSG:

Failed to connect to the host via ssh: ...
debug1: Authenticating to 10.140.190.20:22 as 'torrust'
...
torrust@10.140.190.20: Permission denied (publickey).
```

**Note**: The error shows `as 'torrust'` even though the user configured `"username": "ubuntu"` in their config file.

### Impact

- Users cannot use custom usernames (e.g., "ubuntu", "admin", "deploy")
- Provision workflow fails at SSH connectivity step
- Configuration in `test-config.json` is ignored
- Cloud-init correctly creates the user, but Ansible cannot connect

## Root Cause

**File**: `templates/ansible/inventory.yml.tera`  
**Line**: 73

The template uses a hardcoded value instead of a Tera variable:

```yaml
# ❌ CURRENT (WRONG):
ansible_user: torrust

# ✅ SHOULD BE:
ansible_user: {{ansible_user}}
```

### Affected Files

1. **Template**: `templates/ansible/inventory.yml.tera` (line 73)
2. **Renderer**: `src/infrastructure/external_tools/ansible/template/renderer/inventory.rs`
   - Must pass `ansible_user` variable to template context

## Expected Behavior

1. User specifies username in config: `"username": "ubuntu"`
2. Cloud-init creates user "ubuntu" with SSH key
3. Template renders with: `ansible_user: ubuntu`
4. Ansible connects successfully as "ubuntu"

## Solution

### Step 1: Update Template

**File**: `templates/ansible/inventory.yml.tera`

```yaml
# Before (line 73):
ansible_user: torrust

# After:
ansible_user: {{ansible_user}}
```

### Step 2: Update Renderer

**File**: `src/infrastructure/external_tools/ansible/template/renderer/inventory.rs`

Add `ansible_user` to the template context in the `render()` method:

```rust
context.insert("ansible_user", &environment.ssh_credentials().username());
```

**Location**: Find where other variables like `ansible_host`, `ansible_port`, `ansible_ssh_private_key_file` are inserted, and add `ansible_user` alongside them.

### Step 3: Update Documentation

**File**: `templates/ansible/inventory.yml.tera`

Update the comments around line 73 to reflect that this is a variable:

```yaml
# The username to use when connecting via SSH
# 🔗 LXD VM: This must match the user created in cloud-init
# 🔗 CONTAINER: This must match the pre-configured user in the container image
# 🔗 CONFIGURED: Set via ssh_credentials.username in environment config
ansible_user: { { ansible_user } }
```

## Verification Steps

After implementing the fix:

1. **Create test environment**:

   ```bash
   cat > test-config.json <<EOF
   {
     "environment": { "name": "test-custom-user" },
     "ssh_credentials": {
       "private_key_path": "fixtures/testing_rsa",
       "public_key_path": "fixtures/testing_rsa.pub",
       "username": "ubuntu",
       "port": 22
     }
   }
   EOF
   ```

2. **Create and provision**:

   ```bash
   torrust-tracker-deployer create environment -f test-config.json
   torrust-tracker-deployer provision test-custom-user
   ```

3. **Verify generated inventory**:

   ```bash
   cat build/test-custom-user/ansible/inventory.yml | grep ansible_user
   # Should show: ansible_user: ubuntu
   ```

4. **Verify SSH connectivity**:

   - Provision should complete successfully
   - Ansible should connect as "ubuntu"
   - No "Permission denied" errors

5. **Verify cloud-init alignment**:

   ```bash
   cat build/test-custom-user/tofu/lxd/cloud-init.yml | grep -A 2 "name: ubuntu"
   # Should show the ubuntu user configuration
   ```

## Acceptance Criteria

- [ ] Template uses `{{ansible_user}}` variable instead of hardcoded "torrust"
- [ ] Renderer passes `ansible_user` to template context
- [ ] Generated inventory uses the username from user's config file
- [ ] Provision workflow succeeds with custom usernames (e.g., "ubuntu")
- [ ] Cloud-init user matches Ansible user in generated files
- [ ] No SSH authentication errors due to username mismatch
- [ ] Documentation comments are updated
- [ ] Tests pass with both "torrust" (backward compatibility) and custom usernames

## Related Files

- `templates/ansible/inventory.yml.tera` - Template with hardcoded value
- `src/infrastructure/external_tools/ansible/template/renderer/inventory.rs` - Renderer logic
- `build/*/ansible/inventory.yml` - Generated inventory (verify after fix)
- `build/*/tofu/lxd/cloud-init.yml` - Cloud-init config (already correct)
- `data/*/environment.json` - Environment state (stores SSH credentials)

## Workaround (Temporary)

Until this is fixed, users **must** use username "torrust" in their configuration:

```json
{
  "ssh_credentials": {
    "username": "torrust" // ⚠️ REQUIRED - hardcoded in template
  }
}
```

Any other username will cause SSH authentication failures.

## Priority Justification

**High Priority** because:

- Blocks users from using standard usernames like "ubuntu"
- Causes confusing authentication failures
- Configuration appears to be accepted but is silently ignored
- Affects core provision workflow
- Simple fix with high user impact

## Estimated Effort

30 minutes to 1 hour:

- Update template variable: 5 min
- Update renderer context: 10 min
- Update documentation: 10 min
- Testing with multiple usernames: 15-30 min
