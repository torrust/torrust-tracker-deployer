use torrust_tracker_deployer_sdk::Deployer;

#[test]
fn it_should_return_error_when_building_deployer_without_working_dir() {
    let result = Deployer::builder().build();

    assert!(result.is_err(), "expected MissingWorkingDir error");
}
