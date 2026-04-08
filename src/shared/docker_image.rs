//! Docker image reference value object
//!
//! This module provides a strongly-typed Docker image reference that combines
//! a repository name with a tag. It is used to represent Docker images in
//! service configurations and templates.
//!
//! # Design Decision
//!
//! Docker image versions are **not user-configurable** — they are pinned as
//! constants in the code to ensure compatibility between the deployer and the
//! images it uses. Exposing them through domain configs (rather than hardcoding
//! in templates) gives us:
//!
//! - A **single source of truth** for each image version
//! - The ability to **inspect images via the `show` command**
//! - Automatic propagation of version changes to both templates and CI scanning
//!
//! # Examples
//!
//! ```rust
//! use torrust_tracker_deployer_lib::shared::docker_image::DockerImage;
//!
//! let image = DockerImage::new("torrust/tracker", "develop");
//! assert_eq!(image.full_reference(), "torrust/tracker:develop");
//! assert_eq!(image.repository(), "torrust/tracker");
//! assert_eq!(image.tag(), "develop");
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

/// Docker image reference with repository and tag
///
/// Represents an image reference of the form `repository:tag`,
/// e.g. `torrust/tracker:develop` or `mysql:8.4`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerImage {
    repository: String,
    tag: String,
}

impl DockerImage {
    /// Creates a new Docker image reference
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::shared::docker_image::DockerImage;
    ///
    /// let image = DockerImage::new("torrust/tracker", "develop");
    /// assert_eq!(image.repository(), "torrust/tracker");
    /// assert_eq!(image.tag(), "develop");
    /// ```
    #[must_use]
    pub fn new(repository: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            tag: tag.into(),
        }
    }

    /// Returns the repository name (e.g. `"torrust/tracker"`)
    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Returns the image tag (e.g. `"develop"` or `"8.4"`)
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the full image reference as `repository:tag`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::shared::docker_image::DockerImage;
    ///
    /// let image = DockerImage::new("mysql", "8.4");
    /// assert_eq!(image.full_reference(), "mysql:8.4");
    /// ```
    #[must_use]
    pub fn full_reference(&self) -> String {
        format!("{}:{}", self.repository, self.tag)
    }
}

impl fmt::Display for DockerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.repository, self.tag)
    }
}

impl From<(&str, &str)> for DockerImage {
    fn from((repository, tag): (&str, &str)) -> Self {
        Self::new(repository, tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_docker_image_with_repository_and_tag() {
        let image = DockerImage::new("torrust/tracker", "develop");

        assert_eq!(image.repository(), "torrust/tracker");
        assert_eq!(image.tag(), "develop");
    }

    #[test]
    fn it_should_return_full_reference_as_repository_colon_tag() {
        let image = DockerImage::new("torrust/tracker", "develop");

        assert_eq!(image.full_reference(), "torrust/tracker:develop");
    }

    #[test]
    fn it_should_display_as_full_reference() {
        let image = DockerImage::new("mysql", "8.4");

        assert_eq!(format!("{image}"), "mysql:8.4");
    }

    #[test]
    fn it_should_create_from_str_tuple() {
        let image = DockerImage::from(("prom/prometheus", "v3.5.0"));

        assert_eq!(image.full_reference(), "prom/prometheus:v3.5.1");
    }

    #[test]
    fn it_should_implement_equality() {
        let a = DockerImage::new("grafana/grafana", "12.3.1");
        let b = DockerImage::new("grafana/grafana", "12.3.1");
        let c = DockerImage::new("grafana/grafana", "11.4.0");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
