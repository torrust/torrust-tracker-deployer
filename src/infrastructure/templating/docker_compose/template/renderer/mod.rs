//! # Docker Compose Template Renderer
//!
//! This module handles Docker Compose template rendering for deployment workflows.
//! It manages the creation of build directories and processing dynamic Tera templates
//! with runtime variables.
//!
//! ## Architecture
//!
//! Following the Project Generator pattern:
//! - **Project Generator (`DockerComposeProjectGenerator`)**: Orchestrates all template rendering
//! - **Renderers (`EnvRenderer`, `DockerComposeRenderer`)**: Handle specific template files (.env, docker-compose.yml)
//!
//! ## Key Features
//!
//! - **Dynamic template rendering**: Processes .tera templates with runtime variables
//! - **Structured error handling**: Provides specific error types with detailed context
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring
//! - **Testable design**: Modular structure that allows for comprehensive unit testing

pub mod docker_compose;
pub mod env;
mod project_generator;

pub use docker_compose::DockerComposeRenderer;
pub use env::EnvRenderer;
pub use project_generator::{DockerComposeProjectGenerator, DockerComposeProjectGeneratorError};
