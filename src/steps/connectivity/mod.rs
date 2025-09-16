//! Network connectivity and communication steps
//!
//! This module contains steps that manage network connectivity, communication,
//! and connection establishment operations. These steps ensure that network
//! services and connections are available before proceeding with deployment.
//!
//! ## Available Steps
//!
//! - `wait_ssh_connectivity` - SSH connectivity establishment and verification
//!
//! ## Key Features
//!
//! - Network connectivity verification and waiting mechanisms
//! - SSH connection establishment with retry logic
//! - Service availability checking and validation
//! - Integration with timeout and retry mechanisms
//!
//! These steps ensure that network communications are established and stable
//! before attempting configuration or deployment operations that require
//! remote connectivity.

pub mod wait_ssh_connectivity;

pub use wait_ssh_connectivity::WaitForSSHConnectivityStep;
