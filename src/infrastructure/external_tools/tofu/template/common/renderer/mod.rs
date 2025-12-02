//! # `OpenTofu` Template Renderer
//!
//! This module handles `OpenTofu` template rendering for deployment workflows.
//! It manages the creation of build directories, copying template files, and processing them with
//! variable substitution.
//!
//! ## Provider Support
//!
//! The renderer supports multiple infrastructure providers (LXD, Hetzner) with independent
//! template sets for each provider. Templates are not shared between providers to allow
//! provider-specific customization.
//!
//! ## Future Improvements
//!
//! The following improvements could enhance this module's functionality and maintainability:
//!
//! 1. **Add comprehensive logging** - Add debug/trace logs for each operation step (directory
//!    creation, file copying, template processing) to improve debugging and monitoring.
//!
//! 2. **Extract constants for magic strings** - Create constants for hardcoded paths like "tofu",
//!    and file names to improve maintainability and reduce duplication.
//!
//! 3. **Add input validation** - Validate template names, check for empty strings, validate paths
//!    before processing to provide early error detection and better user feedback.
//!
//! 4. **Improve error messages** - Make error messages more user-friendly and actionable with
//!    suggestions for resolution, including common troubleshooting steps.
//!
//! 5. **Add configuration validation** - Pre-validate that required template files exist before
//!    starting the rendering process to avoid partial failures.
//!
//! 6. **Extract template discovery logic** - Separate the logic for finding and listing available
//!    templates to make it reusable and testable independently.
//!
//! 7. **Add progress reporting** - Add callback mechanism or progress indicators for long-running
//!    operations to improve user experience during deployment.
//!
//! 8. **Improve file operations** - Add more robust file copying with better error handling and
//!    atomic operations to prevent partial state corruption.
//!
//! 9. **Add template caching** - Cache parsed templates to improve performance for repeated
//!    operations and reduce I/O overhead.

pub mod cloud_init;
mod tofu_template_renderer;

pub use tofu_template_renderer::{TofuTemplateRenderer, TofuTemplateRendererError};
