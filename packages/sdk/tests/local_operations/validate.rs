use super::{deployer_in_temp_dir, write_config_json};

#[test]
fn it_should_validate_a_valid_config_file() {
    let (deployer, workspace) = deployer_in_temp_dir();

    let config_path =
        write_config_json(workspace.path(), "validate-test.json", "sdk-test-validate");

    let result = deployer.validate(&config_path).expect("validate failed");

    assert_eq!(result.environment_name, "sdk-test-validate");
    assert_eq!(result.provider, "lxd");
    assert!(!result.has_prometheus);
    assert!(!result.has_grafana);
    assert!(!result.has_https);
    assert!(!result.has_backup);
}

#[test]
fn it_should_return_error_when_validating_invalid_json() {
    let (deployer, workspace) = deployer_in_temp_dir();

    let config_path = workspace.path().join("invalid.json");
    std::fs::write(&config_path, "{ not valid json }").expect("Failed to write config file");

    let result = deployer.validate(&config_path);

    assert!(
        result.is_err(),
        "expected validation error for invalid JSON"
    );
}
