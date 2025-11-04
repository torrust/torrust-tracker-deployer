//! Unit tests for detector functionality
//!
//! Tests for the `ToolDetector` trait implementations including:
//! - Individual detector implementations
//! - `DependencyManager` functionality
//! - Error handling

use torrust_dependency_installer::{
    AnsibleDetector, CargoMacheteDetector, CheckResult, Dependency, DependencyManager, LxdDetector,
    OpenTofuDetector, ToolDetector,
};

// =============================================================================
// DETECTOR TRAIT TESTS
// =============================================================================

#[test]
fn it_should_return_cargo_machete_detector_name() {
    let detector = CargoMacheteDetector;
    assert_eq!(detector.name(), "cargo-machete");
}

#[test]
fn it_should_return_opentofu_detector_name() {
    let detector = OpenTofuDetector;
    assert_eq!(detector.name(), "OpenTofu");
}

#[test]
fn it_should_return_ansible_detector_name() {
    let detector = AnsibleDetector;
    assert_eq!(detector.name(), "Ansible");
}

#[test]
fn it_should_return_lxd_detector_name() {
    let detector = LxdDetector;
    assert_eq!(detector.name(), "LXD");
}

// =============================================================================
// DETECTOR INSTALLATION CHECK TESTS
// =============================================================================
//
// Note: These tests check if the detectors can run without errors.
// The actual installation status depends on the system, so we only verify
// that the detection logic executes successfully without panicking.

#[test]
fn it_should_run_cargo_machete_detector_without_error() {
    let detector = CargoMacheteDetector;
    // Should not panic - result depends on system state
    let result = detector.is_installed();
    assert!(result.is_ok(), "Detection should not error");
}

#[test]
fn it_should_run_opentofu_detector_without_error() {
    let detector = OpenTofuDetector;
    // Should not panic - result depends on system state
    let result = detector.is_installed();
    assert!(result.is_ok(), "Detection should not error");
}

#[test]
fn it_should_run_ansible_detector_without_error() {
    let detector = AnsibleDetector;
    // Should not panic - result depends on system state
    let result = detector.is_installed();
    assert!(result.is_ok(), "Detection should not error");
}

#[test]
fn it_should_run_lxd_detector_without_error() {
    let detector = LxdDetector;
    // Should not panic - result depends on system state
    let result = detector.is_installed();
    assert!(result.is_ok(), "Detection should not error");
}

// =============================================================================
// DETECTOR REQUIRED VERSION TESTS
// =============================================================================

#[test]
fn it_should_return_no_required_version_by_default_for_all_detectors() {
    let cargo_machete = CargoMacheteDetector;
    let opentofu = OpenTofuDetector;
    let ansible = AnsibleDetector;
    let lxd = LxdDetector;

    assert_eq!(cargo_machete.required_version(), None);
    assert_eq!(opentofu.required_version(), None);
    assert_eq!(ansible.required_version(), None);
    assert_eq!(lxd.required_version(), None);
}

// =============================================================================
// DEPENDENCY MANAGER TESTS
// =============================================================================

#[test]
fn it_should_create_dependency_manager() {
    let manager = DependencyManager::new();
    // Should not panic
    drop(manager);
}

#[test]
fn it_should_create_dependency_manager_with_default() {
    let manager = DependencyManager::default();
    // Should not panic
    drop(manager);
}

#[test]
fn it_should_check_all_dependencies_without_error() {
    let manager = DependencyManager::new();
    let results = manager.check_all();

    // Should not panic - result depends on system state
    assert!(results.is_ok(), "check_all should not error");

    // Verify we get results for all 4 tools
    let check_results = results.unwrap();
    assert_eq!(check_results.len(), 4, "Should check 4 dependencies");

    // Verify all expected tools are in results
    let tool_names: Vec<String> = check_results.iter().map(|r| r.tool.clone()).collect();
    assert!(tool_names.contains(&"cargo-machete".to_string()));
    assert!(tool_names.contains(&"OpenTofu".to_string()));
    assert!(tool_names.contains(&"Ansible".to_string()));
    assert!(tool_names.contains(&"LXD".to_string()));
}

#[test]
fn it_should_get_cargo_machete_detector_from_manager() {
    let manager = DependencyManager::new();
    let detector = manager.get_detector(Dependency::CargoMachete);
    assert_eq!(detector.name(), "cargo-machete");
}

#[test]
fn it_should_get_opentofu_detector_from_manager() {
    let manager = DependencyManager::new();
    let detector = manager.get_detector(Dependency::OpenTofu);
    assert_eq!(detector.name(), "OpenTofu");
}

#[test]
fn it_should_get_ansible_detector_from_manager() {
    let manager = DependencyManager::new();
    let detector = manager.get_detector(Dependency::Ansible);
    assert_eq!(detector.name(), "Ansible");
}

#[test]
fn it_should_get_lxd_detector_from_manager() {
    let manager = DependencyManager::new();
    let detector = manager.get_detector(Dependency::Lxd);
    assert_eq!(detector.name(), "LXD");
}

// =============================================================================
// CHECK RESULT TESTS
// =============================================================================

#[test]
fn it_should_create_check_result() {
    let result = CheckResult {
        tool: "test-tool".to_string(),
        installed: true,
    };

    assert_eq!(result.tool, "test-tool");
    assert!(result.installed);
}

#[test]
fn it_should_clone_check_result() {
    let result = CheckResult {
        tool: "test-tool".to_string(),
        installed: false,
    };

    let cloned = result.clone();
    assert_eq!(cloned.tool, "test-tool");
    assert!(!cloned.installed);
}

// =============================================================================
// COMMAND UTILITY TESTS
// =============================================================================

#[test]
fn it_should_detect_existing_command() {
    use torrust_dependency_installer::command::command_exists;

    // Test with 'sh' which should always exist on Unix systems
    let result = command_exists("sh");
    assert!(result.is_ok());
    // 'sh' should exist
    assert!(result.unwrap());
}

#[test]
fn it_should_detect_nonexistent_command() {
    use torrust_dependency_installer::command::command_exists;

    // Test with a command that definitely doesn't exist
    let result = command_exists("this-command-definitely-does-not-exist-12345");
    assert!(result.is_ok());
    // Command should not exist
    assert!(!result.unwrap());
}

#[test]
fn it_should_execute_command_successfully() {
    use torrust_dependency_installer::command::execute_command;

    // Test with 'echo' which should always work
    let result = execute_command("echo", &["hello"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn it_should_fail_to_execute_nonexistent_command() {
    use torrust_dependency_installer::command::execute_command;

    // Test with nonexistent command
    let result = execute_command("this-command-definitely-does-not-exist-12345", &["test"]);
    assert!(result.is_err());
}
