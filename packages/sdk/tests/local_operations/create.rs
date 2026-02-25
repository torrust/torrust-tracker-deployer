use torrust_tracker_deployer_sdk::CreateCommandHandlerError;

use super::{create_environment, deployer_in_temp_dir, minimal_config, write_config_json};

#[test]
fn it_should_create_and_show_an_environment() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    let env_name = create_environment(&deployer, "sdk-test-create");

    let info = deployer.show(&env_name).expect("show failed");

    assert_eq!(info.name, "sdk-test-create");
    assert_eq!(info.state, "Created");
    assert_eq!(info.provider, "LXD");
}

#[test]
fn it_should_return_error_when_creating_duplicate_environment() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    create_environment(&deployer, "sdk-test-dup");

    let result = deployer.create_environment(minimal_config("sdk-test-dup"));

    assert!(
        matches!(
            result,
            Err(CreateCommandHandlerError::EnvironmentAlreadyExists { .. })
        ),
        "expected EnvironmentAlreadyExists, got: {result:?}"
    );
}

#[test]
fn it_should_create_environment_from_json_file() {
    let (deployer, workspace) = deployer_in_temp_dir();

    let config_path = write_config_json(workspace.path(), "test-config.json", "sdk-test-from-file");

    let env_name = deployer
        .create_environment_from_file(&config_path)
        .expect("create_environment_from_file failed");

    assert_eq!(env_name.as_str(), "sdk-test-from-file");

    let info = deployer.show(&env_name).expect("show failed");
    assert_eq!(info.name, "sdk-test-from-file");
}
