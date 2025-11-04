use crate::detector::{
    AnsibleDetector, CargoMacheteDetector, DetectionError, LxdDetector, OpenTofuDetector,
    ToolDetector,
};

/// Result of checking a single dependency
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub tool: String,
    pub installed: bool,
}

/// Enum representing available dependencies
#[derive(Debug, Clone, Copy)]
pub enum Dependency {
    CargoMachete,
    OpenTofu,
    Ansible,
    Lxd,
}

/// Main dependency manager for detection operations
pub struct DependencyManager {
    detectors: Vec<Box<dyn ToolDetector>>,
}

impl DependencyManager {
    /// Create a new dependency manager with all detectors
    #[must_use]
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(CargoMacheteDetector),
                Box::new(OpenTofuDetector),
                Box::new(AnsibleDetector),
                Box::new(LxdDetector),
            ],
        }
    }

    /// Check all dependencies and return results
    ///
    /// # Errors
    ///
    /// Returns an error if any detection operation fails
    pub fn check_all(&self) -> Result<Vec<CheckResult>, DetectionError> {
        self.detectors
            .iter()
            .map(|detector| {
                let installed = detector.is_installed()?;
                Ok(CheckResult {
                    tool: detector.name().to_string(),
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
    pub fn get_detector(&self, dep: Dependency) -> Box<dyn ToolDetector> {
        match dep {
            Dependency::CargoMachete => Box::new(CargoMacheteDetector),
            Dependency::OpenTofu => Box::new(OpenTofuDetector),
            Dependency::Ansible => Box::new(AnsibleDetector),
            Dependency::Lxd => Box::new(LxdDetector),
        }
    }
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}
