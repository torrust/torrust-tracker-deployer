# Deployment Journal: Hetzner Demo Tracker

**Issue**: [#405](https://github.com/torrust/torrust-tracker-deployer/issues/405)
**Date started**: 2026-03-03
**Domain**: `torrust-tracker-demo.com`
**Provider**: Hetzner Cloud

## Purpose

Deploy a public Torrust Tracker demo instance to Hetzner Cloud and document every step of the process. This journal will serve as the source material for a blog post on [torrust.com](https://torrust.com).

## Table of Contents

1. [Prerequisites](prerequisites.md) — Account setup, tools, SSH keys
2. [Configuration](configuration.md) — Environment config decisions and examples
3. [Deployment](#deployment) — Step-by-step deployment walkthrough (below)
4. [Problems](problems.md) — Issues encountered with root causes and resolutions

## Deployment

> This section will be filled in as we execute each deployment phase.

### Phase 1: Setup and Prerequisites

See [prerequisites.md](prerequisites.md) for the complete checklist.

### Phase 2: Create and Configure Environment

See [configuration.md](configuration.md) for config decisions.

<!-- TODO: Document `create template` and `create environment` commands with output -->

### Phase 3: Provision Infrastructure

<!-- TODO: Document `provision` command with output and timing -->

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
