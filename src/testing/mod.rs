//! Testing utilities and fixtures
//!
//! This module provides reusable test fixtures and helpers for testing
//! various components of the application.
//!
//! ## Module Structure
//!
//! - `e2e` - End-to-end testing infrastructure and scenarios
//! - `integration` - Integration testing utilities
//! - `fixtures` - Reusable test fixtures
//! - `mock_clock` - Mock clock implementation for deterministic time testing
//! - `network` - Network testing utilities (port checking, connectivity testing)

pub mod e2e;
pub mod fixtures;
pub mod integration;
pub mod mock_clock;
pub mod network;

// Re-export commonly used testing types
pub use mock_clock::MockClock;
pub use network::{PortChecker, PortCheckerError, PortUsageChecker, PortUsageError};

// Re-export E2E types for convenience
pub use e2e::{
    container::Services,
    containers::{ContainerError, RunningProvisionedContainer, StoppedProvisionedContainer},
    context::{TestContext, TestContextType},
};
