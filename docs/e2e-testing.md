# E2E Testing Guide

This guide explains how to run and understand the End-to-End (E2E) tests for the Torrust Tracker Deployer project.

## 🧪 What are E2E Tests?

The E2E tests validate the complete deployment process using two independent test suites:

1. **E2E Infrastructure Lifecycle Tests** - Test infrastructure provisioning and destruction lifecycle using LXD VMs
2. **E2E Deployment Workflow Tests** - Test software installation and configuration using Docker containers

This split approach ensures reliable testing in CI environments while maintaining comprehensive coverage.

## 🚀 Running E2E Tests

### Independent Test Suites

#### Infrastructure Lifecycle Tests

Test infrastructure provisioning and destruction lifecycle (VM creation, cloud-init, and destruction):

```bash
cargo run --bin e2e-infrastructure-lifecycle-tests
```

#### Deployment Workflow Tests

Test software installation, configuration, release, and run workflows (Ansible playbooks):

```bash
cargo run --bin e2e-deployment-workflow-tests
```

#### Full Local Testing

For local development, you can run the complete end-to-end test:

```bash
cargo run --bin e2e-complete-workflow-tests
```

⚠️ **Note**: The `e2e-complete-workflow-tests` binary cannot run on GitHub Actions due to network connectivity issues, but is useful for local validation.

### Command Line Options

All test binaries support these options:

- `--keep` - Keep the test environment after completion (useful for debugging)
- `--templates-dir` - Specify custom templates directory path
- `--help` - Show help information

### Examples

```bash
# Run infrastructure lifecycle tests
cargo run --bin e2e-infrastructure-lifecycle-tests

# Run infrastructure lifecycle tests with debugging (keep environment)
cargo run --bin e2e-infrastructure-lifecycle-tests -- --keep

# Run deployment workflow tests with debugging
cargo run --bin e2e-deployment-workflow-tests -- --keep

# Run full local tests with custom templates
cargo run --bin e2e-complete-workflow-tests -- --templates-dir ./custom/templates
```

## 📋 Test Sequences

### E2E Infrastructure Lifecycle Tests (`e2e-infrastructure-lifecycle-tests`)

Tests the complete infrastructure lifecycle using LXD VMs:

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

**Validation**:

- ✅ VM is created and running
- ✅ Cloud-init status is "done"
- ✅ Boot completion marker file exists (`/var/lib/cloud/instance/boot-finished`)
- ✅ Infrastructure is properly destroyed after tests complete

#### DestroyCommand Integration

The provision and destroy tests use the `DestroyCommand` from the application layer to test the complete infrastructure lifecycle. This provides:

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

- [Destroy Command User Guide](user-guide/commands/destroy.md)
- [Destroy Command Developer Guide](contributing/commands.md#destroycommand)

### E2E Deployment Workflow Tests (`e2e-deployment-workflow-tests`)

Tests software installation and configuration using Docker containers:

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

**Validation**:

- ✅ Container is accessible via SSH
- ✅ Docker version command works
- ✅ Docker daemon service is active
- ✅ Docker Compose version command works
- ✅ Can parse and validate a test docker-compose.yml file

### E2E Complete Workflow Tests (`e2e-complete-workflow-tests`)

Combines both provision and configuration phases in a single LXD VM for comprehensive local testing.

## 🛠️ Prerequisites

### Automated Setup (Recommended)

The project provides a dependency installer tool that automatically detects and installs required dependencies:

```bash
# Install all required dependencies
cargo run --bin dependency-installer install

# Check which dependencies are installed
cargo run --bin dependency-installer check

# List all dependencies with status
cargo run --bin dependency-installer list
```

The installer supports:

- **cargo-machete** - Detects unused Rust dependencies
- **OpenTofu** - Infrastructure provisioning tool
- **Ansible** - Configuration management tool
- **LXD** - VM-based testing infrastructure

For detailed information, see [`packages/dependency-installer/README.md`](../packages/dependency-installer/README.md).

### Manual Setup

If you prefer manual installation or need to troubleshoot:

#### For E2E Provision Tests

1. **LXD installed and configured**

   ```bash
   sudo snap install lxd
   sudo lxd init  # Follow the setup prompts
   ```

2. **OpenTofu installed**

   ```bash
   # Installation instructions in docs/tech-stack/opentofu.md
   ```

#### For E2E Configuration Tests

1. **Docker installed**

   ```bash
   # Docker is available on most systems or in CI environments
   docker --version
   ```

2. **Ansible installed**

   ```bash
   # Installation instructions in docs/tech-stack/ansible.md
   ```

#### For Full Local Tests (`e2e-tests-full`)

Requires **all** of the above: LXD, OpenTofu, Docker, and Ansible.

### Verification

After setup (automated or manual), verify all dependencies are available:

```bash
# Quick check (exit code indicates success/failure)
cargo run --bin dependency-installer check

# Detailed check with logging
cargo run --bin dependency-installer check --verbose
```

## 🐛 Troubleshooting

### Test Environment Cleanup

#### Provision Tests Cleanup

If provision tests fail and leave LXD resources behind:

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

#### Configuration Tests Cleanup

If configuration tests fail and leave Docker resources behind:

```bash
# Check running containers
docker ps -a

# Stop and remove test containers
docker stop $(docker ps -q --filter "ancestor=torrust-provisioned-instance")
docker rm $(docker ps -aq --filter "ancestor=torrust-provisioned-instance")

# Remove test images if needed
docker rmi torrust-provisioned-instance
```

### Common Issues by Test Suite

#### Provision Tests Issues

- **LXD daemon not running**: `sudo systemctl start lxd`
- **Insufficient privileges**: Ensure your user is in the `lxd` group
- **OpenTofu state corruption**: Delete `build/tofu/lxd/terraform.tfstate` and retry
- **Cloud-init timeout**: VM may need more time; check `lxc exec torrust-tracker-vm -- cloud-init status`

#### Configuration Tests Issues

- **Docker daemon not running**: `sudo systemctl start docker`
- **Container build failures**: Check Docker image build logs
- **SSH connectivity to container**: Verify container networking and SSH service
- **Ansible connection errors**: Check container SSH configuration and key permissions

#### Full Local Tests Issues

- **Network connectivity in VMs**: Known limitation - use split test suites for reliable testing
- **SSH connectivity failures**: Usually means cloud-init is still running or SSH configuration failed
- **Mixed infrastructure issues**: Combines all provision and configuration issues above

### Test Suite Selection Guide

**Use Provision Tests (`e2e-provision-tests`) when**:

- Testing infrastructure changes (OpenTofu, LXD configuration)
- Validating VM creation and cloud-init setup
- Working on provisioning-related features

**Use Configuration and Release Tests (`e2e-config-and-release-tests`) when**:

- Testing Ansible playbooks and software installation
- Validating configuration management changes
- Working on application deployment features

**Use Full Local Tests (`e2e-tests-full`) when**:

- Comprehensive local validation before CI
- Integration testing of provision + configuration
- Debugging end-to-end deployment issues

### CI Network Issues

**Problem**: GitHub Actions runners experience intermittent network connectivity problems within LXD VMs that cause:

- Docker GPG key downloads to fail (`Network is unreachable` errors)
- Package repository access timeouts
- Generally flaky network behavior

**Root Cause**: This is a known issue with GitHub-hosted runners:

- [GitHub Issue #13003](https://github.com/actions/runner-images/issues/13003) - Network connectivity issues with LXD VMs
- [GitHub Issue #1187](https://github.com/actions/runner-images/issues/1187) - Original networking issue
- [GitHub Issue #2890](https://github.com/actions/runner-images/issues/2890) - Specific apt repository timeout issues

**Solution**: We split E2E tests into two suites:

- **Provision Tests**: Use LXD VMs for infrastructure testing only (no network-heavy operations inside VM)
- **Configuration Tests**: Use Docker containers which have reliable network connectivity on GitHub Actions
- **Full Local Tests**: Available for comprehensive local testing where network connectivity works

**Implementation**: Configuration tests use Docker containers with:

- Direct internet access for package downloads
- Reliable networking for Ansible connectivity
- No nested virtualization issues

### SSH Port Conflicts on GitHub Actions

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

**Technical Details**: See [ADR: Register Command SSH Port Override](decisions/register-ssh-port-override.md) for the complete architectural decision, implementation strategy, and alternatives considered.

This enhancement also supports real-world scenarios:

- Registering instances with non-standard SSH ports for security
- Working with containerized environments where port mapping is common
- Connecting to instances behind port-forwarding configurations

### Debug Mode

Use the `--keep` flag to inspect the environment after test completion:

#### Provision Tests Debugging

```bash
cargo run --bin e2e-provision-tests -- --keep

# After test completion, connect to the LXD container:
lxc exec torrust-tracker-vm -- /bin/bash
```

#### Configuration and Release Tests Debugging

```bash
cargo run --bin e2e-config-and-release-tests -- --keep

# After test completion, find and connect to the Docker container:
docker ps
docker exec -it <container-id> /bin/bash
```

#### Full Local Tests Debugging

```bash
cargo run --bin e2e-tests-full -- --keep

# Connect to the LXD VM as above
lxc exec torrust-tracker-vm -- /bin/bash
```

## 🏗️ Architecture

The split E2E testing architecture ensures reliable CI while maintaining comprehensive coverage:

```text
┌───────────────────────────────────────────────────────────────────┐
│                        E2E Test Suites                            │
└─────┬────────────────┬──────────────────┬─────────────────────────┘
      │                │                  │
      │                │                  │
┌─────▼──────┐   ┌─────▼──────────┐   ┌───▼──────────────────┐
│ Provision  │   │Configuration   │   │    Full Local        │
│   Tests    │   │    Tests       │   │      Tests           │
│            │   │                │   │                      │
│ LXD VMs    │   │   Docker       │   │ LXD VMs + Docker     │
│ (CI Safe)  │   │ Containers     │   │ (Local Only)         │
│            │   │ (CI Safe)      │   │                      │
└─────┬──────┘   └───────┬────────┘   └───┬──────────────────┘
      │                  │                │
┌─────▼────────┐   ┌─────▼────────┐   ┌───▼──────────────────┐
│ OpenTofu/    │   │ Testcontain- │   │ OpenTofu + Ansible   │
│    LXD       │   │     ers      │   │    (Full Stack)      │
│Infrastructure│   │   Docker     │   │                      │
│   Layer      │   │ Management   │   │                      │
└──────────────┘   └──────────────┘   └──────────────────────┘
       │                │                         │
┌──────▼──────┐  ┌──────▼──────────┐    ┌─────────▼─────────┐
│ VM Creation │  │Ansible Playbooks│    │  Complete Stack   │
│ Cloud-init  │  │ Configuration   │    │    Validation     │
│ Validation  │  │   Validation    │    │                   │
└─────────────┘  └─────────────────┘    └───────────────────┘
```

### Test Suite Responsibilities

- **Provision Tests**: Infrastructure creation and basic VM setup validation
- **Configuration Tests**: Software installation and application deployment
- **Full Local Tests**: End-to-end integration validation for comprehensive testing

This architecture provides:

1. **Reliability**: Each test suite works independently in CI environments
2. **Speed**: Focused testing reduces execution time
3. **Coverage**: Combined suites provide complete deployment validation
4. **Debugging**: Clear separation makes issue identification easier

## 🐳 Docker Architecture for E2E Testing

The E2E testing system uses a Docker-based architecture for testing the deployment workflow commands (configure, release, run, test) efficiently and reliably in CI environments.

### Architecture Decision: Single Image with Sequential Command Execution

We use a **single Docker image** (`provisioned-instance`) representing the pre-provisioned state, and execute all deployment commands **sequentially** within that container during E2E tests.

**Why Sequential Instead of Multi-Image?**

Initially, we considered creating separate Docker images for each deployment phase (configured, released, running). However, this approach was **rejected** due to:

- **High Maintenance Overhead**: Every code change would require updating multiple Docker images
- **Slower Execution**: Building 4 images takes longer than running 4 commands sequentially
- **Synchronization Complexity**: Keeping multiple images in sync with code changes is error-prone
- **No Real Benefit**: Parallel test execution overhead (Docker build + startup) exceeds sequential execution time

**Sequential Execution Benefits**:

- ✅ **Single Source of Truth**: One Dockerfile to maintain
- ✅ **Faster Overall**: Sequential commands in one container (~48s) vs multiple image builds
- ✅ **Realistic Testing**: Matches real deployment workflow exactly
- ✅ **Easy Debugging**: Single container lifecycle with `--keep` flag
- ✅ **Automatic Synchronization**: Code changes tested via Ansible playbooks without image rebuilds

**Trade-offs Accepted**:

- ❌ Cannot test individual commands in isolation (use unit/integration tests for that)
- ❌ Cannot run E2E tests for different commands in parallel
- ❌ Must run full sequence to test later commands

See [ADR: Single Docker Image for Sequential E2E Command Testing](decisions/single-docker-image-sequential-testing.md) for the complete architectural decision.

### Current Implementation

#### Provisioned Instance (`docker/provisioned-instance/`)

**Purpose**: Represents the state after VM provisioning but before configuration.

**Contents**:

- Ubuntu 24.04 LTS base (matches production VMs)
- SSH server (via supervisor for container-native process management)
- `torrust` user with sudo access
- No application dependencies installed
- Ready for Ansible configuration

**E2E Test Workflow**:

```rust
// E2E deployment workflow tests (simplified)
async fn run_deployment_workflow_tests() -> Result<()> {
    // 1. Start single container (provisioned state)
    let container = start_provisioned_container().await?;

    // 2. Run deployment commands sequentially
    run_create_command()?;       // Create environment
    run_register_command()?;     // Register container IP
    run_configure_command()?;    // Install dependencies (modifies container)
    run_release_command()?;      // Deploy applications (modifies container)
    run_run_command()?;          // Start services (modifies container)
    run_test_command()?;         // Validate deployment

    // 3. Cleanup
    container.stop().await?;
    Ok(())
}
```

**Key Characteristics**:

- **Stateful Testing**: Each command modifies the container state for the next command
- **Complete Workflow**: Tests the full deployment pipeline end-to-end
- **Fast Execution**: ~48 seconds total (container start + all commands + validation)
- **CI Reliable**: Avoids GitHub Actions connectivity issues with LXD VMs

### Benefits of Single-Image Sequential Architecture

1. **Low Maintenance**: Single Dockerfile, changes propagate automatically via playbooks
2. **Realistic Testing**: Sequential execution matches real deployment workflow exactly
3. **Fast Feedback**: Faster than building multiple images, comparable to parallel execution
4. **Simple Debugging**: Use `--keep` flag to inspect final container state
5. **CI Reliability**: Single container uses fewer resources, avoids VM networking issues
6. **Code Synchronization**: Ansible playbooks ensure image reflects current code

### Testing Strategy

**What This Tests**:

- ✅ Complete deployment workflow (create → register → configure → release → run → test)
- ✅ Command integration and state transitions
- ✅ Ansible playbook execution in container environment
- ✅ Service deployment and validation

**What This Doesn't Test**:

- ❌ Individual command isolation (use unit tests)
- ❌ Infrastructure provisioning (use `e2e-infrastructure-lifecycle-tests`)
- ❌ VM-specific features (use `e2e-complete-workflow-tests` locally)

### Container vs VM Trade-offs

| Aspect                       | Docker Container                  | LXD VM                          |
| ---------------------------- | --------------------------------- | ------------------------------- |
| **Network Reliability (CI)** | ✅ Excellent                      | ❌ Poor (GitHub Actions issues) |
| **Startup Time**             | ✅ ~2-3 seconds                   | ⚠️ ~17-30 seconds               |
| **Production Similarity**    | ⚠️ Container (different from VMs) | ✅ Full VM (matches production) |
| **Resource Usage**           | ✅ Lightweight                    | ⚠️ Higher overhead              |
| **Best For**                 | Configuration/deployment workflow | Infrastructure provisioning     |

**Result**: Use Docker containers for deployment workflow tests, LXD VMs for infrastructure tests.

## 📝 Contributing to E2E Tests

When adding new features or making changes:

### Infrastructure Changes

For OpenTofu, LXD, or cloud-init modifications:


When adding new features or making changes:

### Infrastructure Changes

For OpenTofu, LXD, or cloud-init modifications:

1. **Update provision tests** in `src/bin/e2e_provision_tests.rs`
2. **Add validation methods** for new infrastructure components
3. **Test locally**: `cargo run --bin e2e-provision-tests`
4. **Verify CI passes** on `.github/workflows/test-e2e-provision.yml`

### Configuration Changes

For Ansible playbooks or software installation modifications:

1. **Update configuration tests** in `src/bin/e2e_config_tests.rs`
2. **Add validation methods** for new software components
3. **Update Docker image** in `docker/provisioned-instance/` if needed
4. **Test locally**: `cargo run --bin e2e-config-and-release-tests`
5. **Verify CI passes** on `.github/workflows/test-e2e-config.yml`

### End-to-End Integration

For comprehensive changes affecting multiple components:

1. **Test with full local suite**: `cargo run --bin e2e-tests-full`
2. **Verify both provision and configuration suites pass independently**
3. **Update this documentation** to reflect changes
4. **Consider split approach**: Can the change be tested in isolated suites?

### Test Design Principles

- **Provision tests**: Focus on infrastructure readiness, minimal network dependencies
- **Configuration tests**: Focus on software functionality, reliable network access via containers
- **Full local tests**: Comprehensive validation for development workflows
- **Independence**: Each suite should be runnable independently without conflicts

The split E2E testing approach ensures reliable CI while maintaining comprehensive coverage of the entire deployment pipeline.

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

This technique is useful for:

- Testing the `register` command without external infrastructure
- Verifying that `destroy` correctly preserves registered infrastructure
- Testing the full deployment pipeline on registered environments
