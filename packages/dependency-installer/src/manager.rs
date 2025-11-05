//! Dependency management and detection coordination
//!
//! This module provides the main dependency manager that coordinates detection
//! operations for all supported dependencies.

// Standard library
use std::fmt;
use std::str::FromStr;

// Internal crate
use crate::detector::{
    AnsibleDetector, CargoMacheteDetector, DependencyDetector, DetectionError, LxdDetector,
    OpenTofuDetector,
};
use crate::installer::{
    AnsibleInstaller, CargoMacheteInstaller, DependencyInstaller, InstallationError, LxdInstaller,
    OpenTofuInstaller,
};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Enum representing available dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dependency {
    CargoMachete,
    OpenTofu,
    Ansible,
    Lxd,
}

/// Result of checking a single dependency
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// The dependency that was checked
    pub dependency: Dependency,
    /// Whether the dependency is installed
    pub installed: bool,
}

/// Result of installing a single dependency
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// The dependency that was installed
    pub dependency: Dependency,
    /// Whether the installation succeeded
    pub success: bool,
    /// Error message if installation failed
    pub error: Option<String>,
}

/// Main dependency manager for detection operations
pub struct DependencyManager;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

impl Dependency {
    /// Returns all available dependencies
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::CargoMachete, Self::OpenTofu, Self::Ansible, Self::Lxd]
    }

    /// Returns the canonical name for this dependency
    #[must_use]
    pub const fn canonical_name(&self) -> &'static str {
        match self {
            Self::CargoMachete => "cargo-machete",
            Self::OpenTofu => "opentofu",
            Self::Ansible => "ansible",
            Self::Lxd => "lxd",
        }
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

impl FromStr for Dependency {
    type Err = DependencyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cargo-machete" | "machete" => Ok(Self::CargoMachete),
            "opentofu" | "tofu" => Ok(Self::OpenTofu),
            "ansible" => Ok(Self::Ansible),
            "lxd" => Ok(Self::Lxd),
            _ => Err(DependencyParseError::UnknownDependency {
                name: s.to_string(),
            }),
        }
    }
}

impl DependencyManager {
    /// Create a new dependency manager
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Check all dependencies and return results
    ///
    /// # Errors
    ///
    /// Returns an error if any detection operation fails
    pub fn check_all(&self) -> Result<Vec<CheckResult>, DetectionError> {
        Dependency::all()
            .iter()
            .map(|&dependency| {
                let detector = self.get_detector(dependency);
                let installed = detector.is_installed()?;
                Ok(CheckResult {
                    dependency,
                    installed,
                })
            })
            .collect()
    }

    /// Get a specific detector by dependency type
    ///
    /// Note: This creates a new detector instance on each call, which is acceptable
    /// since detectors are lightweight and stateless.
    #[must_use]
    pub fn get_detector(&self, dep: Dependency) -> Box<dyn DependencyDetector> {
        match dep {
            Dependency::CargoMachete => Box::new(CargoMacheteDetector),
            Dependency::OpenTofu => Box::new(OpenTofuDetector),
            Dependency::Ansible => Box::new(AnsibleDetector),
            Dependency::Lxd => Box::new(LxdDetector),
        }
    }

    /// Get a specific installer by dependency type
    ///
    /// Note: This creates a new installer instance on each call, which is acceptable
    /// since installers are lightweight and stateless.
    #[must_use]
    pub fn get_installer(&self, dep: Dependency) -> Box<dyn DependencyInstaller> {
        match dep {
            Dependency::CargoMachete => Box::new(CargoMacheteInstaller),
            Dependency::OpenTofu => Box::new(OpenTofuInstaller),
            Dependency::Ansible => Box::new(AnsibleInstaller),
            Dependency::Lxd => Box::new(LxdInstaller),
        }
    }

    /// Install a specific dependency
    ///
    /// # Errors
    ///
    /// Returns an error if the installation process fails
    pub async fn install(&self, dep: Dependency) -> Result<(), InstallationError> {
        let installer = self.get_installer(dep);
        installer.install().await
    }

    /// Install all dependencies and return results
    ///
    /// This method attempts to install all dependencies, collecting results
    /// for each one. Even if some installations fail, all dependencies will
    /// be attempted.
    pub async fn install_all(&self) -> Vec<InstallResult> {
        let mut results = Vec::new();

        for &dependency in Dependency::all() {
            let result = match self.install(dependency).await {
                Ok(()) => InstallResult {
                    dependency,
                    success: true,
                    error: None,
                },
                Err(e) => InstallResult {
                    dependency,
                    success: false,
                    error: Some(e.to_string()),
                },
            };
            results.push(result);
        }

        results
    }
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Error that occurs when parsing a dependency name
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyParseError {
    /// Unknown dependency name provided
    UnknownDependency { name: String },
}

impl fmt::Display for DependencyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDependency { name } => {
                let available = Dependency::all()
                    .iter()
                    .map(Dependency::canonical_name)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Unknown dependency: {name}. Available: {available}")
            }
        }
    }
}

impl std::error::Error for DependencyParseError {}
