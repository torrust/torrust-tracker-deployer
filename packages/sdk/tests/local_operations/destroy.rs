use super::{create_environment, deployer_in_temp_dir};

#[test]
fn it_should_destroy_a_created_environment() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    let env_name = create_environment(&deployer, "sdk-test-destroy");

    // Destroy transitions the state but does not remove local data.
    deployer.destroy(&env_name).expect("destroy failed");

    // The environment should still be visible (state changes, not deleted).
    let info = deployer.show(&env_name).expect("show after destroy failed");
    assert_eq!(info.name, "sdk-test-destroy");
}
