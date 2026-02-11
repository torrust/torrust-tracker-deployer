//! DNS resolution module
//!
//! This module provides DNS resolution capabilities for validating that configured
//! domains resolve to the expected IP addresses. This is used by the `test` command
//! to provide advisory warnings about DNS configuration without failing tests.
//!
//! ## Design Philosophy
//!
//! DNS resolution checks are **advisory only** - they provide early warnings about
//! potential DNS issues without blocking infrastructure tests. This separation is
//! intentional:
//!
//! - **Service tests**: Verify applications work (using internal IP resolution)
//! - **DNS checks**: Verify external access will work (using system DNS)
//!
//! ## Use Cases
//!
//! - Validating DNS configuration after deployment
//! - Detecting DNS propagation delays
//! - Troubleshooting accessibility issues
//! - Verifying `/etc/hosts` entries for `.local` domains
//!
//! ## Execution Context
//!
//! DNS resolution runs from the **deployer machine** (not inside the VM) to test
//! how external users would resolve domain names. This matches the real-world
//! access pattern where users connect from outside the infrastructure.

pub mod resolver;

pub use resolver::{DnsResolutionError, DnsResolver};
