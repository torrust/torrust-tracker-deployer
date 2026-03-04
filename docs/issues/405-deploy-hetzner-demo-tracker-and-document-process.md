# Deploy Hetzner Demo Tracker and Document the Process

**Issue**: #405
**Parent Epic**: None
**Related**: [docs/user-guide/providers/hetzner.md](../user-guide/providers/hetzner.md), [docs/user-guide/quick-start/docker.md](../user-guide/quick-start/docker.md)

## Overview

Deploy a real Torrust Tracker demo instance to Hetzner Cloud using the deployer tool and document the entire process end-to-end. The documentation will serve two purposes:

1. **Internal reference**: A deployment journal under `docs/deployments/hetzner-demo-tracker/` capturing every step, decision, and problem encountered during a real deployment.
2. **Blog post source**: The documented process will be adapted into a blog post for the [torrust.com](https://torrust.com) blog.

## Domain Plan

**Main domain**: `torrust-tracker-demo.com`

Subdomains for individual services will be defined during the configuration phase.

## Goals

- [ ] Successfully deploy a Torrust Tracker demo instance to Hetzner Cloud
- [ ] Document every step of the deployment process with commands and outputs
- [ ] Capture all problems encountered with root causes and resolutions
- [ ] Produce documentation that can be adapted into a blog post

## Documentation Structure

A new `docs/deployments/` directory will be created for real-world deployment journals (distinct from the generic `docs/user-guide/` reference docs):

```text
docs/deployments/
└── hetzner-demo-tracker/
    ├── README.md              # Main deployment journal (step-by-step walkthrough)
    ├── prerequisites.md       # Account setup, API tokens, SSH keys, tool installation
    ├── configuration.md       # Environment config decisions and sanitized examples
    ├── problems.md            # Issues encountered, root causes, and solutions
    └── screenshots/           # Terminal output, Hetzner console, Grafana dashboards, etc.
```

### Why a separate `docs/deployments/` directory?

- **Different nature**: User guides are reference docs; deployment journals are narratives with decisions, context, and real problems.
- **Blog-post-ready**: The journal format maps directly to a blog post structure.
- **Reusable pattern**: Future deployments (different providers, configs) get their own subdirectory.
- **No duplication**: Links to existing docs (Hetzner provider, command reference, quick-start) instead of repeating them.

## Implementation Plan

### Phase 1: Setup and Prerequisites

- [x] Task 1.1: Create `docs/deployments/hetzner-demo-tracker/` directory structure
- [x] Task 1.2: Document prerequisites (Hetzner account, API token, SSH keys, tool versions)
- [x] Task 1.3: Verify all required tools are installed and working

### Phase 2: Create and Configure Environment

- [x] Task 2.1: Generate environment configuration template for Hetzner
- [x] Task 2.2: Document configuration decisions (server type, location, image, credentials)
- [x] Task 2.3: Create the environment using the deployer

### Phase 3: Deploy the Tracker

- [x] Task 3.1: Provision infrastructure (create Hetzner VM)
- [x] Task 3.2: Configure the instance (Docker, SSH, system setup)
- [x] Task 3.3: Release the application (deploy tracker files)
- [ ] Task 3.4: Run the services (start the tracker)

### Phase 3.5: Post-Provision Manual Setup

Steps required after provisioning and before running `configure`.
See [`docs/deployments/hetzner-demo-tracker/post-provision/`](../deployments/hetzner-demo-tracker/post-provision/README.md).

**DNS Setup** ([dns-setup.md](../deployments/hetzner-demo-tracker/post-provision/dns-setup.md)):

- [x] Task 3.5.1: Assign IPv4 floating IP (`116.202.176.169`) to the server in Hetzner Console
- [x] Task 3.5.2: Assign IPv6 floating IP (`2a01:4f8:1c0c:9aae::/64`) to the server in Hetzner Console
- [x] Task 3.5.3: Configure floating IPs permanently inside the VM (netplan)
- [x] Task 3.5.4: Create DNS records for all six subdomains via Hetzner Cloud API
- [x] Task 3.5.5: Verify all DNS records resolve correctly

**Volume Setup** ([volume-setup.md](../deployments/hetzner-demo-tracker/post-provision/volume-setup.md)):

- [x] Task 3.5.6: Create a 50 GB Hetzner volume (`torrust-tracker-demo-storage`) in `nbg1`
- [x] Task 3.5.7: Format the volume (`ext4`) and mount it at `/opt/torrust/storage`
- [x] Task 3.5.8: Add the volume to `/etc/fstab` for persistent mounting
- [x] Task 3.5.9: Verify volume is correctly mounted and writable

### Phase 4: Verify and Document

- [ ] Task 4.1: Verify tracker is accessible and functioning
- [ ] Task 4.2: Verify monitoring stack (Grafana, Prometheus)
- [ ] Task 4.3: Take screenshots of running services
- [ ] Task 4.4: Document any problems encountered during all phases

### Phase 5: Finalize Documentation

- [ ] Task 5.1: Write the main deployment journal (`README.md`)
- [ ] Task 5.2: Review and polish all documentation files
- [ ] Task 5.3: Update `docs/README.md` index with new deployments section

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] A Torrust Tracker demo instance is running on Hetzner Cloud
- [ ] `docs/deployments/hetzner-demo-tracker/README.md` contains a complete step-by-step walkthrough
- [ ] All problems encountered are documented in `problems.md` with resolutions
- [ ] Configuration examples are sanitized (no real secrets/tokens)
- [ ] Documentation links to existing user-guide docs where appropriate (no duplication)
- [ ] `docs/README.md` updated to reference the new deployments section

## Related Documentation

- [Hetzner Cloud Provider](../user-guide/providers/hetzner.md)
- [Quick Start: Docker Deployment](../user-guide/quick-start/docker.md)
- [Deployment Overview](../deployment-overview.md)
- [User Guide](../user-guide/README.md)

## Notes

- All secrets, API tokens, and passwords must be sanitized in the documentation. Use placeholders like `YOUR_HETZNER_API_TOKEN`.
- The blog post adaptation is out of scope for this issue — it will be done separately on the torrust.com repository.
- The demo tracker instance will incur Hetzner Cloud costs. Document the chosen server type and estimated monthly cost.
