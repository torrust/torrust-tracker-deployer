# Advanced E2E Testing Techniques

This guide covers advanced testing techniques and workflows for experienced users.

## 🧪 Manual E2E Testing with Cross-Environment Registration

When manually testing the `register` command or the deployment pipeline, you can use a cross-environment technique that avoids manually provisioning VMs.

### The Technique

Use the deployer to provision one environment, then register that VM with a second environment:

```bash
# 1. Create and provision the first environment (owns the VM)
torrust-tracker-deployer --working-dir envs create environment --env-file envs/env-01.json
torrust-tracker-deployer --working-dir envs provision env-01

# 2. Get the instance IP from env-01
cat envs/data/env-01/environment.json | grep instance_ip
# Example output: "instance_ip": "10.140.190.186"

# 3. Create the second environment and register it with env-01's VM
torrust-tracker-deployer --working-dir envs create environment --env-file envs/env-02.json
torrust-tracker-deployer --working-dir envs register env-02 --instance-ip 10.140.190.186

# 4. Test the register workflow (configure, test, destroy)
torrust-tracker-deployer --working-dir envs configure env-02
torrust-tracker-deployer --working-dir envs test env-02
torrust-tracker-deployer --working-dir envs destroy env-02  # VM preserved!

# 5. Clean up the actual VM
torrust-tracker-deployer --working-dir envs destroy env-01  # VM destroyed
```

### Why This Works

- **env-01** has `provision_method: null` (or `Provisioned`) → destroy removes the VM
- **env-02** has `provision_method: Registered` → destroy preserves the VM

### Use Cases

This technique is useful for:

- **Testing register command**: Without needing external infrastructure
- **Verifying destroy behavior**: Confirming registered infrastructure is preserved
- **Testing deployment pipeline**: On registered environments
- **Rapid iteration**: Reuse same VM across multiple test cycles
- **Resource efficiency**: Avoid repeated VM provisioning during development

### Advanced Patterns

#### Multiple Registered Environments

You can register multiple environments to the same VM:

```bash
# Provision one VM
torrust-tracker-deployer provision env-01

# Register multiple test environments to it
torrust-tracker-deployer register env-test-a --instance-ip 10.140.190.186
torrust-tracker-deployer register env-test-b --instance-ip 10.140.190.186
torrust-tracker-deployer register env-test-c --instance-ip 10.140.190.186

# Test different configurations on same VM
torrust-tracker-deployer configure env-test-a
torrust-tracker-deployer configure env-test-b  # Different config
torrust-tracker-deployer configure env-test-c  # Another config

# Clean up all test environments (VM preserved)
torrust-tracker-deployer destroy env-test-a
torrust-tracker-deployer destroy env-test-b
torrust-tracker-deployer destroy env-test-c

# Finally destroy the VM
torrust-tracker-deployer destroy env-01
```

#### Non-Standard SSH Ports

Test with custom SSH ports:

```bash
# Register with custom SSH port
torrust-tracker-deployer register env-test \
    --instance-ip 10.140.190.186 \
    --ssh-port 2222

# All subsequent commands use the custom port automatically
torrust-tracker-deployer configure env-test
torrust-tracker-deployer test env-test
```

## 🔧 Custom Template Testing

Test custom templates without modifying the main template directory:

```bash
# Copy templates to a custom location
cp -r templates/ /tmp/my-custom-templates/

# Modify templates as needed
vim /tmp/my-custom-templates/ansible/playbooks/install-docker.yml

# Run tests with custom templates
cargo run --bin e2e-deployment-workflow-tests -- \
    --templates-dir /tmp/my-custom-templates
```

## 🐛 Advanced Debugging Techniques

### Inspect Container State During Execution

Use `--keep` flag and connect while tests are paused:

```bash
# Terminal 1: Run test with keep flag
cargo run --bin e2e-deployment-workflow-tests -- --keep

# Terminal 2: While test is running, find container
docker ps

# Terminal 3: Connect and inspect
docker exec -it <container-id> /bin/bash

# Inside container: check logs, validate state, etc.
journalctl -u docker
cat /var/log/cloud-init-output.log
```

### LXD VM Snapshots for Debugging

Create snapshots at specific test stages:

```bash
# During test execution, create snapshot
lxc snapshot torrust-tracker-vm pre-configure

# If test fails, restore to snapshot
lxc restore torrust-tracker-vm pre-configure

# Manually test the failing step
lxc exec torrust-tracker-vm -- /bin/bash
```

### Ansible Verbose Output

Enable verbose Ansible output for debugging:

```bash
# Set environment variable before running tests
export ANSIBLE_VERBOSITY=3
cargo run --bin e2e-deployment-workflow-tests
```

## 📊 Performance Analysis

### Measure Test Execution Time

```bash
# Time complete test run
time cargo run --bin e2e-complete-workflow-tests

# Time individual phases
time cargo run --bin e2e-infrastructure-lifecycle-tests
time cargo run --bin e2e-deployment-workflow-tests
```

### Profile Resource Usage

```bash
# Monitor system resources during test
docker stats  # For deployment workflow tests
lxc info torrust-tracker-vm  # For infrastructure tests
```

## 🔄 Continuous Integration Testing

### Local CI Simulation

Simulate GitHub Actions environment locally:

```bash
# Use act to run GitHub Actions locally
act -j test-e2e-infrastructure
act -j test-e2e-deployment
```

### Parallel Test Execution

Run independent test suites in parallel:

```bash
# Terminal 1
cargo run --bin e2e-infrastructure-lifecycle-tests

# Terminal 2 (can run simultaneously)
cargo run --bin e2e-deployment-workflow-tests
```

## 🎯 Best Practices

1. **Use split tests for CI**: Always use infrastructure and deployment tests separately in CI
2. **Complete tests locally**: Run complete workflow tests before submitting PRs
3. **Debug with --keep**: Always use `--keep` flag when debugging failed tests
4. **Custom templates**: Test template changes with `--templates-dir` before committing
5. **Cross-environment**: Use cross-environment registration for rapid iteration
6. **Snapshots**: Leverage LXD snapshots for complex debugging scenarios
7. **Cleanup**: Always clean up resources after manual testing

## 🔗 Related Documentation

- [Running Tests](running-tests.md) - Basic test execution
- [Troubleshooting](troubleshooting.md) - Common issues and fixes
- [Architecture](architecture.md) - Understanding the test architecture
- [Contributing](contributing.md) - Extending E2E tests
