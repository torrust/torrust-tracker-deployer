# E2E Testing Troubleshooting

This guide helps you debug common issues with E2E tests and provides cleanup procedures.

## 🧹 Test Environment Cleanup

### Infrastructure Tests Cleanup

If infrastructure lifecycle tests fail and leave LXD resources behind:

```bash
# Check running containers
lxc list

# Stop and delete the test container
lxc stop torrust-tracker-vm
lxc delete torrust-tracker-vm

# Or use OpenTofu to clean up
cd build/tofu/lxd
tofu destroy -auto-approve
```

### Deployment Workflow Tests Cleanup

If deployment workflow tests fail and leave Docker resources behind:

```bash
# Check running containers
docker ps -a

# Stop and remove test containers
docker stop $(docker ps -q --filter "ancestor=torrust-provisioned-instance")
docker rm $(docker ps -aq --filter "ancestor=torrust-provisioned-instance")

# Remove test images if needed
docker rmi torrust-provisioned-instance
```

## 🐛 Common Issues by Test Suite

### Infrastructure Lifecycle Tests Issues

**LXD daemon not running**:

```bash
sudo systemctl start lxd
```

**Insufficient privileges**:

- Ensure your user is in the `lxd` group
- May need to log out and back in after adding to group

**OpenTofu state corruption**:

```bash
# Delete corrupted state and retry
rm build/tofu/lxd/terraform.tfstate
cargo run --bin e2e-infrastructure-lifecycle-tests
```

**Cloud-init timeout**:

- VM may need more time to complete initialization
- Check cloud-init status manually:

```bash
lxc exec torrust-tracker-vm -- cloud-init status
```

### Deployment Workflow Tests Issues

**Docker daemon not running**:

```bash
sudo systemctl start docker
```

**Container build failures**:

- Check Docker image build logs
- Ensure Dockerfile syntax is correct
- Verify base image is accessible

**SSH connectivity to container**:

- Verify container networking is functional
- Check SSH service is running in container
- Validate SSH key permissions (should be 600)

**Ansible connection errors**:

- Check container SSH configuration
- Verify Ansible inventory has correct IP/port
- Ensure SSH key matches between test and container

### Complete Workflow Tests Issues

**Network connectivity in VMs**:

- This is a known limitation on GitHub Actions
- Use split test suites for reliable testing in CI
- Complete workflow tests are for local use only

**SSH connectivity failures**:

- Usually means cloud-init is still running
- Wait for cloud-init to complete before SSH attempts
- Check SSH configuration hasn't failed during cloud-init

**Mixed infrastructure issues**:

- This test combines all provision and deployment issues
- Use split tests to isolate whether issue is in infrastructure or deployment
- Check both LXD and Docker logs

## 🔍 Debug Mode

Use the `--keep` flag to inspect the environment after test completion.

### Infrastructure Tests Debugging

```bash
cargo run --bin e2e-infrastructure-lifecycle-tests -- --keep

# After test completion, connect to the LXD container:
lxc exec torrust-tracker-vm -- /bin/bash
```

### Deployment Workflow Tests Debugging

```bash
cargo run --bin e2e-deployment-workflow-tests -- --keep

# After test completion, find and connect to the Docker container:
docker ps
docker exec -it <container-id> /bin/bash
```

### Complete Workflow Tests Debugging

```bash
cargo run --bin e2e-complete-workflow-tests -- --keep

# Connect to the LXD VM as above
lxc exec torrust-tracker-vm -- /bin/bash
```

## ⚙️ SSH Port Conflicts on GitHub Actions

**Problem**: GitHub Actions runners have SSH service running on port 22, which conflicts with test containers that also expose SSH on port 22.

**Root Cause**: When using Docker host networking (`--network host`), the container's SSH port 22 directly conflicts with the runner's SSH service on port 22.

**Solution**: Use Docker bridge networking (default) with dynamic port mapping:

- Container SSH port 22 is mapped to a random host port (e.g., 33061)
- The `register` command accepts an optional `--ssh-port` argument to specify the mapped port
- Ansible inventory is automatically updated with the custom SSH port

**Implementation**:

```bash
# E2E test discovers the mapped SSH port and passes it to register command
torrust-tracker-deployer register e2e-config --instance-ip 127.0.0.1 --ssh-port 33061
```

**Technical Details**: See [ADR: Register Command SSH Port Override](../decisions/register-ssh-port-override.md) for the complete architectural decision, implementation strategy, and alternatives considered.

This enhancement also supports real-world scenarios:

- Registering instances with non-standard SSH ports for security
- Working with containerized environments where port mapping is common
- Connecting to instances behind port-forwarding configurations

## 📝 Known Issues and Expected Behaviors

Some behaviors that appear as errors are actually expected. See [docs/contributing/known-issues.md](../contributing/known-issues.md) for:

- SSH host key warnings (red but normal in E2E tests)
- Expected stderr output that looks like errors but isn't
- Ansible warning messages that are safe to ignore

## 🆘 Getting Help

If you're still experiencing issues:

1. Check the project's GitHub Issues for similar problems
2. Review the [contributing guide](../contributing/README.md) for development setup
3. Consult the [logging guide](../contributing/logging-guide.md) for enabling detailed logs
4. Ask in project discussions or open a new issue with full context
