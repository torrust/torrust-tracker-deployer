use torrust_tracker_deployer_sdk::EnvironmentName;

use super::{
    assert_environment_exists, assert_environment_not_exists, create_environment,
    deployer_in_temp_dir,
};

#[test]
fn it_should_report_exists_correctly() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    let name = EnvironmentName::new("sdk-test-exists").expect("invalid name");

    assert_environment_not_exists(&deployer, &name);

    create_environment(&deployer, "sdk-test-exists");

    assert_environment_exists(&deployer, &name);
}
