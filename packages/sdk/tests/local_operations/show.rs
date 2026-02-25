use torrust_tracker_deployer_sdk::{EnvironmentName, ShowCommandHandlerError};

use super::deployer_in_temp_dir;

#[test]
fn it_should_return_error_when_showing_non_existent_environment() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    let name = EnvironmentName::new("does-not-exist").expect("invalid name");
    let result = deployer.show(&name);

    assert!(
        matches!(
            result,
            Err(ShowCommandHandlerError::EnvironmentNotFound { .. })
        ),
        "expected EnvironmentNotFound, got: {result:?}"
    );
}
