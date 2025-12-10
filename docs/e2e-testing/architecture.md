# E2E Testing Architecture

This document explains the architectural decisions behind the E2E testing system, including the split testing approach and Docker-based deployment workflow validation.

## 🏗️ Overall Architecture

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

## 🎯 Test Suite Responsibilities

- **Infrastructure Lifecycle Tests**: Infrastructure creation and basic VM setup validation
- **Deployment Workflow Tests**: Software installation and application deployment
- **Complete Workflow Tests**: End-to-end integration validation for comprehensive testing

This architecture provides:

1. **Reliability**: Each test suite works independently in CI environments
2. **Speed**: Focused testing reduces execution time
3. **Coverage**: Combined suites provide complete deployment validation
4. **Debugging**: Clear separation makes issue identification easier

## 🐳 Docker Architecture for Deployment Workflow Testing

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

See [ADR: Single Docker Image for Sequential E2E Command Testing](../decisions/single-docker-image-sequential-testing.md) for the complete architectural decision.

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

## 📊 Container vs VM Trade-offs

| Aspect                       | Docker Container                  | LXD VM                          |
| ---------------------------- | --------------------------------- | ------------------------------- |
| **Network Reliability (CI)** | ✅ Excellent                      | ❌ Poor (GitHub Actions issues) |
| **Startup Time**             | ✅ ~2-3 seconds                   | ⚠️ ~17-30 seconds               |
| **Production Similarity**    | ⚠️ Container (different from VMs) | ✅ Full VM (matches production) |
| **Resource Usage**           | ✅ Lightweight                    | ⚠️ Higher overhead              |
| **Best For**                 | Configuration/deployment workflow | Infrastructure provisioning     |

**Result**: Use Docker containers for deployment workflow tests, LXD VMs for infrastructure tests.

## 🔄 Why the Split Approach?

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

- **Infrastructure Lifecycle Tests**: Use LXD VMs for infrastructure testing only (no network-heavy operations inside VM)
- **Deployment Workflow Tests**: Use Docker containers which have reliable network connectivity on GitHub Actions
- **Complete Workflow Tests**: Available for comprehensive local testing where network connectivity works

**Implementation**: Deployment workflow tests use Docker containers with:

- Direct internet access for package downloads
- Reliable networking for Ansible connectivity
- No nested virtualization issues

## 🎯 Test Design Principles

- **Infrastructure tests**: Focus on infrastructure readiness, minimal network dependencies
- **Deployment tests**: Focus on software functionality, reliable network access via containers
- **Complete tests**: Comprehensive validation for development workflows
- **Independence**: Each suite should be runnable independently without conflicts

The split E2E testing approach ensures reliable CI while maintaining comprehensive coverage of the entire deployment pipeline.
