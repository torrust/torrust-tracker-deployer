//! Filesystem-based persistence infrastructure
//!
//! This module provides filesystem-based implementations for data persistence,
//! including file locks, JSON repositories, and environment repositories.

pub mod file_environment_repository;
pub mod file_lock;
pub mod json_file_repository;
mod platform;
