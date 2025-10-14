//! SSH wrapper module for secure remote operations
//!
//! This module provides comprehensive SSH functionality for secure remote command
//! execution and connectivity management. It includes credential management,
//! connection configuration, and client implementation with proper error handling.
//!
//! ## Module Components
//!
//! - `client` - SSH client implementation for remote command execution
//! - `config` - SSH configuration and management
//! - `credentials` - SSH authentication credentials and key management
//! - `error` - SSH error types and implementations
//! - `public_key` - SSH public key representation and validation
//! - `service_checker` - SSH service availability testing without authentication
//!
//! ## Key Features
//!
//! - Private key authentication with configurable credentials
//! - Connection timeout and retry mechanisms
//! - Secure remote command execution with error handling
//! - SSH service availability checking for connectivity testing
//! - Integration with deployment automation workflows
//!
//! The SSH wrapper is designed for automated deployment scenarios where
//! secure remote access is essential for configuration and management tasks.

pub mod client;
pub mod config;
pub mod credentials;
pub mod error;
pub mod public_key;
pub mod service_checker;

pub use client::SshClient;
pub use config::{
    SshConfig, SshConnectionConfig, DEFAULT_CONNECT_TIMEOUT_SECS, DEFAULT_MAX_RETRY_ATTEMPTS,
    DEFAULT_RETRY_INTERVAL_SECS, DEFAULT_RETRY_LOG_FREQUENCY, DEFAULT_SSH_PORT,
};
pub use credentials::SshCredentials;
pub use error::SshError;
pub use public_key::SshPublicKey;
pub use service_checker::SshServiceChecker;
