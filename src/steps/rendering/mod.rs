/*!
 * Rendering Steps
 *
 * This module contains steps that handle template and configuration generation.
 * These steps prepare configuration files and templates for deployment.
 *
 * Current steps:
 * - `OpenTofu` template rendering
 * - Ansible template rendering
 *
 * Future steps may include:
 * - Docker Compose configuration generation
 * - Environment variable file generation
 * - Custom configuration template rendering
 */

pub mod ansible_templates;
pub mod opentofu_templates;

pub use ansible_templates::{RenderAnsibleTemplatesError, RenderAnsibleTemplatesStep};
pub use opentofu_templates::RenderOpenTofuTemplatesStep;
