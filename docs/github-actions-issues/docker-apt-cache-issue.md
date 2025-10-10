# Docker Installation APT Cache Issue

## Problem Description

When running the E2E configuration tests in GitHub Actions, the Ansible playbook fails to install Docker with the error "No package matching 'docker.io' is available". This occurs despite Docker being a standard Ubuntu package.

## Error Output

```text
2025-09-24T06:46:09.136842Z  INFO e2e_config_tests: status: "expected_failure", error: Command execution failed: Command 'ansible-playbook -v install-docker.yml' failed with exit code 2
Stdout: Using /home/runner/work/torrust-tracker-deployer/torrust-tracker-deployer/build/ansible/ansible.cfg as config file

PLAY [Install Docker (Simplified for E2E)] *************************************

TASK [Gathering Facts] *********************************************************
ok: [torrust-tracker-vm]

TASK [🐳 Starting simplified Docker installation] ******************************
ok: [torrust-tracker-vm] => {}

MSG:

🚀 Installing Docker CE via Ubuntu repositories on torrust-tracker-vm

TASK [Update apt cache] ********************************************************
ok: [torrust-tracker-vm] => {
    "cache_update_time": 1758696352,
    "cache_updated": false,
    "changed": false
}

TASK [Install Docker from Ubuntu repositories] *********************************
task path: /home/runner/work/torrust-tracker-deployer/torrust-tracker-deployer/build/ansible/install-docker.yml:26
fatal: [torrust-tracker-vm]: FAILED! => {
    "changed": false
}

MSG:

No package matching 'docker.io' is available

PLAY RECAP *********************************************************************
torrust-tracker-vm         : ok=3    changed=0    unreachable=0    failed=1    skipped=0    rescued=0    ignored=0
```

## Root Cause

GitHub Actions containers can have stale APT package caches or incomplete repository configurations. The issue manifests in several ways:

1. **Stale APT Cache**: The `cache_valid_time: 3600` parameter prevents cache updates if the cache appears recent, but the cache might be stale or incomplete in containerized environments.

2. **Container Environment Differences**: GitHub Actions runners may have different repository configurations or missing universe/multiverse repositories compared to standard Ubuntu installations.

3. **APT Backend Issues**: Using the default APT backend in containerized environments can sometimes fail to properly update package lists.

## Solution

The fix involves forcing APT cache updates and using more robust APT options in containerized environments:

1. **Remove cache validity constraints** to ensure fresh package list retrieval
2. **Add `force_apt_get` flag** to bypass potential APT caching issues
3. **Add debugging information** to troubleshoot repository availability
4. **Ensure proper repository configuration** verification

## Code Changes

The fix was implemented in `templates/ansible/install-docker.yml`:

```diff
-    - name: Update apt cache
+    - name: Force update apt cache for container environment
       ansible.builtin.apt:
         update_cache: true
-        cache_valid_time: 3600
+        force_apt_get: true
       when: ansible_os_family == "Debian"

+    - name: Ensure universe repository is available
+      ansible.builtin.shell: |
+        apt-cache policy docker.io || echo "docker.io not found"
+        apt list --installed | grep -E "(universe|multiverse)" | head -5 || echo "No universe/multiverse packages found"
+      register: repo_check
+      changed_when: false
+
+    - name: Display repository check results
+      ansible.builtin.debug:
+        var: repo_check.stdout_lines
+
     - name: Install Docker from Ubuntu repositories
       ansible.builtin.apt:
         name:
           - docker.io
         state: present
+        force_apt_get: true
       when: ansible_os_family == "Debian"
```

## Commit Reference

**Fixed in commit**: [`f697165`](https://github.com/torrust/torrust-tracker-deployer/commit/f697165f2580a2c450fb4cf26fb49457c583ff60)

**Commit message**: `fix: force apt cache update in Docker installation for GitHub Actions containers`

## Prevention

To prevent this issue in the future:

1. **Always use `force_apt_get: true`** in GitHub Actions environments when using Ansible apt module
2. **Remove `cache_valid_time`** constraints in CI environments where fresh package lists are critical
3. **Add debugging tasks** to verify repository availability before attempting package installation
4. **Test playbooks in containerized environments** that mirror GitHub Actions runners
5. **Consider using `update_cache: true` with `force_apt_get: true`** for all package installations in CI

## Related Issues

- Similar APT cache issues may occur with other packages in GitHub Actions environments
- This pattern should be applied to other Ansible playbooks that install packages in CI
