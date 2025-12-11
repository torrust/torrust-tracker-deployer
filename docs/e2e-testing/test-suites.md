# E2E Test Suites

This document describes each E2E test suite in detail, including what they test and how they validate functionality.

## 📋 E2E Infrastructure Lifecycle Tests

**Binary**: `e2e-infrastructure-lifecycle-tests`

Tests the complete infrastructure lifecycle using LXD VMs.

### Test Sequence

1. **Preflight Cleanup**

   - Removes artifacts from previous test runs that may have failed to clean up

2. **Infrastructure Provisioning**

   - Uses OpenTofu configuration from `templates/tofu/lxd/`
   - Creates LXD container with Ubuntu and cloud-init configuration

3. **Cloud-init Completion**

   - Waits for cloud-init to finish system initialization
   - Validates user accounts and SSH key setup
   - Verifies basic network interface setup

4. **Infrastructure Destruction**
   - Destroys infrastructure using `DestroyCommand` (application layer)
   - Falls back to manual cleanup if `DestroyCommand` fails
   - Ensures proper resource cleanup regardless of test success or failure

### Validation

- ✅ VM is created and running
- ✅ Cloud-init status is "done"
- ✅ Boot completion marker file exists (`/var/lib/cloud/instance/boot-finished`)
- ✅ Infrastructure is properly destroyed after tests complete

### DestroyCommand Integration

The infrastructure lifecycle tests use the `DestroyCommand` from the application layer to test the complete infrastructure lifecycle. This provides:

- **Application Layer Testing**: Tests the actual command that users will execute
- **Idempotent Cleanup**: Destroy command can be run multiple times safely
- **Fallback Strategy**: Manual cleanup if destroy command fails (ensures CI reliability)

**Implementation**:

```rust
// Import destroy command from application layer
use torrust_tracker_deployer_lib::application::commands::destroy::DestroyCommand;

// Execute destroy via application command
async fn cleanup_with_destroy_command(
    environment: Environment<Provisioned>,
    opentofu_client: Arc<OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
) -> Result<(), DestroyCommandError> {
    let destroy_cmd = DestroyCommand::new(opentofu_client, repository);
    destroy_cmd.execute(environment)?;
    Ok(())
}
```

**Fallback Cleanup**:

If the `DestroyCommand` fails (e.g., due to infrastructure issues), the test suite falls back to manual cleanup:

```rust
// Try application layer destroy first
if let Err(e) = run_destroy_command(&context).await {
    error!("DestroyCommand failed: {}, falling back to manual cleanup", e);
    cleanup_test_infrastructure(&context).await?;
}
```

This ensures:

- CI tests always clean up resources
- Real-world destroy command is validated
- Infrastructure issues don't block CI

For detailed destroy command documentation, see:

- [Destroy Command User Guide](../user-guide/commands/destroy.md)
- [Destroy Command Developer Guide](../contributing/commands.md#destroycommand)

## 📋 E2E Deployment Workflow Tests

**Binary**: `e2e-deployment-workflow-tests`

Tests software installation and configuration using Docker containers.

### Test Sequence

1. **Container Setup**

   - Creates Docker container from `docker/provisioned-instance/`
   - Configures SSH connectivity for Ansible

2. **Software Installation** (`install-docker.yml`)

   - Installs Docker Community Edition
   - Configures Docker service
   - Validates Docker daemon is running

3. **Docker Compose Installation** (`install-docker-compose.yml`)
   - Installs Docker Compose binary
   - Validates installation with test configuration

### Validation

- ✅ Container is accessible via SSH
- ✅ Docker version command works
- ✅ Docker daemon service is active
- ✅ Docker Compose version command works
- ✅ Can parse and validate a test docker-compose.yml file

## 📋 E2E Complete Workflow Tests

**Binary**: `e2e-complete-workflow-tests`

Combines both provision and configuration phases in a single LXD VM for comprehensive local testing.

### Why Local Only?

This test cannot run on GitHub Actions due to network connectivity issues within LXD VMs on GitHub-hosted runners. See [architecture.md](architecture.md#-why-the-split-approach) for details about CI network limitations.

### When to Use

- Comprehensive local validation before submitting PRs
- Full integration testing of provision + deployment workflow
- Debugging complex issues that span infrastructure and deployment
- Final verification before releases
