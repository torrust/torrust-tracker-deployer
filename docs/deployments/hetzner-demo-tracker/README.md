# Deployment Journal: Hetzner Demo Tracker

**Issue**: [#405](https://github.com/torrust/torrust-tracker-deployer/issues/405)
**Date started**: 2026-03-03
**Domain**: `torrust-tracker-demo.com`
**Provider**: Hetzner Cloud

## Purpose

Deploy a public Torrust Tracker demo instance to Hetzner Cloud and document every step of the process. This journal will serve as the source material for a blog post on [torrust.com](https://torrust.com).

## Table of Contents

1. [Prerequisites](prerequisites.md) — Account setup, tools, SSH keys
2. [Deployment Specification](deployment-spec.md) — What we want to deploy: config decisions,
   endpoints, sanitized config
3. Deployment commands — step-by-step per deployer command:
   - [create](commands/create/README.md) — generate template, validate, create environment
   - [provision](commands/provision/README.md) — create the Hetzner VM
4. Problems — issues encountered, per command:
   - [create problems](commands/create/problems.md)
   - [provision problems](commands/provision/problems.md)
5. Improvements — recommended deployer improvements found during this deployment:
   - [provision improvements](commands/provision/improvements.md)

## Deployment

> This section will be filled in as we execute each deployment phase.

### Phase 1: Setup and Prerequisites

See [prerequisites.md](prerequisites.md) for the complete checklist.

### Phase 2: Create and Configure Environment

See [deployment-spec.md](deployment-spec.md) for config decisions and the sanitized config.
See [commands/create/README.md](commands/create/README.md) for running the `create template`, `validate`, and
`create environment` commands.

### Phase 3: Provision Infrastructure

See [commands/provision/README.md](commands/provision/README.md) for running the `provision` command and server
details.

### Phase 4: Configure Instance

<!-- TODO: Document `configure` command with output -->

### Phase 5: Release Application

<!-- TODO: Document `release` command with output -->

### Phase 6: Run Services

<!-- TODO: Document `run` command with output -->

### Phase 7: Verify Deployment

<!-- TODO: Document verification steps, test results, and service endpoints -->

## Service Endpoints

> Will be filled after deployment.

| Service      | URL | Status |
| ------------ | --- | ------ |
| HTTP Tracker | TBD | -      |
| UDP Tracker  | TBD | -      |
| Tracker API  | TBD | -      |
| Health Check | TBD | -      |
| Grafana      | TBD | -      |

## Cost

> Will be documented after choosing server type.

| Resource | Monthly Cost (EUR) |
| -------- | ------------------ |
| Server   | TBD                |
| Total    | TBD                |
