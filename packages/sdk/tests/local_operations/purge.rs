use super::{
    assert_environment_exists, assert_environment_not_exists, create_environment,
    deployer_in_temp_dir,
};

#[test]
fn it_should_purge_environment_completely() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    let env_name = create_environment(&deployer, "sdk-test-purge");

    assert_environment_exists(&deployer, &env_name);

    deployer.purge(&env_name).expect("purge failed");

    assert_environment_not_exists(&deployer, &env_name);
}
