//! # Docker Compose Template Renderer
//!
//! This module handles Docker Compose template rendering for deployment workflows.
//! It manages the creation of build directories, copying static template files,
//! and processing dynamic Tera templates with runtime variables.
//!
//! ## Architecture
//!
//! Following the Project Generator pattern:
//! - **Project Generator (`DockerComposeProjectGenerator`)**: Orchestrates all template rendering
//! - **Renderers (`EnvRenderer`)**: Handle specific template files (.env)
//!
//! ## Key Features
//!
//! - **Static file copying**: Handles Docker Compose files that don't need Tera templating
//! - **Dynamic template rendering**: Processes .tera templates with runtime variables
//! - **Structured error handling**: Provides specific error types with detailed context
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring
//! - **Testable design**: Modular structure that allows for comprehensive unit testing

pub mod env;
mod project_generator;

pub use env::EnvRenderer;
pub use project_generator::{DockerComposeProjectGenerator, DockerComposeProjectGeneratorError};
