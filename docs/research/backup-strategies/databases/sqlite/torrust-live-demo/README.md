# Torrust Live Demo Research

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This folder contains research and documentation related to the Torrust Live Demo (<https://github.com/torrust/torrust-demo>), which serves as a real-world production environment for testing backup strategies and performance improvements.

## Why the Live Demo Matters

The Torrust Live Demo is an invaluable lab for this research because:

- **Real production environment** with actual traffic
- **Metrics via Grafana** for measuring performance impact
- **Safe to experiment** - demo data is not critical
- **Quick feedback loop** - can apply and measure changes rapidly

Findings from the Live Demo will be applied to:

1. The Torrust Tracker Deployer (this project)
2. Future versions of the Torrust Demo itself

## Documents

| Document                                                     | Description                                   |
| ------------------------------------------------------------ | --------------------------------------------- |
| [Current Implementation Analysis](current-implementation.md) | Analysis of the current backup script         |
| [Proposed Improvements](proposed-improvements.md)            | Issues to open on the torrust-demo repository |

## Related GitHub Issues (torrust-demo)

Issues created based on this research:

- [ ] Use `.backup` command instead of `cp` for SQLite backups
- [ ] Evaluate WAL mode for improved performance

## Status

- [x] Analyze current backup implementation
- [x] Identify journal mode (delete)
- [x] Document proposed improvements
- [ ] Open issues on torrust-demo repository
- [ ] Measure performance after changes
