# E2E Tests Split: Provision vs Configuration

## Summary

### Problem

The current E2E tests are failing on GitHub Actions runners due to network connectivity issues within LXD virtual machines. After successful VM provisioning, the VMs cannot install dependencies because they lack network connectivity in the GitHub Actions environment.

**Related Issues:**

- [GitHub Actions Runner Images Issue #13003](https://github.com/actions/runner-images/issues/13003) - Network connectivity issues with LXD VMs on GitHub runners
- [Reproduction Repository](https://github.com/josecelano/test-docker-install-inside-vm-in-runner) - Test repository demonstrating the network connectivity issues
- [Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support) - Comprehensive testing of virtualization tools on GitHub Actions, demonstrating Docker feasibility
- [Original Virtualization Investigation](https://github.com/actions/runner-images/issues/12933) - Background context on GitHub Actions virtualization support

### Current Deployment Phases

Our deployment workflow consists of these sequential phases:

1. **Provision** - Create infrastructure (VMs/containers) using OpenTofu/LXD
2. **Configure** - Install and configure software using Ansible
3. **Release** - Deploy application artifacts
4. **Run** - Start and validate running services

Currently, all phases are tested together in a single E2E test suite, which fails due to the network connectivity issue in phase 2 (Configure).

## Solution

Split the E2E testing into two independent test suites:

### 1. E2E Provision Tests (`e2e-provision`)

- **Scope**: Test only the provisioning phase
- **Technology**: Continue using LXD VMs via GitHub Actions
- **Coverage**:
  - VM/container creation
  - Cloud-init completion
  - Basic infrastructure validation
- **Success Criteria**: VM is created and cloud-init has finished successfully

### 2. E2E Configuration Tests (`e2e-config`)

- **Scope**: Test configuration, release, and run phases
- **Technology**: Use Docker containers instead of VMs (proven feasible per [virtualization research](https://github.com/josecelano/github-actions-virtualization-support))
- **Coverage**:
  - Ansible playbook execution
  - Software installation (Docker, Docker Compose, etc.)
  - Application deployment
  - Service validation
- **Success Criteria**: All software is installed and services are running correctly

### Benefits

1. **Reliability**: Provision tests continue working on GitHub Actions
2. **Speed**: Configuration tests run faster in Docker containers
3. **Isolation**: Issues in one test suite don't block the other
4. **Maintainability**: Each test suite has a single, focused responsibility
5. **Debugging**: Easier to identify whether issues are in provisioning or configuration

## Implementation Plan

### Phase A: Create E2E Provision Tests ✅ COMPLETED

#### A.1: Define naming and structure

- [x] **Task**: Define binary and workflow names
  - Binary: `e2e-provision-tests`
  - Workflow: `.github/workflows/test-e2e-provision.yml`
  - Purpose: Test infrastructure provisioning only

#### A.2: Create provision-only workflow

- [x] **Task**: Create `.github/workflows/test-e2e-provision.yml`
  - Copy structure from existing `test-e2e.yml`
  - Use `cargo run --bin e2e-provision-tests`
  - Keep all LXD/OpenTofu setup steps
  - Remove Ansible installation (not needed for provision-only tests)

#### A.3: Create provision-only binary

- [x] **Task**: Create `src/bin/e2e_provision_tests.rs`
  - Copy code from `src/bin/e2e_tests.rs`
  - Remove `configure_infrastructure` call in `run_full_deployment_test()`
  - Focus only on:
    - `cleanup_lingering_resources()`
    - `provision_infrastructure()`
    - Basic validation that VM is created and cloud-init completed
    - `cleanup_infrastructure()`

#### A.4: Update provision test validation

- [x] **Task**: Modify validation logic in provision tests
  - Check VM/container exists and is running
  - Verify cloud-init has completed successfully
  - Validate basic network interface setup
  - Skip application-level validations

#### A.5: Test and commit provision workflow

- [x] **Task**: Verify provision-only workflow works
  - Test locally: `cargo run --bin e2e-provision-tests`
  - Commit changes with conventional commit format
  - Verify new GitHub workflow passes
  - Update workflow status badges in README if needed

#### Phase A Results Summary

✅ **Successfully Completed** (September 2025)

**Implementation Details:**

- Created `src/bin/e2e_provision_tests.rs` - provision-only E2E test binary
- Created `.github/workflows/test-e2e-provision.yml` - GitHub Actions workflow for provision testing
- Updated `Cargo.toml` with new binary configuration
- Added `.cargo/config.toml` alias: `e2e-provision = "run --bin e2e-provision-tests"`

**Test Results:**

- Local execution time: ~29 seconds (significantly faster than full E2E tests)
- Successfully creates LXD VM, validates cloud-init completion, and cleans up resources
- Focuses solely on infrastructure provisioning without Ansible configuration
- All linting and testing checks pass

**Key Benefits Achieved:**

- Isolated infrastructure provisioning testing from configuration issues
- Faster feedback for provisioning-related changes
- Clear separation of concerns between infrastructure and configuration testing
- Reduced dependency on network connectivity within VMs for basic provisioning validation

### Phase B: Create E2E Configuration Tests

#### B.1: Research Docker container approach

- [x] **Task**: Design Docker-based test environment
  - **Reference**: Use proven approach from [virtualization support research](https://github.com/josecelano/github-actions-virtualization-support)
  - **Additional Reference**: [Docker-in-VM testing](https://github.com/josecelano/test-docker-install-inside-vm-in-runner) - Research on Docker installation within VMs documenting network connectivity issues
  - Create Ubuntu 24.04 base container configuration
  - Investigate cloud-init support in Docker (or alternative initialization)
  - Research testcontainers integration for Rust
  - Document container networking requirements for Ansible
  - **Research Documentation**: See `docs/research/e2e-docker-config-testing.md` for detailed analysis
  - **Advantage**: Docker is well-established and reliable on GitHub Actions

#### B.1 Results Summary

✅ **Successfully Completed** (September 2025)

**Implementation Details:**

- Created comprehensive research document at `docs/research/e2e-docker-config-testing.md`
- Documented Docker container requirements for Ansible testing
- Analyzed Ubuntu 24.04 base image strategy with SSH server and systemd configuration
- Researched cloud-init alternatives for container initialization
- Evaluated testcontainers-rs integration options vs direct Docker CLI approach
- Documented network configuration requirements for Ansible connectivity

**Key Findings:**

- Docker containers are well-established and reliable on GitHub Actions (no nested virtualization issues)
- Need systemd-in-Docker configuration for service management (Docker daemon, etc.)
- SSH server setup required for Ansible connectivity with proper user/sudo access
- Testcontainers-rs provides good integration options but direct Docker CLI is simpler for initial implementation
- Container networking provides consistent internet access for package downloads

**Research References:**

- [Virtualization Support Research](https://github.com/josecelano/github-actions-virtualization-support) - Comprehensive virtualization tools testing
- [Docker-in-VM Testing](https://github.com/josecelano/test-docker-install-inside-vm-in-runner) - Specific network connectivity issue documentation

#### B.2: Create Docker configuration ✅ COMPLETED

- [x] **Task**: Create `docker/provisioned-instance/Dockerfile`
  - Ubuntu 24.04 base image ✅
  - Supervisor for process management (replaced systemd) ✅
  - SSH server configuration for Ansible connectivity ✅
  - Password authentication with SSH key support ✅
  - Network configuration for container accessibility ✅
  - Required system dependencies ✅

**Architecture Decision**: Named `provisioned-instance` to represent the deployment phase state:

- **Current**: `docker/provisioned-instance/` - Post-provision, pre-configuration state
- **Future**: `docker/configured-instance/` - Post-configuration state
- **Future**: `docker/released-instance/` - Post-application deployment state

**Implementation Notes**:

- Used supervisor instead of systemd for container-friendly process management
- Password authentication (`torrust:torrust123`) for initial access
- E2E tests copy SSH public key during setup phase
- No privileged mode required

#### B.3: Create configuration-only binary with testcontainers ✅ COMPLETED

- [x] **Task**: Create `src/bin/e2e_config_tests.rs` with testcontainers integration
  - Copy code from original `src/bin/e2e_tests.rs` (before provision-only changes)
  - Replace LXD VM provisioning with Docker container setup using testcontainers
  - Implement Docker container lifecycle management via testcontainers-rs
  - Keep all configuration, release, and run phase testing
  - Update infrastructure cleanup to handle Docker containers
  - Add `testcontainers` crate dependency with blocking features
  - Implement container management through testcontainers API for reliable cleanup

**Implementation Details:**

- Created `src/bin/e2e_config_tests.rs` with complete Docker-based E2E configuration testing
- Implemented `src/e2e/provisioned_container.rs` using testcontainers for container lifecycle management
- Added testcontainers v0.25 dependency with blocking features for synchronous container operations
- Integrated SSH key authentication via docker exec for Ansible connectivity
- Fixed container port mapping (22:22) for simplified SSH access
- Enhanced Docker image with python3-apt for Ansible APT operations
- Implemented container detection in Ansible templates to skip systemd operations
- Disabled cloud-init validation for container-based testing
- Added comprehensive logging and error handling throughout the workflow

**Key Achievements:**

- Complete Docker-based E2E testing infrastructure replacing LXD VMs
- Working SSH authentication and Ansible connectivity to containers
- Successful Docker and Docker Compose installation via Ansible playbooks
- Proper container cleanup via testcontainers automatic management
- All configuration tests passing with verified software installations

#### B.4: Test configuration workflow locally ✅ COMPLETED

- [x] **Task**: Validate configuration tests work locally
  - Test: `cargo run --bin e2e-config-tests` ✅
  - Verify container creation and networking ✅
  - Validate Ansible connectivity to container ✅
  - Confirm all configuration/release/run phases complete ✅
  - Test cleanup procedures ✅
  - Verify Docker and Docker Compose installations work correctly ✅

**Validation Results:**

- Local test execution time: ~30 seconds for complete configuration workflow
- Container networking: SSH connectivity on port 22 working correctly
- Ansible playbook execution: Docker and Docker Compose installed successfully
- Software verification: Both `docker --version` and `docker-compose --version` confirmed working
- Container cleanup: Testcontainers automatically removes containers after test completion
- All validation steps pass including Docker daemon functionality tests

#### B.5: Create configuration workflow ✅ COMPLETED

- [x] **Task**: Create `.github/workflows/test-e2e-config.yml`
  - Remove LXD/OpenTofu setup steps ✅
  - Keep Ansible installation ✅
  - Add Docker setup if needed ✅
  - Use `cargo run --bin e2e-config-tests` ✅
  - Configure appropriate timeout limits ✅

**Implementation Details:**

- Created `.github/workflows/test-e2e-config.yml` - GitHub Actions workflow for configuration testing
- Removed LXD and OpenTofu installation steps (not needed for Docker-based testing)
- Retained Ansible installation via `./scripts/setup/install-ansible.sh`
- Added Docker setup using `docker/setup-buildx-action@v3`
- Updated verification steps to check `docker --version` and `ansible --version`
- Used `cargo run --bin e2e-config-tests` for the E2E configuration test execution
- Set timeout to 45 minutes for complete software installation testing
- Implemented Docker container lifecycle management in cleanup steps
- Updated container image references to match `torrust-provisioned-instance` from Rust code
- Added comprehensive debugging and cleanup steps for Docker containers
- Followed GitHub Actions conventions with proper error handling and logging

**Key Achievements:**

- Complete GitHub Actions workflow for Docker-based E2E configuration testing
- Proper container cleanup using correct image names (`torrust-provisioned-instance`)
- Removed infrastructure dependencies (LXD/OpenTofu) while keeping configuration tools (Ansible)
- All YAML linting checks pass with project's `.yamllint-ci.yml` configuration
- Workflow follows same structure and conventions as provision workflow for consistency

#### B.6: Test and commit configuration workflow

- [ ] **Task**: Verify configuration workflow on GitHub Actions
  - Commit configuration test changes
  - Verify new GitHub workflow passes
  - Test that Docker containers work correctly in GitHub Actions
  - Validate all software installation steps complete

### Phase C: Integration and Documentation

#### C.1: Update documentation

- [ ] **Task**: Update relevant documentation
  - Update `docs/e2e-testing.md` to reflect new split approach
  - Document how to run each test suite independently
  - Update `README.md` workflow badges for both test suites
  - Add troubleshooting guide for each test type

#### C.2: Remove legacy workflow

- [ ] **Task**: Remove original E2E workflow
  - Remove `.github/workflows/test-e2e.yml` entirely
  - Update any CI dependencies or status checks that reference the old workflow
  - Update workflow status badges in README to point to new split workflows

#### C.3: Rename old binary for local development

- [ ] **Task**: Rename `src/bin/e2e_tests.rs` for local use
  - Rename to `src/bin/e2e_tests_full.rs` (or similar descriptive name)
  - Update `Cargo.toml` binary configuration to match new name
  - Add documentation comment explaining it's for local development only
  - Update any related documentation to clarify this is for local testing
  - Note: This binary cannot run on GitHub Actions due to network connectivity issues but remains useful for local full deployment testing

#### C.4: Validate complete solution

- [ ] **Task**: End-to-end validation
  - Verify both test suites pass independently
  - Test that they can run in parallel without conflicts
  - Validate comprehensive coverage across all deployment phases
  - Confirm GitHub Actions reliability improvements

## Success Criteria

1. **Provision Tests**: Consistently pass on GitHub Actions, testing VM creation and cloud-init
2. **Configuration Tests**: Consistently pass on GitHub Actions, testing software installation and deployment
3. **Independence**: Each test suite can run independently without interference
4. **Coverage**: Combined test suites provide equivalent or better coverage than original tests
5. **Performance**: Overall test execution time is equal or improved
6. **Maintainability**: Clear separation of concerns makes debugging and maintenance easier

## Risks and Mitigations

### Risk: Docker environment differs from LXD VMs

- **Mitigation**: Carefully configure Docker container to match LXD VM environment
- **Validation**: Cross-reference configurations between Docker and LXD templates

### Risk: Testcontainers adds complexity

- **Mitigation**: Start with direct Docker approach, only add testcontainers if clearly beneficial
- **Fallback**: Direct Docker CLI integration is simpler and well-documented

### Risk: Loss of end-to-end coverage

- **Mitigation**: Ensure that provision tests validate infrastructure is ready for configuration
- **Validation**: Document the interface contract between provision and configuration phases

### Risk: Increased maintenance burden

- **Mitigation**: Share common code between test suites through library modules
- **Best Practice**: Keep test configurations as similar as possible between suites
